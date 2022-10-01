use crate::{
    command::cmd_err,
    index::{use_index, Index},
    prelude::*,
};
use serenity::model::prelude::component::ButtonStyle;

#[cfg(debug_assertions)]
pub const COMMAND_NAME: &str = "nightly_greed";

#[cfg(not(debug_assertions))]
pub const COMMAND_NAME: &str = "greed";

pub async fn register(ctx: &Context) -> CommandResult {
    command::Command::create_global_application_command(
        &ctx.http,
        CreateApplicationCommand::new(COMMAND_NAME).description("Retreive a random meme"),
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
            );
        }
    };

    interactor
        .create_interaction_response(
            &ctx.http,
            CreateInteractionResponse::new().interaction_response_data(
                CreateInteractionResponseData::new()
                    .content(format!(
                        "Triggered by <@{}>\n{}",
                        interactor.user().id.0,
                        &meme.meme().url,
                    ))
                    .components(
                        CreateComponents::new().add_action_row(
                            CreateActionRow::new()
                                .add_button(
                                    CreateButton::new(ButtonStyle::Primary, COMMAND_NAME)
                                        .emoji('\u{267B}')
                                        .label("I'm feeling Greedy"),
                                )
                                .add_button(
                                    CreateButton::new(
                                        ButtonStyle::Primary,
                                        format!(
                                            "op_{}_{}_{}",
                                            meme.guild_id(),
                                            meme.channel_id(),
                                            meme.message_id(),
                                        ),
                                    )
                                    .emoji('\u{2049}')
                                    .label("sauce??"),
                                ),
                        ),
                    )
                    .allowed_mentions(CreateAllowedMentions::new().all_users(true)),
            ),
        )
        .await
}
