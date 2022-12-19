use serenity::gateway::ActivityData;

use crate::{command::interactor::Interactor, prelude::*};
// use serenity::gateway::ActivityData;

pub struct GreedHandler;

#[async_trait]
impl EventHandler for GreedHandler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(cmd) = interaction {
            let cmd_str = cmd.data.name.clone();
            let interactor = Interactor::Command(Box::new(cmd));
            crate::command::handle_command(&cmd_str, &ctx, &interactor).await;
        } else if let Interaction::Component(cmd) = interaction {
            let cmd_str = cmd.data.custom_id.clone();
            let interactor = Interactor::Message(Box::new(cmd));
            crate::command::handle_command(&cmd_str, &ctx, &interactor).await;
        } else {
            error!("Received an unsupported interaction: {interaction:#?}");
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("{} connected. Initializing...", ready.user.name);

        if let Some(true) = BotConfig::get().test_settings.delete_all_commands {
            let commands = ctx.http.get_global_application_commands().await;

            match commands {
                Ok(commands) => {
                    for command in commands {
                        debug!("deleting global application command: {}", command.name);
                        if let Err(err) =
                            ctx.http.delete_global_application_command(command.id).await
                        {
                            error!("could not delete global application command: {err}");
                        }
                    }
                }
                Err(err) => {
                    error!("could not get global application commands for deletion: {err}");
                }
            }
        }

        crate::command::greed::register(&ctx)
            .await
            .unwrap_or_else(|_| {
                panic!(
                    "failed to register {} command",
                    crate::command::greed::COMMAND_NAME,
                )
            });
        crate::command::memedex::register(&ctx)
            .await
            .unwrap_or_else(|_| {
                panic!(
                    "failed to register {} command",
                    crate::command::memedex::COMMAND_NAME,
                )
            });
        crate::command::info::register(&ctx)
            .await
            .unwrap_or_else(|_| {
                panic!(
                    "failed to register {} command",
                    crate::command::info::COMMAND_NAME,
                )
            });

        // i have no idea why or how this isn't async
        ctx.set_activity(Some(ActivityData::watching("you be greedy")));

        info!("Initialized and ready for interactions.");
    }
}
