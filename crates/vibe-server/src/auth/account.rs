use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "state")]
pub enum AuthRequestResponse {
    #[serde(rename = "requested")]
    Requested,
    #[serde(rename = "authorized")]
    Authorized { token: String, response: String },
}

#[derive(Debug, Clone, Serialize)]
pub struct AuthRequestStatusResponse {
    pub status: &'static str,
    #[serde(rename = "supportsV2")]
    pub supports_v2: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct TokenResponse {
    pub success: bool,
    pub token: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SuccessResponse {
    pub success: bool,
}
