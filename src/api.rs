use std::{collections::HashMap, error::Error};

use reqwest::{header::USER_AGENT, Client, RequestBuilder, Url};
use serde::Deserialize;
use serde_json::from_str;

const PRODUCTS_ENDPOINT: &str = "https://api.cardtrader.com/api/v2/marketplace/products";

#[derive(Deserialize, Debug)]
pub struct Price {
    pub cents: usize,
    pub currency: String,
}

#[derive(Deserialize, Debug)]
pub enum ProductCondition {
    Mint,
    #[serde(rename = "Near Mint")]
    NearMint,
    #[serde(rename = "Slightly Played")]
    SlightlyPlayed,
    #[serde(rename = "Moderately Played")]
    ModeratelyPlayed,
    Played,
    #[serde(rename = "Heavily Played")]
    HeavilyPlayed,
    Poor,
}

#[derive(Deserialize, Debug)]
pub struct Properties {
    #[serde(default)]
    pub condition: Option<ProductCondition>,
    #[serde(default)]
    pub collector_number: Option<String>,
    #[serde(default)]
    pub tournament_legal: Option<bool>,
    #[serde(default)]
    pub signed: Option<bool>,
    #[serde(default)]
    pub mtg_card_colors: Option<String>,
    #[serde(default)]
    pub mtg_foil: Option<bool>,
    #[serde(default)]
    pub mtg_rarity: Option<String>,
    #[serde(default)]
    pub mtg_language: Option<String>,
    #[serde(default)]
    pub altered: Option<bool>,
}

#[derive(Deserialize, Debug)]
pub struct User {
    pub id: usize,
    pub username: String,
    pub can_sell_via_hub: bool,
    pub country_code: String,
    pub user_type: String,
    pub max_sellable_in24h_quantity: Option<usize>,
}

#[derive(Deserialize, Debug)]
pub struct Expansion {
    pub id: usize,
    pub code: String,
    pub name_en: String,
}

#[derive(Deserialize, Debug)]
pub struct Product {
    pub id: usize,
    pub blueprint_id: usize,
    pub name_en: String,
    pub quantity: usize,
    pub price: Price,
    pub description: Option<String>,
    pub properties_hash: Properties,
    pub expansion: Expansion,
    pub user: User,
    pub graded: Option<bool>,
    pub on_vacation: bool,
    pub bundle_size: usize,
}

#[derive(Default)]
pub struct ListMarketplaceProductsOptions<'a> {
    pub expansion_id: Option<usize>,
    pub blueprint_id: Option<usize>,
    pub foil: Option<bool>,
    pub language: Option<&'a String>,
}

pub struct ApiClient<'a> {
    pub bearer_token: &'a String,
}

impl ApiClient<'_> {
    pub async fn list_marketplace_products(
        &self,
        options: ListMarketplaceProductsOptions<'_>,
    ) -> Result<HashMap<String, Vec<Product>>, Box<dyn Error>> {
        let mut url = Url::parse(PRODUCTS_ENDPOINT)?;

        if options.expansion_id.is_none() && options.blueprint_id.is_none() {
            return Err("Either expansion_id or bluperint_id has to be specified".into());
        }

        match options.blueprint_id {
            Some(value) => {
                url.query_pairs_mut()
                    .append_pair("blueprint_id", value.to_string().as_str());
            }
            None => (),
        }
        match options.expansion_id {
            Some(value) => {
                url.query_pairs_mut()
                    .append_pair("expansion_id", value.to_string().as_str());
            }
            None => (),
        }
        match options.foil {
            Some(value) => {
                url.query_pairs_mut()
                    .append_pair("foil", value.to_string().as_str());
            }
            None => (),
        }
        match options.language {
            Some(value) => {
                url.query_pairs_mut()
                    .append_pair("language", value.as_str());
            }
            None => (),
        }

        let response = self.add_base_headers(Client::new().get(url)).send().await?;

        let text = response.text().await?;
        let products: serde_json::Result<HashMap<String, Vec<Product>>> = from_str(text.as_str());

        match products {
            Ok(value) => {
                return Ok(value);
            }
            Err(err) => {
                eprintln!("Couldn't parse marketplace products: {err}");
                return Err(err.into());
            }
        }
    }

    pub fn build_card_url(blueprint_id: usize) -> String {
        return format!("https://www.cardtrader.com/cards/{blueprint_id}");
    }

    fn add_base_headers(&self, builder: RequestBuilder) -> RequestBuilder {
        return builder
            .header(
                USER_AGENT,
                format!("{}-{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION")),
            )
            .header("Authorization", format!("Bearer {}", &self.bearer_token));
    }
}
