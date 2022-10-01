use crate::{index::Index, prelude::*};
use once_cell::sync::OnceCell;
use std::time::Instant;

#[global_allocator]
static ALLOC: snmalloc_rs::SnMalloc = snmalloc_rs::SnMalloc;

mod command;
mod config;
mod handler;
mod index;
#[cfg(feature = "logging")]
mod log;
mod prelude;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

static UP_SINCE: OnceCell<Instant> = OnceCell::new();

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    #[cfg(feature = "logging")]
    log::init_logging();

    if let Err(err) = BotConfig::load() {
        error!("Failed to load configuration: {err}");
        return;
    }

    if let Err(err) = Index::load().await {
        error!("Failed to parse index: {err}");
        return;
    }

    UP_SINCE.set(Instant::now()).unwrap();

    let mut client = Client::builder(
        &BotConfig::get().token,
        GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT,
    )
    .event_handler(handler::GreedHandler)
    .await
    .expect("failed to create Discord client");

    if let Err(err) = client.start().await {
        error!("Client error: {err}");
    }
}
