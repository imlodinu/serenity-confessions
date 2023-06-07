use std::borrow::Cow;

use crate::Data;
use anyhow::{anyhow, Result};
use poise::serenity_prelude as serenity;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

pub fn get_serenity<'a>(ctx: &Context<'a>) -> &'a serenity::Context {
    ctx.serenity_context()
}