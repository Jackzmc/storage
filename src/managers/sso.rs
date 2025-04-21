use std::env::var;
use std::net::IpAddr;
use std::sync::{Arc, LazyLock};
use std::time::Duration;
use anyhow::anyhow;
use log::{info, warn};
use moka::future::Cache;
use openidconnect::core::{CoreAuthDisplay, CoreAuthPrompt, CoreClaimName, CoreClient, CoreErrorResponseType, CoreGenderClaim, CoreJsonWebKey, CoreJweContentEncryptionAlgorithm, CoreProviderMetadata, CoreRevocableToken, CoreRevocationErrorResponse, CoreTokenIntrospectionResponse, CoreTokenResponse};
use openidconnect::http::{HeaderMap, HeaderValue};
use openidconnect::{Client, ClientId, ClientSecret, CsrfToken, EmptyAdditionalClaims, EndpointMaybeSet, EndpointNotSet, EndpointSet, IssuerUrl, Nonce, ProviderMetadata, RedirectUrl, Scope, StandardErrorResponse};
use openidconnect::url::ParseError;
use rocket::yansi::Paint;
use tokio::sync::Mutex;
use crate::config::{AppConfig, OidcConfig};

pub struct SSO {
    http_client: reqwest::Client,
    issuer_url: IssuerUrl,
    client_id: ClientId,
    client_secret: Option<ClientSecret>,
    public_url: String,
    scopes: Vec<String>,
    cache: Cache<IpAddr, SSOSessionData>,
}
pub struct HttpProxySettings {
    url: String,
    disable_cert_check: bool
}

#[derive(Clone)]
pub struct SSOSessionData {
    pub pkce_challenge: String,
    pub nonce: Nonce,
    pub csrf_token: CsrfToken,
    pub return_to: Option<String>
    // ip: IpAddr,
}
pub type SSOState = Option<Arc<Mutex<SSO>>>;
impl SSO {
    pub async fn create(config: &AppConfig) -> Self {
        let oidc_config = config.auth.oidc.as_ref().expect("OIDC config not provided");
        let referer = config.general.get_public_url().domain().map(|s| s.to_string());
        let proxy_settings = SSO::setup_proxy();
        let http_client = SSO::setup_http_client(referer, proxy_settings);
        let issuer_url = IssuerUrl::new(oidc_config.issuer_url.to_string()).expect("bad issuer url");
        let client_id = ClientId::new(oidc_config.client_id.to_string());
        let client_secret = Some(ClientSecret::new(oidc_config.client_secret.to_string()));
        let cache = Self::setup_cache();
        Self {
            http_client,
            issuer_url,
            client_id,
            client_secret,
            cache,
            scopes: oidc_config.claims.to_owned(),
            public_url: config.general.public_url.to_string(),
        }
    }

    fn setup_cache() -> Cache<IpAddr, SSOSessionData> {
        Cache::builder()
            .time_to_live(Duration::from_secs(120))
            .max_capacity(100)
            .build()
    }

    fn setup_proxy() -> Option<HttpProxySettings> {
        if let Ok(proxy_url) = var("DEV_PROXY_URL") {
            return Some(HttpProxySettings {
                url: proxy_url,
                disable_cert_check: var("DEV_PROXY_DANGER_DISABLE_CERT_CHECK").is_ok()
            })
        }
        None
    }

    fn setup_http_client(referer: Option<String>, proxy_settings: Option<HttpProxySettings>) -> reqwest::Client {
        let mut headers = HeaderMap::new();
        // TODO: pull from config.
        // Set referrer as some providers (authentik) block POST w/o referrer
        if let Some(ref referer) = referer {
            headers.insert("Referer", referer.parse().expect("bad referer"));
        }
        let mut builder = reqwest::ClientBuilder::new()
            // Following redirects opens the client up to SSRF vulnerabilities.
            .redirect(reqwest::redirect::Policy::none())
            .default_headers(headers);
        if let Some(proxy) = proxy_settings {
            info!("Using proxy url: {}", proxy.url);
            if proxy.disable_cert_check {
                warn!("!! DEV_PROXY_DANGER_DISABLE_CERT_CHECK is set: requests are proxied, ignoring certificates");
            }
            builder = builder
                .proxy(reqwest::Proxy::https(proxy.url).unwrap())
                .danger_accept_invalid_certs(proxy.disable_cert_check);
        };
        builder.build().expect("Client should build")
    }
    pub fn http_client(&self) -> &reqwest::Client {
        &self.http_client
    }

    pub async fn create_client(&self) -> Result<OidcClient, anyhow::Error> {
        let provider_metadata = CoreProviderMetadata::discover_async(
            /* TODO: pull from config */
            self.issuer_url.clone(),
            &self.http_client,
        ).await.map_err(|e| anyhow!(e.to_string()))?;
        Ok(CoreClient::from_provider_metadata(
            provider_metadata,
            // TODO: pull from config
            self.client_id.clone(),
            self.client_secret.clone(),
        ))
    }

    pub async fn create_client_redirect(&self) -> Result<OidcClient, anyhow::Error> {
        let redirect_url = RedirectUrl::new( format!("{}/auth/sso/cb", self.public_url))
            .map_err(|e: ParseError | anyhow!(e))?;
        let client = self.create_client().await?;
        Ok(client.set_redirect_uri(redirect_url))
    }

    pub fn scopes(&self) -> Vec<Scope> {
        self.scopes.iter().map(|c| Scope::new(c.to_string())).collect()
    }

    pub async fn cache_set(&mut self, ip: IpAddr, data: SSOSessionData) {
        self.cache.insert(ip, data).await;
    }

    pub async fn cache_take(&mut self, ip: IpAddr) -> Option<SSOSessionData> {
        self.cache.remove(&ip).await
    }
}
// From https://github.com/IgnisDa/ryot/blob/75a1379f743b412df0e42fc88177d18cd34d48d7/crates/utils/application/src/lib.rs#L141C31-L148C2
pub type OidcClient<
    HasAuthUrl = EndpointSet,
    HasDeviceAuthUrl = EndpointNotSet,
    HasIntrospectionUrl = EndpointNotSet,
    HasRevocationUrl = EndpointNotSet,
    HasTokenUrl = EndpointMaybeSet,
    HasUserInfoUrl = EndpointMaybeSet,
> = Client<
    EmptyAdditionalClaims,
    CoreAuthDisplay,
    CoreGenderClaim,
    CoreJweContentEncryptionAlgorithm,
    CoreJsonWebKey,
    CoreAuthPrompt,
    StandardErrorResponse<CoreErrorResponseType>,
    CoreTokenResponse,
    CoreTokenIntrospectionResponse,
    CoreRevocableToken,
    CoreRevocationErrorResponse,
    HasAuthUrl,
    HasDeviceAuthUrl,
    HasIntrospectionUrl,
    HasRevocationUrl,
    HasTokenUrl,
    HasUserInfoUrl,
>;