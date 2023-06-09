use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum ButtonCustomId {
    ApproveConfession(crate::commands::confessions::ConfessionInfo),
    DenyConfession(crate::commands::confessions::ConfessionInfo),
}