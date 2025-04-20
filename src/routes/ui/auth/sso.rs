use std::env::var;
use std::net::IpAddr;
use std::sync::{LazyLock, OnceLock};
use std::time::Duration;
use anyhow::anyhow;
use log::warn;
use moka::future::Cache;
use rocket::{get, post, uri};
use rocket::response::Redirect;
use rocket_session_store::Session;
use crate::guards::AuthUser;
use crate::SessionData;
use openidconnect::{reqwest, AccessTokenHash, AsyncHttpClient, AuthenticationFlow, AuthorizationCode, Client, ClientId, ClientSecret, CsrfToken, EmptyAdditionalClaims, HttpClientError, IssuerUrl, Nonce, OAuth2TokenResponse, PkceCodeChallenge, PkceCodeVerifier, ProviderMetadata, RedirectUrl, Scope, StandardErrorResponse, TokenResponse};
use openidconnect::core::{CoreAuthDisplay, CoreAuthPrompt, CoreAuthenticationFlow, CoreClient, CoreGenderClaim, CoreJsonWebKey, CoreJweContentEncryptionAlgorithm, CoreProviderMetadata, CoreTokenResponse, CoreUserInfoClaims};
use openidconnect::http::HeaderValue;
use reqwest::header::HeaderMap;
// TODO: not have this lazy somehow, move to OnceLock and have fn to refresh it? (own module?)
// and/or also move to State<>

static HTTP_CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| {
    let mut headers = HeaderMap::new();
    // TODO: pull from config.
    // Set referrer as some providers (authentik) block POST w/o referrer
    headers.insert("Referer", HeaderValue::from_static("http://localhost:8080"));
    let mut builder = reqwest::ClientBuilder::new()
        // Following redirects opens the client up to SSRF vulnerabilities.
        .redirect(reqwest::redirect::Policy::none())
        .default_headers(headers);
    if var("DANGER_DEV_PROXY").is_ok() {
        warn!("DANGER_DEV_PROXY set, requests are being proxied & ignoring certificates");
        builder = builder
            .proxy(reqwest::Proxy::https("https://localhost:8082").unwrap())
            .danger_accept_invalid_certs(true)
    };
    builder.build().expect("Client should build")
});
#[derive(Clone)]
struct SSOSessionData {
    pkce_challenge: String,
    nonce: Nonce,
    csrf_token: CsrfToken
    // ip: IpAddr,
}
static SSO_SESSION_CACHE: LazyLock<Cache<IpAddr, SSOSessionData>> = LazyLock::new(|| Cache::builder()
    .time_to_live(Duration::from_secs(120))
    .max_capacity(100)
    .build());
#[get("/auth/sso")]
pub async fn page(ip: IpAddr) -> Redirect {
    let http_client = HTTP_CLIENT.clone();
    // FIXME: temp, remove
    let provider_metadata = CoreProviderMetadata::discover_async(
        /* TODO: pull from config */
        IssuerUrl::new(var("SSO_ISSUER_URL").expect("dev: missing sso url")).expect("bad issuer url"),
        &http_client,
    ).await.map_err(|e| e.to_string()).expect("discovery failed");
    let client =
        CoreClient::from_provider_metadata(
            provider_metadata,
            // TODO: pull from config
            ClientId::new(var("SSO_CLIENT_ID").expect("dev: sso client id missing")),
            Some(ClientSecret::new(var("SSO_CLIENT_SECRET").expect("dev sso client secret missing")
                .to_string())),
        ).set_redirect_uri(RedirectUrl::new("http://localhost:8080/auth/sso/cb".to_string()).unwrap());

    // Generate a PKCE challenge.
    // TODO: store in hashmap for request ip? leaky bucket?
    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    // Generate the full authorization URL.
    let (auth_url, csrf_token, nonce) = client
        .authorize_url(
            CoreAuthenticationFlow::AuthorizationCode,
            CsrfToken::new_random,
            Nonce::new_random,
        )
        // Set the desired scopes.
        // TODO: change scopes
        .add_scope(Scope::new("email".to_string()))
        .add_scope(Scope::new("name".to_string()))
        // Set the PKCE code challenge.
        .set_pkce_challenge(pkce_challenge)
        .url();
    SSO_SESSION_CACHE.insert(ip, SSOSessionData {
        nonce: nonce,
        pkce_challenge: pkce_verifier.into_secret(),
        csrf_token
    }).await;

    Redirect::to(auth_url.to_string())

    // This is the URL you should redirect the user to, in order to trigger the authorization
    // process.
}

#[get("/auth/sso/cb?<code>&<state>")]
pub async fn callback(session: Session<'_, SessionData>, ip: IpAddr, code: String, state: String) -> Result<String, String> {
    let session_data = SSO_SESSION_CACHE.remove(&ip).await.ok_or_else(|| "no sso session started".to_string())?;
    // Now you can exchange it for an access token and ID token.
    if &state != session_data.csrf_token.secret() {
        return Err(format!("csrf validation failed {}", state));
    }

    // FIXME: temp, remove
    let http_client = HTTP_CLIENT.clone();
    let provider_metadata = CoreProviderMetadata::discover_async(
        /* TODO: pull from config */
        IssuerUrl::new(var("SSO_ISSUER_URL").expect("dev: missing sso url")).expect("bad issuer url"),
        &http_client,
    ).await.expect("discovery failed");
    let client =
        CoreClient::from_provider_metadata(
            provider_metadata,
            // TODO: pull from config
            ClientId::new(var("SSO_CLIENT_ID").expect("dev: sso client id missing")),
            Some(ClientSecret::new(var("SSO_CLIENT_SECRET").expect("dev sso client secret missing")
                .to_string())),
        ).set_redirect_uri(RedirectUrl::new("http://localhost:8080/auth/sso/cb".to_string()).unwrap());

    let token_response =
        client
            .exchange_code(AuthorizationCode::new(code)).expect("bad auth code")
            // Set the PKCE code verifier.
            .set_pkce_verifier(PkceCodeVerifier::new(session_data.pkce_challenge)) // TODO: somehow have this??
            .request_async(&http_client).await.expect("token exchange error");

    // Extract the ID token claims after verifying its authenticity and nonce.
    let id_token = token_response
        .id_token()
        .ok_or_else(|| "Server did not return an ID token".to_string())?;
    let id_token_verifier = client.id_token_verifier();
    let claims = id_token.claims(&id_token_verifier, &session_data.nonce).expect("bad claims"); // TODO: and this?

    // Verify the access token hash to ensure that the access token hasn't been substituted for
    // another user's.
    if let Some(expected_access_token_hash) = claims.access_token_hash() {
        let actual_access_token_hash = AccessTokenHash::from_token(
            token_response.access_token(),
            id_token.signing_alg().expect("signing failed (alg)"),
            id_token.signing_key(&id_token_verifier).expect("signing failed (key)"),
        ).expect("access token resolve error");
        if actual_access_token_hash != *expected_access_token_hash {
            return Err("Invalid access token".to_string());
        }
    }

    // The authenticated user's identity is now available. See the IdTokenClaims struct for a
    // complete listing of the available claims.
    println!(
        "User {} with e-mail address {} has authenticated successfully",
        claims.subject().as_str(),
        claims.email().map(|email| email.as_str()).unwrap_or("<not provided>"),
    );

    // If available, we can use the user info endpoint to request additional information.

    // The user_info request uses the AccessToken returned in the token response. To parse custom
    // claims, use UserInfoClaims directly (with the desired type parameters) rather than using the
    // CoreUserInfoClaims type alias.
    let userinfo: CoreUserInfoClaims = client
        .user_info(token_response.access_token().to_owned(), None).expect("user info missing")
        .request_async(&http_client)
        .await
        .map_err(|err| format!("Failed requesting user info: {}", err))?;
    Ok(format!("user={:?}\nemail={:?}\nname={:?}", userinfo.subject(), userinfo.email(), userinfo.name()))
}
