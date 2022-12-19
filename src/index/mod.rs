use crate::{index::meme::GreedMeme, prelude::*};
use chrono::NaiveDateTime;
use once_cell::sync::OnceCell;
use rand::prelude::*;
use serenity::futures::future::join_all;
use std::{collections::HashMap, num::NonZeroU64, path::PathBuf};
use tokio::fs::read_to_string;

pub mod meme;

const INDEX_FILENAME: &str = "index.json";

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub enum ChannelStatus {
    #[default]
    NotIndexed,
    Indexing,
    Indexed,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct ChannelState {
    current_message: Option<NonZeroU64>,
    status: ChannelStatus,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ChannelIndex {
    pub memes: Vec<GreedMeme>,
    pub state: ChannelState,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Index {
    index: HashMap<NonZeroU64, HashMap<NonZeroU64, ChannelIndex>>,
    last_index: Option<NaiveDateTime>,
    #[serde(skip)]
    all_memes: Vec<IndexedMeme>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexedMeme(
    pub(crate) NonZeroU64,
    pub(crate) NonZeroU64,
    pub(crate) GreedMeme,
);

impl IndexedMeme {
    pub fn guild_id(&self) -> NonZeroU64 {
        self.0
    }

    pub fn channel_id(&self) -> NonZeroU64 {
        self.1
    }

    pub fn message_id(&self) -> NonZeroU64 {
        self.2.message_id
    }

    pub fn meme(&self) -> &GreedMeme {
        &self.2
    }
}

pub enum IndexMode {
    Forwards(NonZeroU64),
    Backwards,
}

static INSTANCE: OnceCell<Mutex<Index>> = OnceCell::new();

impl Index {
    fn index_path() -> PathBuf {
        let mut path = BotConfig::get().data_folder.clone();
        path.push(INDEX_FILENAME);
        path
    }

    pub async fn load() -> Result<(), serde_json::Error> {
        debug!("Loading index...");
        let data_folder = BotConfig::get().data_folder.clone();
        match data_folder.try_exists() {
            Ok(exists) => {
                if !exists {
                    debug!("Data folder does not exist, creating it...");
                    std::fs::create_dir_all(data_folder).expect("Failed to create data folder");
                }
            }
            Err(err) => {
                error!("Failed to check data folder: {err}");
            }
        }
        let mut index = match read_to_string(Self::index_path()).await {
            Ok(index) => serde_json::from_str(&index)?,
            Err(_) => Index::default(),
        };
        index.rebuild_memory_index().await;
        INSTANCE.set(Mutex::new(index)).unwrap();
        debug!("Done loading index");
        Ok(())
    }

    pub fn get() -> &'static Mutex<Index> {
        INSTANCE.get().unwrap()
    }

    async fn save_self(&self) -> Result<(), std::io::Error> {
        let index_str = serde_json::to_string(&self)?;
        tokio::fs::write(Self::index_path(), index_str).await?;
        Ok(())
    }

    async fn rebuild_memory_index(&mut self) {
        debug!("Rebuilding memory index...");

        let mut all_memes = Vec::new();

        for (guild, channels) in self.index.iter() {
            for (channel, idx) in channels.iter() {
                all_memes.extend(
                    idx.memes
                        .iter()
                        .map(|meme| IndexedMeme(*guild, *channel, meme.clone())),
                )
            }
        }

        self.all_memes = all_memes;
    }

    pub fn get_last_indexed(&self) -> &Option<NaiveDateTime> {
        &self.last_index
    }

    pub fn get_meme_count(&self) -> usize {
        self.all_memes.len()
    }

    pub async fn get_random_meme(&self) -> Option<&IndexedMeme> {
        #[cfg(debug_assertions)]
        if let Some(ref test_settings) = BotConfig::get().test_settings {
            if let Some(ref force_meme) = test_settings.force_meme {
                return Some(force_meme);
            }
        }

        if self.all_memes.is_empty() {
            return None;
        }

        let mut rng = rand::thread_rng();
        self.all_memes.choose(&mut rng)
    }

    pub async fn refresh(ctx: &Context) -> CommandResult<HashMap<String, usize>> {
        use_index!().last_index = Some(chrono::Utc::now().naive_utc());

        let mut channel_states = use_index!()
            .index
            .iter()
            .flat_map(|(_, channels)| {
                channels
                    .iter()
                    .map(|(channel, idx)| (*channel, idx.state.clone()))
            })
            .collect::<HashMap<_, _>>();
        let mut meme_counts = HashMap::<String, usize>::new();

        // rate limiting makes parallelizing this almost completely useless

        for id in BotConfig::get().channel_ids.iter() {
            let state = channel_states.entry(*id).or_default();
            let channel = match ctx.http.get_channel(ChannelId(*id)).await {
                Ok(channel) => match channel {
                    Channel::Guild(guild) => guild,
                    _ => {
                        error!("Channel {id} is not a guild channel");
                        continue;
                    }
                },
                Err(err) => {
                    error!("Failed to access channel {id}: {err}");
                    continue;
                }
            };

            if channel.kind != ChannelType::Text {
                error!("Channel {id} is not a text channel");
                continue;
            }

            debug!("{id}: index start");

            let count = match state.status {
                ChannelStatus::NotIndexed => {
                    Index::index_channel(ctx, &channel, IndexMode::Backwards).await
                }
                ChannelStatus::Indexing => {
                    warn!("Not indexing {id} because it is already indexing");
                    0
                }
                ChannelStatus::Indexed => {
                    Index::index_channel(
                        ctx,
                        &channel,
                        IndexMode::Forwards(state.current_message.unwrap()),
                    )
                    .await
                }
            };

            *meme_counts.entry(channel.name).or_insert(0) += count;
        }

        use_index!().rebuild_memory_index().await;

        Ok(meme_counts)
    }

    pub async fn index_channel(ctx: &Context, channel: &GuildChannel, mode: IndexMode) -> usize {
        let mut batch = 0;
        let mut newest_message = match mode {
            IndexMode::Backwards => None,
            IndexMode::Forwards(id) => Some(id),
        };
        let mut current_message = newest_message;
        let guild_id = channel.guild_id.0;
        let channel_id = channel.id.0;
        let mut memes_found = 0;

        {
            use_index!(guild_id, channel_id).state.status = ChannelStatus::Indexing;
        }

        loop {
            batch += 1;
            if cfg!(debug_assertions) && batch > 10 {
                // limit the index depth in debug mode, we don't need everything
                break;
            }

            let get_request = match mode {
                IndexMode::Backwards => {
                    if let Some(id) = current_message {
                        GetMessages::new().before(id)
                    } else {
                        GetMessages::new()
                    }
                }
                IndexMode::Forwards(_) => GetMessages::new().after(current_message.unwrap()),
            }
            .limit(100);
            let messages = match channel.id.messages(&ctx.http, get_request).await {
                Ok(messages) => messages,
                Err(err) => {
                    error!("{channel_id}: batch {batch} -> failed to retreive messages: {err}");
                    continue;
                }
            };

            if messages.is_empty() {
                break;
            } else {
                match mode {
                    IndexMode::Backwards => {
                        if newest_message.is_none() {
                            newest_message = Some(messages.first().unwrap().id.0);
                        }
                        current_message = Some(messages.last().unwrap().id.0);
                    }
                    IndexMode::Forwards(_) => {
                        newest_message = Some(messages.first().unwrap().id.0);
                        current_message = newest_message;
                    }
                }
            }

            // avoid collecting this because we're just going to use it to extend another buffer
            let memes = join_all(
                messages
                    .iter()
                    .map(|msg| async { GreedMeme::parse(msg).await }),
            )
            .await
            .into_iter()
            .flatten();

            let count = {
                let mut index = Index::get().lock().await;
                let list = &mut use_index!(index, guild_id, channel_id).memes;
                let pre_size = list.len();
                list.extend(memes);

                list.len() - pre_size
            };

            debug!("{channel_id}: batch {batch} -> {count} memes found");

            memes_found += count;
        }

        {
            let mut index = Index::get().lock().await;
            let channel_state = &mut use_index!(index, guild_id, channel_id).state;
            channel_state.current_message = newest_message;
            channel_state.status = ChannelStatus::Indexed;
            index.save_self().await.expect("failed to save index");
        }

        debug!("{channel_id}: finished with {memes_found} new memes");

        memes_found
    }
}

macro_rules! use_index {
    () => {
        Index::get().lock().await
    };
    ($guild_id:expr, $channel_id:expr) => {
        Index::get()
            .lock()
            .await
            .index
            .entry($guild_id)
            .or_default()
            .entry($channel_id)
            .or_default()
    };
    ($index_instance:expr, $guild_id:expr, $channel_id:expr) => {
        $index_instance
            .index
            .entry($guild_id)
            .or_default()
            .entry($channel_id)
            .or_default()
    };
}

pub(crate) use use_index;
