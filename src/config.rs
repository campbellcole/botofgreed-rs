use once_cell::sync::OnceCell;
use std::{num::NonZeroU64, path::PathBuf};

#[derive(Debug)]
pub struct BotConfig {
    pub token: String,
    pub data_folder: PathBuf,
    pub channel_ids: Vec<NonZeroU64>,
}

static INSTANCE: OnceCell<BotConfig> = OnceCell::new();

impl BotConfig {
    pub fn load() -> Result<(), std::env::VarError> {
        use std::env::var;
        let token = var("DISCORD_TOKEN")?;
        let data_folder = PathBuf::from(var("DATA_FOLDER")?);
        let channel_ids = var("CHANNEL_IDS")?
            .split(',')
            .map(|id| {
                id.parse::<NonZeroU64>()
                    .expect("all channel IDs must be non-zero u64")
            })
            .collect();
        let config = BotConfig {
            token,
            data_folder,
            channel_ids,
        };
        INSTANCE
            .set(config)
            .expect("attempted to load BotConfig twice");
        Ok(())
    }

    pub fn get() -> &'static BotConfig {
        INSTANCE
            .get()
            .expect("attempted to get BotConfig before load")
    }
}
