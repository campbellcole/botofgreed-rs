use crate::{index::Index, prelude::*};
use std::fmt::Write;

#[cfg(debug_assertions)]
pub const COMMAND_NAME: &str = "nightly_memedex";

#[cfg(not(debug_assertions))]
pub const COMMAND_NAME: &str = "memedex";

pub async fn register(ctx: &Context) -> CommandResult {
    Command::create_global_application_command(
        &ctx.http,
        CreateCommand::new(COMMAND_NAME).description("Refresh the meme database"),
    )
    .await
    .map(|_| debug!("registered `{COMMAND_NAME}` command globally"))
}

pub async fn command(ctx: &Context, interactor: &Interactor) -> CommandResult {
    interactor
        .create_response(
            &ctx.http,
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new().content("Refreshing index..."),
            ),
        )
        .await?;

    let meme_counts = Index::refresh(ctx).await?;

    let mut status_str = "Index updated:".to_owned();
    for (channel, count) in meme_counts.iter() {
        let channel = channel
            .replace('_', "\\_")
            .replace('*', "\\*")
            .replace('`', "\\`")
            .replace('~', "\\~");

        write!(
            status_str,
            "\n{channel}: {count} new meme{}",
            if *count != 1 { "s" } else { "" }
        )
        .unwrap();
    }

    interactor
        .edit_response(
            &ctx.http,
            EditInteractionResponse::new().content(status_str),
        )
        .await
        .map(|_| ())
}
