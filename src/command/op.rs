use super::cmd_err;
use crate::prelude::*;
use std::str::FromStr;

pub struct EncodedOP {
    guild_id: u64,
    channel_id: u64,
    message_id: u64,
}

impl EncodedOP {
    pub fn message_link(&self) -> String {
        format!(
            "https://discord.com/channels/{}/{}/{}",
            self.guild_id, self.channel_id, self.message_id
        )
    }

    pub fn channel_link(&self) -> String {
        format!(
            "https://discord.com/channels/{}/{}",
            self.guild_id, self.channel_id
        )
    }
}

impl FromStr for EncodedOP {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut split = s.split('_');
        split.next(); // consume `op`
        let guild_id = split.next().and_then(|v| v.parse::<u64>().ok());
        let channel_id = split.next().and_then(|v| v.parse::<u64>().ok());
        let message_id = split.next().and_then(|v| v.parse::<u64>().ok());
        match (guild_id, channel_id, message_id) {
            (Some(guild_id), Some(channel_id), Some(message_id)) => Ok(Self {
                guild_id,
                channel_id,
                message_id,
            }),
            _ => {
                warn!("Failed to parse OP interaction: {s}");
                Err(())
            }
        }
    }
}

pub async fn command(ctx: &Context, interactor: &Interactor, op: EncodedOP) -> CommandResult {
    const ERR_MSG: &str = "Failed to retreive OP information";

    let EncodedOP {
        channel_id,
        message_id,
        ..
    } = op;

    let channel_id = ChannelId(match channel_id.try_into() {
        Ok(id) => id,
        Err(err) => {
            // this route will only be taken if the segment was a valid u64, but not a NonZeroU64
            return cmd_err!(
                interactor,
                &ctx.http,
                ERR_MSG,
                "Encoded channel id was zero: {err}"
            );
        }
    });

    let channel = match ctx.http.get_channel(channel_id).await {
        Ok(channel) => match channel {
            Channel::Guild(channel) => channel,
            _ => {
                return cmd_err!(
                    interactor,
                    &ctx.http,
                    ERR_MSG,
                    "Attempted to retreive OP information from a non guild channel"
                );
            }
        },
        Err(err) => {
            return cmd_err!(
                interactor,
                &ctx.http,
                ERR_MSG,
                "Failed to get channel for OP information: {err}"
            );
        }
    };

    let message = match channel.message(&ctx.http, message_id).await {
        Ok(message) => message,
        Err(err) => {
            return cmd_err!(
                interactor,
                &ctx.http,
                ERR_MSG,
                "Failed to get message for OP information: {err}"
            );
        }
    };

    interactor
        .create_interaction_response(
            &ctx.http,
            CreateInteractionResponse::new().interaction_response_data(
                CreateInteractionResponseData::new()
                    .embed(
                        CreateEmbed::new()
                            .colour(Colour::from_rgb(149, 54, 161))
                            .author(
                                CreateEmbedAuthor::new(message.author.tag())
                                    .icon_url(message.author.face()),
                            )
                            .timestamp(message.timestamp)
                            .footer(CreateEmbedFooter::new(channel.name())),
                    )
                    .components(
                        CreateComponents::new().add_action_row(
                            CreateActionRow::new()
                                .add_button(
                                    CreateButton::new_link(op.message_link()).label("Message"),
                                )
                                .add_button(
                                    CreateButton::new_link(op.channel_link()).label("Channel"),
                                ),
                        ),
                    ),
            ),
        )
        .await
}
