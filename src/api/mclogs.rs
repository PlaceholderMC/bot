use std::collections::HashMap;

use eyre::Result;
use serde::{Deserialize, Serialize};

use super::{HttpClient, HttpClientExt};

const MCLOGS: &str = "https://api.mclo.gs/1";
const UPLOAD: &str = "/log";
const RAW: &str = "/raw";

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PostLogResponse {
	pub success: bool,
	pub id: Option<String>,
	pub url: Option<String>,
	pub raw: Option<String>,
	pub error: Option<String>,
}

pub async fn upload_log(http: &HttpClient, content: &str) -> Result<PostLogResponse> {
	let url = format!("{MCLOGS}{UPLOAD}");
	let request = http
		.post(url)
		.form(&HashMap::from([("content", content)]))
		.build()?;

	Ok(http.execute(request).await?.json().await?)
}

pub async fn raw_log(http: &HttpClient, id: &str) -> Result<String> {
	let url = format!("{MCLOGS}{RAW}/{id}");

	Ok(http.get_request(&url).await?.text().await?)
}
