use poise::serenity_prelude as serenity;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum ButtonCustomId {
    ApproveConfession((serenity::UserId, serenity::ChannelId)),
    DenyConfession(),
}

impl ButtonCustomId {
    pub fn from_string(s: &String) -> Option<Self> {
        serde_json::from_str(&s).ok()
    }

    pub fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap_or("unknown".to_owned())
    }
}
