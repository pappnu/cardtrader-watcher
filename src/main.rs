use config::read_config;
use email::EmailHandler;
use watcher::Watcher;

mod api;
mod config;
mod email;
mod utils;
mod watcher;

#[tokio::main]
async fn main() {
    let config = read_config("./config.json")
        .await
        .expect("Failed to read config");
    let email_handler = match &config.email {
        Some(value) => Some(EmailHandler::new(&value)),
        None => None,
    };
    let mut watcher = Watcher::new(&config, email_handler.as_ref());
    watcher.watch().await;
}
