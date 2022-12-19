use crate::{
    index::{use_index, Index},
    prelude::*,
    UP_SINCE, VERSION,
};
use std::time::Instant;

#[cfg(debug_assertions)]
pub const COMMAND_NAME: &str = "nightly_info";

#[cfg(not(debug_assertions))]
pub const COMMAND_NAME: &str = "info";

pub async fn register(ctx: &Context) -> CommandResult {
    Command::create_global_application_command(
        &ctx.http,
        CreateCommand::new(COMMAND_NAME).description("Print information about Bot of Greed"),
    )
    .await
    .map(|_| debug!("registered `{COMMAND_NAME}` command globally"))
}

pub async fn command(ctx: &Context, interactor: &Interactor) -> CommandResult {
    let (last_indexed, meme_count) = {
        let idx = use_index!();
        let last_indexed = idx
            .get_last_indexed()
            .map(|dt| dt.format("%m-%d-%Y %H:%M:%S").to_string())
            .unwrap_or_else(|| "never".into());
        let meme_count = idx.get_meme_count();
        (last_indexed, meme_count)
    };

    let uptime = Instant::now().checked_duration_since(*UP_SINCE.get().unwrap());

    let uptime = uptime
        .map(|duration| humantime::format_duration(duration).to_string())
        .unwrap_or_else(|| "unknown".into());

    let info = format!("```\nVersion: {VERSION}\nLast indexed at: {last_indexed}\nMemes indexed: {meme_count}\nUptime: {uptime}\n```");

    interactor
        .create_response(
            &ctx.http,
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new().content(info),
            ),
        )
        .await
}
