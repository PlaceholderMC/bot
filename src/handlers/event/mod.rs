use crate::{api, Data, Error};

use analyze_logs::handle_component_interaction;
use log::{debug, info, trace};
use poise::serenity_prelude::{ActivityData, Context, FullEvent, OnlineStatus};
use poise::FrameworkContext;

mod analyze_logs;
mod delete_on_reaction;
mod eta;
mod expand_link;
mod give_role;
mod pluralkit;
mod support_onboard;

pub async fn handle(
	ctx: &Context,
	event: &FullEvent,
	_: FrameworkContext<'_, Data, Error>,
	data: &Data,
) -> Result<(), Error> {
	match event {
		FullEvent::Ready { data_about_bot } => {
			info!("Logged in as {}!", data_about_bot.user.name);

			let latest_minecraft_version =
				api::prism_meta::latest_minecraft_version(&data.http_client).await?;
			let activity = ActivityData::playing(format!("Minecraft {latest_minecraft_version}"));

			info!("Setting presence to activity {activity:#?}");
			ctx.set_presence(Some(activity), OnlineStatus::Online);
		}

		FullEvent::InteractionCreate { interaction } => {
			if let Some(component_interaction) = interaction.as_message_component() {
				give_role::handle(ctx, component_interaction).await?;
				handle_component_interaction(ctx, component_interaction, data).await?;
			}
		}

		FullEvent::Message { new_message } => {
			trace!("Received message {}", new_message.content);

			// ignore new messages from bots
			// note: the webhook_id check allows us to still respond to PK users
			if (new_message.author.bot && new_message.webhook_id.is_none())
				|| (new_message.author == **ctx.cache.current_user())
			{
				trace!("Ignoring message {} from bot", new_message.id);
				return Ok(());
			}

			if let Some(storage) = &data.storage {
				let http = &data.http_client;
				// detect PK users first to make sure we don't respond to unproxied messages
				pluralkit::handle(ctx, http, storage, new_message).await?;

				if storage.is_user_plural(new_message.author.id).await?
					&& pluralkit::is_message_proxied(http, new_message).await?
				{
					debug!("Not replying to unproxied PluralKit message");
					return Ok(());
				}
			}

			eta::handle(ctx, new_message).await?;
			expand_link::handle(ctx, &data.http_client, new_message).await?;
			analyze_logs::handle_message(ctx, new_message, data).await?;
		}

		FullEvent::ReactionAdd { add_reaction } => {
			trace!(
				"Received reaction {} on message {} from {}",
				add_reaction.emoji,
				add_reaction.message_id.to_string(),
				add_reaction.user_id.unwrap_or_default().to_string()
			);
			delete_on_reaction::handle(ctx, add_reaction).await?;
		}

		FullEvent::ThreadCreate { thread } => {
			trace!("Received  thread {}", thread.id);
			support_onboard::handle(ctx, thread).await?;
		}

		_ => {}
	}

	Ok(())
}
