use poise::serenity_prelude as serenity;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum ConfessionButton {
    ApproveConfession((serenity::UserId, serenity::ChannelId)),
    DenyConfession,
    None,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ConfessionRevealButton {
    RevealConfession(String),
    KeepConfession,
    None,
}

macro_rules! impl_button {
    ($($t:ty)+) => ($(
        impl $t {
            pub fn from_string(s: &String) -> Option<Self> {
                serde_json::from_str(&s).ok()
            }
            pub fn to_string(&self) -> String {
                serde_json::to_string(self).unwrap_or("unknown".to_owned())
            }
        }

        impl Default for $t {
            fn default() -> Self {
                Self::None
            }
        }

        impl std::fmt::Display for $t {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.to_string())
            }
        }
    )+)
}

impl_button!(ConfessionButton);
impl_button!(ConfessionRevealButton);
