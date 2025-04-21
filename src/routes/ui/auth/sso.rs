use std::env::var;
use std::net::IpAddr;
use std::sync::{LazyLock, OnceLock};
use std::time::Duration;
use anyhow::{anyhow, Error};
use log::{debug, warn};
use moka::future::Cache;
use rocket::{get, post, uri, State};
use rocket::response::Redirect;
use rocket_session_store::Session;
use crate::guards::AuthUser;
use crate::SessionData;
use openidconnect::{reqwest, AccessTokenHash, AsyncHttpClient, AuthenticationFlow, AuthorizationCode, Client, ClientId, ClientSecret, CsrfToken, EmptyAdditionalClaims, HttpClientError, IssuerUrl, Nonce, OAuth2TokenResponse, PkceCodeChallenge, PkceCodeVerifier, ProviderMetadata, RedirectUrl, Scope, StandardErrorResponse, TokenResponse};
use openidconnect::core::{CoreAuthDisplay, CoreAuthPrompt, CoreAuthenticationFlow, CoreClient, CoreGenderClaim, CoreJsonWebKey, CoreJweContentEncryptionAlgorithm, CoreProviderMetadata, CoreTokenResponse, CoreUserInfoClaims};
use openidconnect::http::HeaderValue;
use reqwest::header::HeaderMap;
use rocket::http::{Header, Status};
use rocket_dyn_templates::{context, Template};
use tokio::sync::MutexGuard;
use crate::managers::sso::{SSOSessionData, SSOState, SSO};
use crate::routes::ui::auth::HackyRedirectBecauseRocketBug;

async fn page_handler(sso: &State<SSOState>, ip: IpAddr, return_to: Option<String>) -> Result<Redirect, anyhow::Error> {
    let mut sso = sso.as_ref().ok_or_else(|| anyhow!("SSO is not configured"))?.lock().await;
    let client = sso.create_client_redirect().await?;
    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();
    let (auth_url, csrf_token, nonce) = client
        .authorize_url(
            CoreAuthenticationFlow::AuthorizationCode,
            CsrfToken::new_random,
            Nonce::new_random,
        )
        // Set the desired scopes.
        .add_scopes(sso.scopes())
        // Set the PKCE code challenge.
        .set_pkce_challenge(pkce_challenge)
        .url();
    sso.cache_set(ip, SSOSessionData {
        nonce: nonce,
        pkce_challenge: pkce_verifier.into_secret(),
        csrf_token,
        return_to
    }).await;
    Ok(Redirect::to(auth_url.to_string()))
}
#[get("/auth/sso?<return_to>")]
pub async fn page(ip: IpAddr, sso: &State<SSOState>, return_to: Option<String>) -> Result<Redirect, (Status, Template)> {
    page_handler(sso, ip, return_to).await
        .map_err(|e| (Status::InternalServerError, Template::render("errors/500", context! {
                error: e.to_string()
            })))
}

async fn callback_handler(sso: &State<SSOState>, ip: IpAddr, code: String, state: String) -> Result<(CoreUserInfoClaims, Option<String>), anyhow::Error> {
    let mut sso = sso.as_ref().ok_or_else(||anyhow!("SSO is not configured"))?.lock().await;
    let sess_data = sso.cache_take(ip).await.ok_or_else(|| anyhow!("No valid sso started"))?;
    if &state != sess_data.csrf_token.secret() {
        return Err(anyhow!("CSRF verification failed"));
    }
    let client = sso.create_client_redirect().await?;
    let token_response =
        client
            .exchange_code(AuthorizationCode::new(code)).map_err(|e| anyhow!("oidc code is invalid"))?
            // Set the PKCE code verifier.
            .set_pkce_verifier(PkceCodeVerifier::new(sess_data.pkce_challenge)) // TODO: somehow have this??
            .request_async(sso.http_client()).await
                .map_err(|e| anyhow!("OIDC Token exchange error"))?;

    // Extract the ID token claims after verifying its authenticity and nonce.
    let id_token = token_response
        .id_token()
        .ok_or_else(|| anyhow!("Server did not return an ID token"))?;
    let id_token_verifier = client.id_token_verifier();
    let claims = id_token.claims(&id_token_verifier, &sess_data.nonce).map_err(|e| anyhow!("OIDC Token claims error: {}", e))?;

    // Verify the access token hash to ensure that the access token hasn't been substituted for another user's.
    if let Some(expected_access_token_hash) = claims.access_token_hash() {
        let actual_access_token_hash = AccessTokenHash::from_token(
            token_response.access_token(),
            id_token.signing_alg().map_err(|e| anyhow!("OIDC token signature error: {}", e))?,
            id_token.signing_key(&id_token_verifier).map_err(|e| anyhow!("OIDC token signature error: {}", e))?
        ).expect("access token resolve error");
        if actual_access_token_hash != *expected_access_token_hash {
            return Err(anyhow!("Invalid access token"))
        }
    }

    // If available, we can use the user info endpoint to request additional information.

    // The user_info request uses the AccessToken returned in the token response. To parse custom
    // claims, use UserInfoClaims directly (with the desired type parameters) rather than using the
    // CoreUserInfoClaims type alias.
    let userinfo: CoreUserInfoClaims = client
        .user_info(token_response.access_token().to_owned(), None).map_err(|_| anyhow!("could not acquire user data"))?
        .request_async(sso.http_client())
        .await
        .map_err(|_| anyhow!("could not acquire user data"))?;
    Ok((userinfo, sess_data.return_to))
}

#[get("/auth/sso/cb?<code>&<state>")]
pub async fn callback(session: Session<'_, SessionData>, ip: IpAddr, sso: &State<SSOState>, code: String, state: String) -> Result<HackyRedirectBecauseRocketBug, (Status, Template)> {
    let (userinfo, return_to) = callback_handler(sso, ip, code, state).await
        .map_err(|e| (Status::InternalServerError, Template::render("errors/500", context! {
                error: e.to_string()
            })))?;
    debug!("user={:?}\nemail={:?}\nname={:?}", userinfo.subject(), userinfo.email(), userinfo.name());
    let return_to = return_to.unwrap_or("/".to_string());
    Ok(HackyRedirectBecauseRocketBug {
        inner: "Login successful, redirecting...".to_string(),
        location: Header::new("Location", return_to),
    })
}
