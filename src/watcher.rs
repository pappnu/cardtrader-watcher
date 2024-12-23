use lettre::{
    message::{Body, MessageBuilder},
    Message,
};
use std::usize;
use tokio::time::{interval, Duration};

use crate::{
    api::{ApiClient, ListMarketplaceProductsOptions, Product},
    config::{AppConfig, Watchable},
    email::EmailHandler,
    utils::cents_to_basic_unit,
};

pub struct WatchProduct<'a> {
    pub lowest: Option<Product>,
    pub name: Option<String>,
    pub watchable: &'a Watchable,
}

pub struct Watcher<'a> {
    interval: u64,
    seller_country_blacklist: &'a Vec<String>,
    watchables: Vec<WatchProduct<'a>>,
    api_client: ApiClient<'a>,
    email_handler: Option<&'a EmailHandler<'a>>,
}

impl<'a> Watcher<'a> {
    pub fn new(config: &'a AppConfig, email_handler: Option<&'a EmailHandler>) -> Self {
        return Self {
            interval: config.interval,
            seller_country_blacklist: &config.seller_country_blacklist,
            watchables: config
                .watchables
                .iter()
                .map(|watchable| WatchProduct {
                    lowest: None,
                    watchable: &watchable,
                    name: None,
                })
                .collect(),
            api_client: ApiClient {
                bearer_token: &config.bearer_token,
            },
            email_handler: email_handler,
        };
    }

    pub async fn watch(&mut self) {
        let mut watch_interval = interval(Duration::from_millis(self.interval));
        loop {
            watch_interval.tick().await;

            let mut api_call_interval = interval(Duration::from_millis(1000));
            for watch_product in &mut self.watchables {
                api_call_interval.tick().await;

                let result = self
                    .api_client
                    .list_marketplace_products(ListMarketplaceProductsOptions {
                        blueprint_id: Some(watch_product.watchable.blueprint_id),
                        language: watch_product.watchable.language.as_ref(),
                        ..Default::default()
                    })
                    .await;

                match result {
                    Ok(mut products) => {
                        let mut filtered_products: Vec<Product> = products
                            .remove(&watch_product.watchable.blueprint_id.to_string())
                            .unwrap_or_default()
                            .into_iter()
                            .filter(|product| {
                                product.price.cents <= watch_product.watchable.price_limit
                                    && (!watch_product.watchable.can_order_via_zero
                                        || product.user.can_sell_via_hub)
                                    && !self
                                        .seller_country_blacklist
                                        .contains(&product.user.country_code)
                            })
                            .collect();
                        filtered_products.sort_by(|a, b| a.price.cents.cmp(&b.price.cents));

                        match filtered_products.into_iter().next() {
                            Some(first) => 'some_arm: {
                                let price = (|| Some(watch_product.lowest.as_ref()?.price.cents))()
                                    .unwrap_or(usize::MAX);

                                if price == first.price.cents {
                                    break 'some_arm;
                                }

                                let increase = price < first.price.cents;

                                match self.email_handler {
                                    Some(handler) => {
                                        let (msg, body) = Self::build_price_change_email(
                                            increase,
                                            watch_product.lowest.as_ref(),
                                            &first,
                                        );
                                        let _ = handler.send_email(msg, body).await;
                                    }
                                    None => (),
                                }
                                println!(
                                    "[{}] {}",
                                    if increase { "+" } else { "-" },
                                    Self::format_product_details(Some(&first)),
                                );
                                watch_product.lowest = Some(first);
                            }
                            None => match &watch_product.lowest {
                                Some(value) => {
                                    println!(
                                        "[0] None - {}",
                                        watch_product.name.as_deref().unwrap_or("None")
                                    );
                                    match self.email_handler {
                                        Some(handler) => {
                                            let (msg, body) =
                                                Self::build_not_available_anymore_email(value);
                                            let _ = handler.send_email(msg, body).await;
                                        }
                                        None => (),
                                    }
                                    watch_product.lowest = None;
                                }
                                None => (),
                            },
                        }
                    }
                    Err(err) => {
                        eprintln!("Failed to fetch marketplace products: {err}");
                    }
                }
            }
        }
    }

    fn format_product_details(product: Option<&Product>) -> String {
        return match product {
            Some(value) => format!(
                "{} {} - {} - {}",
                cents_to_basic_unit(value.price.cents),
                value.price.currency,
                value.user.country_code,
                value.name_en
            ),
            None => "None".to_string(),
        };
    }

    fn build_price_change_email(
        increase: bool,
        previous: Option<&Product>,
        new: &Product,
    ) -> (MessageBuilder, Body) {
        return (
            Message::builder().subject(format!(
                "[{}] {}",
                if increase { "+" } else { "-" },
                Self::format_product_details(Some(new)),
            )),
            Body::new(format!(
                "New:      {}\n\
                 Previous: {}\n\
                 {}",
                Self::format_product_details(Some(new)),
                Self::format_product_details(previous),
                ApiClient::build_card_url(new.blueprint_id),
            )),
        );
    }

    fn build_not_available_anymore_email(previous: &Product) -> (MessageBuilder, Body) {
        return (
            Message::builder().subject(format!("[0] Unavailable - {}", previous.name_en,)),
            Body::new(format!(
                "New:      None\n\
                 Previous: {}\n\
                 {}",
                Self::format_product_details(Some(previous)),
                ApiClient::build_card_url(previous.blueprint_id),
            )),
        );
    }
}
