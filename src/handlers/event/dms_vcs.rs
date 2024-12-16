use std::{sync::OnceLock, time::SystemTime};

use eyre::Result;
use log::trace;
use poise::serenity_prelude::{Context, Message};
use regex::Regex;

fn regex() -> &'static Regex {
	static REGEX: OnceLock<Regex> = OnceLock::new();
	REGEX.get_or_init(|| Regex::new(r"(?i)\b(?:dms|dm|vc|vcs|voice call|screenshare)\b").unwrap())
}

const MESSAGE: &str = "Please try to keep all support conversations here, run `/tag dm` to learn more.";

pub async fn handle(ctx: &Context, message: &Message) -> Result<()> {
    if !regex().is_match(&message.content) {
        trace!(
            "The message '{}' (probably) doesn't say DMs, VCs",
            message.content
        );
        return Ok(());
    }

    let response = format!(MESSAGE);
    message.reply(ctx, &response).await?;

    Ok(())
}
