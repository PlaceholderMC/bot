use log::trace;
use reqwest::Response;

pub mod dadjoke;
pub mod github;
pub mod mclogs;
pub mod paste_gg;
pub mod pluralkit;
pub mod prism_meta;
pub mod rory;

pub type HttpClient = reqwest::Client;

pub trait HttpClientExt {
	// sadly i can't implement the actual Default trait :/
	fn default() -> Self;
	async fn get_request(&self, url: &str) -> Result<Response, reqwest::Error>;
}

impl HttpClientExt for HttpClient {
	fn default() -> Self {
		let version = option_env!("CARGO_PKG_VERSION").unwrap_or("development");
		let user_agent = format!("refraction/{version}");
		reqwest::ClientBuilder::new()
			.user_agent(user_agent)
			.build()
			.unwrap_or_default()
	}

	async fn get_request(&self, url: &str) -> Result<Response, reqwest::Error> {
		trace!("Making request to {url}");
		let resp = self.get(url).send().await?;
		resp.error_for_status_ref()?;

		Ok(resp)
	}
}
