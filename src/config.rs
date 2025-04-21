use std::collections::HashMap;
use std::env::var;
use figment::Figment;
use figment::providers::{Env, Format, Toml};
use log::{debug, error};
use openidconnect::core::{CoreClient, CoreProviderMetadata};
use openidconnect::IssuerUrl;
use openidconnect::url::Url;
use rocket::serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct AppConfig {
    pub general: GeneralConfig,
    pub auth: AuthConfig,
    pub smtp: Option<EmailConfig>
}

pub fn get_settings() -> AppConfig {
    let f = Figment::new()
        .merge(Toml::file("config.toml"))
        .merge(Env::prefixed("STORAGE_")
            .map(|f| f.as_str().replace("__", "-").into()))
        .extract();
    match f {
        Ok(settings) => settings,
        Err(e) => {
            error!("Failed to read configuration");
            error!("{}", e);
            std::process::exit(1);
        }
    }
}




#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct GeneralConfig {
    pub listen_ip: Option<String>,
    pub listen_port: Option<u16>,
    pub public_url: String,
    pub database_url: Option<String>,
}
impl GeneralConfig {
    pub fn get_public_url(&self) -> Url {
        self.public_url.parse().expect("failed to parse general.public-url")
    }
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct AuthConfig {
    pub disable_registration: bool,
    pub oidc: Option<OidcConfig>,
}

impl AuthConfig {
    pub fn oidc_enabled(&self) -> bool {
        self.oidc.as_ref().map(|o| o.enabled).unwrap_or(false)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct OidcConfig {
    #[serde(default)]
    pub enabled: bool,
    pub issuer_url: String,
    pub client_id: String,
    pub client_secret: String,
    #[serde(default)]
    pub claims: Vec<String>,
    #[serde(default)]
    pub create_account: bool,
    #[serde(default)]
    pub disable_normal_login: bool
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SmtpEncryption {
    None,
    StartTls,
    Tls
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct EmailConfig {
    #[serde(default)]
    pub enabled: bool,
    pub hostname: String,
    pub  port: u16,
    pub username: String,
    pub password: String,
    pub tls: Option<SmtpEncryption>,
    pub from_name: Option<String>,
    pub from_email: Option<String>,
}
