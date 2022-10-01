use crate::prelude::*;
use std::str::FromStr;

pub mod greed;
pub mod info;
pub mod interactor;
pub mod memedex;
pub mod op;

pub type CommandResult<T = ()> = Result<T, serenity::Error>;

/// `cmd_err!(interactor, http, pub_msg, internal_msg)`
macro_rules! cmd_err {
    ($interactor:expr, $http:expr, $pub_msg:expr, $internal_msg:expr) => {{
        use serenity::builder::EditInteractionResponse;
        error!($internal_msg);
        $interactor
            .edit_original_interaction_response(
                $http,
                EditInteractionResponse::new().content($pub_msg),
            )
            .await
            .map(|_| ())
    }};
}

pub(crate) use cmd_err;

use self::op::EncodedOP;

async fn unknown_command(cmd: &str, ctx: &Context, interactor: &Interactor) -> CommandResult {
    cmd_err!(
        interactor,
        &ctx.http,
        "Unknown command. Not sure how we got here...",
        "Received an unknown command from interaction: {cmd}"
    )
}

pub async fn handle_command(cmd: &str, ctx: &Context, interactor: &Interactor) {
    if let Err(err) = match cmd {
        greed::COMMAND_NAME => greed::command(ctx, interactor).await,
        memedex::COMMAND_NAME => memedex::command(ctx, interactor).await,
        info::COMMAND_NAME => info::command(ctx, interactor).await,
        op if op.starts_with("op_") => match EncodedOP::from_str(op) {
            Ok(op) => op::command(ctx, interactor, op).await,
            _ => unknown_command(cmd, ctx, interactor).await,
        },
        _ => unknown_command(cmd, ctx, interactor).await,
    } {
        error!("Error handling interaction: {err}")
    }
}
