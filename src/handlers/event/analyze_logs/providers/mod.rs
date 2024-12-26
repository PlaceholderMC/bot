use crate::api::HttpClient;

use std::slice::Iter;

use enum_dispatch::enum_dispatch;
use eyre::Result;
use poise::serenity_prelude::Message;
use regex::Regex;

use self::{
	_0x0::_0x0 as _0x0st, haste::Haste, mclogs::MCLogs, paste_gg::PasteGG, pastebin::PasteBin,
};

#[path = "0x0.rs"]
mod _0x0;
mod haste;
mod mclogs;
mod paste_gg;
mod pastebin;

#[enum_dispatch]
pub trait LogProvider {
	async fn find_match(&self, message: &Message) -> Option<String>;
	async fn fetch(&self, http: &HttpClient, content: &str) -> Result<String>;
}

fn get_first_capture(regex: &Regex, string: &str) -> Option<String> {
	regex
		.captures_iter(string)
		.find_map(|c| c.get(1).map(|c| c.as_str().to_string()))
}

#[enum_dispatch(LogProvider)]
enum Provider {
	_0x0st,
	Haste,
	MCLogs,
	PasteGG,
	PasteBin,
}

impl Provider {
	pub fn iterator() -> Iter<'static, Provider> {
		static PROVIDERS: [Provider; 5] = [
			Provider::_0x0st(_0x0st),
			Provider::Haste(Haste),
			Provider::MCLogs(MCLogs),
			Provider::PasteBin(PasteBin),
			Provider::PasteGG(PasteGG),
		];
		PROVIDERS.iter()
	}
}

pub async fn find_log(http: &HttpClient, message: &Message) -> Result<Option<String>> {
	let providers = Provider::iterator();

	for provider in providers {
		if let Some(found) = provider.find_match(message).await {
			let log = provider.fetch(http, &found).await?;
			return Ok(Some(log));
		}
	}

	Ok(None)
}
