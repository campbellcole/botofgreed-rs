use crate::{
    command::cmd_err,
    index::{use_index, Index},
    prelude::*,
};

#[cfg(debug_assertions)]
pub const COMMAND_NAME: &str = "nightly_greed";

#[cfg(not(debug_assertions))]
pub const COMMAND_NAME: &str = "greed";

pub async fn register(ctx: &Context) -> CommandResult {
    Command::create_global_application_command(
        &ctx.http,
        CreateCommand::new(COMMAND_NAME).description("Retreive a random meme"),
    )
    .await
    .map(|_| debug!("registered `{COMMAND_NAME}` command globally"))
}

pub async fn command(ctx: &Context, interactor: &Interactor) -> CommandResult {
    let meme = match use_index!().get_random_meme().await {
        Some(meme) => meme.clone(),
        None => {
            return cmd_err!(
                interactor,
                &ctx.http,
                format!(
                    "Failed to retreive a meme (have you ran `/{}` yet?)",
                    crate::command::memedex::COMMAND_NAME
                ),
                "Failed to retreive a meme"
            )
        }
    };

    let msg = ctx
        .http
        .get_message(meme.channel_id().into(), meme.message_id().get().into())
        .await;

    if let Ok(msg) = msg {
        let mut emoji_reactions =
            msg.reactions
                .iter()
                .filter_map(|reaction| match reaction.reaction_type {
                    ReactionType::Unicode(ref unicode) => Some(unicode),
                    _ => None,
                });

        let unused_emoji = emojis::iter()
            .find(|emoji| !emoji_reactions.any(|reaction| reaction == emoji.as_str()));

        if let Some(emoji) = unused_emoji {
            if let Err(err) = ctx
                .http
                .create_reaction(
                    meme.channel_id().into(),
                    meme.message_id().into(),
                    &ReactionType::Unicode(emoji.to_string()),
                )
                .await
            {
                error!("failed to react to original message: {err}");
            }
        } else {
            warn!(
                "failed to find an unused emoji to react to message with: {}/{}",
                meme.channel_id(),
                meme.message_id()
            );
        }
    } else {
        error!("could not get original message to react to, just sending the meme");
    };

    interactor
        .create_response(
            &ctx.http,
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .content(format!(
                        "Triggered by <@{}>\n{}",
                        interactor.user().id.0,
                        &meme.meme().url,
                    ))
                    .components(vec![CreateActionRow::Buttons(vec![
                        CreateButton::new(COMMAND_NAME)
                            .style(ButtonStyle::Primary)
                            .emoji('\u{267B}')
                            .label("I'm feeling Greedy"),
                        CreateButton::new(format!(
                            "op_{}_{}_{}",
                            meme.guild_id(),
                            meme.channel_id(),
                            meme.message_id(),
                        ))
                        .style(ButtonStyle::Primary)
                        .emoji('\u{2049}')
                        .label("sauce??"),
                    ])])
                    .allowed_mentions(CreateAllowedMentions::new().all_users(true)),
            ),
        )
        .await
}
