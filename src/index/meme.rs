use crate::prelude::*;
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::num::NonZeroU64;

#[derive(Debug, Clone, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum MemeType {
    DiscordAttachment = 0,
    FileURL = 1,
    SupportedEmbed = 2,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GreedMeme {
    pub message_id: NonZeroU64,
    pub url: String,
    pub meme_type: MemeType,
}

impl GreedMeme {
    fn check_extension(filename: &str) -> bool {
        let ext = match std::path::Path::new(filename).extension() {
            Some(ext) => match ext.to_str() {
                Some(ext) => ext.to_lowercase(),
                None => {
                    warn!("filename has extension but cannot be converted to string: {filename}");
                    return false;
                }
            },
            None => {
                warn!("Found an attachment without a file extension: {filename}");
                return false;
            }
        };

        matches!(
            ext.as_str(),
            "png"
                | "jpg"
                | "jpeg"
                | "gif"
                | "tiff"
                | "heic"
                | "heif"
                | "bmp"
                | "raw"
                | "webp"
                | "svg"
                | "mkv"
                | "mp4"
                | "avi"
                | "webm"
                | "mpeg"
                | "mp4v"
                | "mpeg4"
                | "mov"
                | "movie"
                | "gifv"
                | "h264"
        )
    }

    pub async fn from_attachments(msg: &Message) -> Option<Self> {
        let attachment = &msg.attachments[0];
        let url = &attachment.url;
        if match &attachment.content_type {
            Some(content_type) => {
                content_type.starts_with("image") || content_type.starts_with("video")
            }
            None => Self::check_extension(&attachment.filename),
        } {
            Some(GreedMeme {
                message_id: msg.id.0,
                url: url.clone(),
                meme_type: MemeType::DiscordAttachment,
            })
        } else {
            None
        }
    }

    pub async fn parse(msg: &Message) -> Option<GreedMeme> {
        if msg.author.bot || msg.kind != MessageType::Regular {
            return None;
        }

        if !msg.attachments.is_empty() {
            GreedMeme::from_attachments(msg).await
        // } else if !msg.embeds.is_empty() {
        //     trace!("skipping embed parsing, not yet implemented");
        //     None
        } else {
            None
        }
    }
}
