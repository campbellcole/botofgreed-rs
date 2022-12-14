use crate::prelude::*;
use serenity::{
    http::Http,
    model::prelude::interaction::{
        application_command::ApplicationCommandInteraction,
        message_component::MessageComponentInteraction,
    },
};
use std::result::Result as StdResult;

type Result<T> = StdResult<T, serenity::Error>;

pub enum Interactor {
    Command(Box<ApplicationCommandInteraction>),
    Message(Box<MessageComponentInteraction>),
}

impl Interactor {
    pub async fn create_interaction_response<'a>(
        &self,
        http: impl AsRef<Http>,
        builder: CreateInteractionResponse<'a>,
    ) -> Result<()> {
        match self {
            Self::Command(aci) => aci.create_interaction_response(http, builder).await,
            Self::Message(mci) => mci.create_interaction_response(http, builder).await,
        }
    }

    pub async fn edit_original_interaction_response<'a>(
        &self,
        http: impl AsRef<Http>,
        builder: EditInteractionResponse<'a>,
    ) -> Result<Message> {
        match self {
            Self::Command(aci) => aci.edit_original_interaction_response(http, builder).await,
            Self::Message(mci) => mci.edit_original_interaction_response(http, builder).await,
        }
    }

    pub fn user(&self) -> &User {
        match self {
            Self::Command(aci) => &aci.user,
            Self::Message(mci) => &mci.user,
        }
    }
}
