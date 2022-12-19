use once_cell::sync::OnceCell;
use std::{num::NonZeroU64, path::PathBuf};

use crate::index::{
    meme::{GreedMeme, MemeType},
    IndexedMeme,
};

use std::env::var;

#[derive(Debug, Default)]
pub struct TestSettings {
    pub force_meme: Option<IndexedMeme>,
    pub delete_all_commands: Option<bool>,
}

impl TestSettings {
    #[inline(always)]
    fn missing_segment(seg: &'static str) -> String {
        format!("missing {seg} segment of TEST_FORCE_MEME=<guild_id>,<channel_id>,<message_id>,<attachment_url>")
    }

    pub fn from_env() -> Self {
        let mut test_settings = Self::default();

        if let Ok(force_meme) = var("TEST_FORCE_MEME") {
            let mut split = force_meme.split(',');
            let guild_id = split
                .next()
                .unwrap_or_else(|| panic!("{}", Self::missing_segment("guild_id")))
                .parse::<NonZeroU64>()
                .expect("guild_id segment is not a valid NonZeroU64");
            let channel_id = split
                .next()
                .unwrap_or_else(|| panic!("{}", Self::missing_segment("channel_id")))
                .parse::<NonZeroU64>()
                .expect("channel_id segment is not a valid NonZeroU64");
            let message_id = split
                .next()
                .unwrap_or_else(|| panic!("{}", Self::missing_segment("message_id")))
                .parse::<NonZeroU64>()
                .expect("message_id segment is not a valid NonZeroU64");
            let attachment_url = split
                .next()
                .unwrap_or_else(|| panic!("{}", Self::missing_segment("attachment_url")))
                .to_owned();

            let meme = IndexedMeme(
                guild_id,
                channel_id,
                GreedMeme {
                    meme_type: MemeType::DiscordAttachment,
                    message_id,
                    url: attachment_url,
                },
            );

            test_settings.force_meme = Some(meme);
        }

        if let Ok(force_delete_all_commands) = var("TEST_DELETE_ALL_COMMANDS") {
            test_settings.delete_all_commands = Some(matches!(
                force_delete_all_commands.to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "y"
            ));
        }

        test_settings
    }
}

#[derive(Debug)]
pub struct BotConfig {
    pub token: String,
    pub data_folder: PathBuf,
    pub channel_ids: Vec<NonZeroU64>,
    pub test_settings: TestSettings,
}

static INSTANCE: OnceCell<BotConfig> = OnceCell::new();

impl BotConfig {
    pub fn load() -> Result<(), std::env::VarError> {
        let token = var("DISCORD_TOKEN")?;
        let data_folder = PathBuf::from(var("DATA_FOLDER")?);
        let channel_ids = var("CHANNEL_IDS")?
            .split(',')
            .map(|id| {
                id.parse::<NonZeroU64>()
                    .expect("all channel IDs must be non-zero u64")
            })
            .collect();

        let test_settings = TestSettings::from_env();

        let config = BotConfig {
            token,
            data_folder,
            channel_ids,
            test_settings,
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
