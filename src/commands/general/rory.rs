use crate::api::rory::get_rory;
use crate::Context;

use color_eyre::eyre::Result;
use poise::serenity_prelude::{CreateEmbed, CreateEmbedFooter};
use poise::CreateReply;

/// Gets a Rory photo!
#[poise::command(slash_command, prefix_command)]
pub async fn rory(
	ctx: Context<'_>,
	#[description = "specify a Rory ID"] id: Option<u64>,
) -> Result<()> {
	let rory = get_rory(id).await?;

	let embed = {
		let embed = CreateEmbed::new();
		if let Some(error) = rory.error {
			embed.title("Error!").description(error)
		} else {
			let footer = CreateEmbedFooter::new(format!("ID {}", rory.id));
			embed
				.title("Rory :3")
				.url(&rory.url)
				.image(rory.url)
				.footer(footer)
		}
	};

	let reply = CreateReply::default().embed(embed);
	ctx.send(reply).await?;

	Ok(())
}
