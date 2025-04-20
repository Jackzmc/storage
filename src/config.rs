use rocket::serde::{Serialize,Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    general: GeneralConfig,
    auth: AuthConfig,
    smtp: EmailConfig
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GeneralConfig {
    pub listen_ip: Option<String>,
    pub listen_port: Option<u32>
}
#[derive(Debug, Serialize, Deserialize)]
pub struct AuthConfig {
    pub disable_registration: bool,
    pub openid_enabled: Option<bool>,
    pub openid_issuer_url: Option<String>,
    pub openid_client_id: Option<String>,
    pub openid_client_secret: Option<String>

}
#[derive(Debug, Serialize, Deserialize)]
pub struct EmailConfig {

}