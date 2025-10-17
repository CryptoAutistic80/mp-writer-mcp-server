use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct CurrentDatetimeDto {
    pub utc: String,
    pub local: String,
}
