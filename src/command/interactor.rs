use crate::prelude::*;
use serenity::http::Http;
use std::result::Result as StdResult;

type Result<T> = StdResult<T, serenity::Error>;

pub enum Interactor {
    Command(Box<CommandInteraction>),
    Message(Box<ComponentInteraction>),
}

impl Interactor {
    pub async fn create_response(
        &self,
        http: impl AsRef<Http>,
        builder: CreateInteractionResponse,
    ) -> Result<()> {
        match self {
            Self::Command(aci) => aci.create_response(http, builder).await,
            Self::Message(mci) => mci.create_response(http, builder).await,
        }
    }

    pub async fn edit_response(
        &self,
        http: impl AsRef<Http>,
        builder: EditInteractionResponse,
    ) -> Result<Message> {
        match self {
            Self::Command(aci) => aci.edit_response(http, builder).await,
            Self::Message(mci) => mci.edit_response(http, builder).await,
        }
    }

    pub fn user(&self) -> &User {
        match self {
            Self::Command(aci) => &aci.user,
            Self::Message(mci) => &mci.user,
        }
    }
}
