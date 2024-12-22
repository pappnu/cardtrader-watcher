use std::{error::Error, path::Path};

use lettre::message::Mailbox;
use serde::Deserialize;
use serde_json::from_str;
use tokio::fs::read_to_string;

use crate::api::ProductCondition;

#[derive(Deserialize)]
pub struct Watchable {
    pub blueprint_id: usize,
    pub price_limit: usize, // inclusive limit in cents, e.g. 1 € is 100
    #[serde(default)]
    pub language: Option<String>,
    #[serde(default)]
    pub min_condition: Option<ProductCondition>,
    #[serde(default)]
    pub can_order_via_zero: bool,
}

#[derive(Deserialize)]
pub struct EmailConfig {
    pub relay_host: String,
    pub from: Mailbox,
    pub to: Mailbox,
}

#[derive(Deserialize)]
pub struct AppConfig {
    pub bearer_token: String,
    pub interval: u64,
    pub watchables: Vec<Watchable>,
    #[serde(default)]
    pub seller_country_blacklist: Vec<String>,
    pub email: Option<EmailConfig>,
}

pub async fn read_config(path: impl AsRef<Path>) -> Result<AppConfig, Box<dyn Error>> {
    return Ok(from_str((read_to_string(path).await?).as_str())?);
}
