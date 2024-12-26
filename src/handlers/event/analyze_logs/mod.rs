use crate::{
	api::{mclogs, HttpClientExt},
	consts::Colors,
	utils::first_text_attachment,
	Data,
};

use eyre::{eyre, OptionExt, Result};
use info::Info;
use log::{debug, trace};
use poise::serenity_prelude::{
	ButtonStyle, ComponentInteraction, Context, CreateAllowedMentions, CreateButton, CreateEmbed,
	CreateInteractionResponse, CreateInteractionResponseMessage, CreateMessage, EditMessage,
	Message, MessageType,
};

mod info;
mod issues;
mod providers;

use providers::find_log;

const BUTTON_UPLOAD_YES: &str = "log-upload-yes";
const BUTTON_UPLOAD_NO: &str = "log-upload-no";

pub async fn handle_message(ctx: &Context, message: &Message, data: &Data) -> Result<()> {
	trace!(
		"Checking message {} from {} for logs",
		message.id,
		message.author.id
	);
	let channel = message.channel_id;

	let Ok(log) = find_log(&data.http_client, message).await else {
		let embed = CreateEmbed::new()
			.title("Analysis failed!")
			.description("Couldn't download log");
		let allowed_mentions = CreateAllowedMentions::new().replied_user(true);
		let our_message = CreateMessage::new()
			.reference_message(message)
			.allowed_mentions(allowed_mentions)
			.embed(embed);

		channel.send_message(ctx, our_message).await?;

		return Ok(());
	};

	let attachment = first_text_attachment(message);

	let log = if let Some(log) = log {
		log
	} else if let Some(attachment) = attachment {
		data.http_client
			.get_request(&attachment.url)
			.await?
			.text()
			.await?
	} else {
		debug!("No log found in message! Skipping analysis");
		return Ok(());
	};

	let issues = issues::find(&log, data).await?;
	let info = info::find(&log);

	let show_analysis = !issues.is_empty() || matches!(info, Some(Info::Game));
	let show_upload_prompt = attachment.is_some() && info.is_some();

	if !show_analysis && !show_upload_prompt {
		debug!("Found log but there is nothing to respond with");
		return Ok(());
	}

	let allowed_mentions = CreateAllowedMentions::new().replied_user(true);

	let mut message = CreateMessage::new()
		.reference_message(message)
		.allowed_mentions(allowed_mentions);

	if show_analysis {
		message = message.add_embed({
			let mut e = CreateEmbed::new().title("Log analysis");

			if issues.is_empty() {
				e = e
					.color(Colors::Green)
					.description("The automatic check didn't reveal any issues, but it's possible that some issues went undetected. Please wait for a volunteer to assist you.");
			} else {
				e = e.color(Colors::Red);

				for (title, description) in issues {
					e = e.field(title, description, false);
				}
			}

			e
		});
	}

	if show_upload_prompt {
		message = message.add_embed(
			CreateEmbed::new()
				.title("Upload log?")
				.description(
					"Discord attachments make it difficult for volunteers to view logs. \
				 	 Would you like to upload your log to [mclo.gs](https://mclo.gs/)?",
				)
				.color(Colors::Blue),
		);
		message = message
			.button(
				CreateButton::new(BUTTON_UPLOAD_NO)
					.style(ButtonStyle::Secondary)
					.label("No"),
			)
			.button(
				CreateButton::new(BUTTON_UPLOAD_YES)
					.style(ButtonStyle::Primary)
					.label("Yes"),
			);
	}

	channel.send_message(ctx, message).await?;

	Ok(())
}

pub async fn handle_component_interaction(
	ctx: &Context,
	interaction: &ComponentInteraction,
	data: &Data,
) -> Result<()> {
	if interaction.message.kind != MessageType::InlineReply {
		debug!("Ignoring component interaction on message which is not a reply");
		return Ok(());
	}

	let yes = interaction.data.custom_id == BUTTON_UPLOAD_YES;

	if !yes && interaction.data.custom_id != BUTTON_UPLOAD_NO {
		debug!(
			"Ignoring component interaction without ID {BUTTON_UPLOAD_YES} or {BUTTON_UPLOAD_NO}"
		);
		return Ok(());
	}

	let mut embeds: Vec<CreateEmbed> = interaction
		.message
		.embeds
		.iter()
		.map(|embed| CreateEmbed::from(embed.to_owned()))
		.collect();

	// for some reason Discord never sends us the referenced message, only its id
	let message_reference = interaction
		.message
		.message_reference
		.as_ref()
		.ok_or_eyre("Missing message reference")?;

	let referenced_message = ctx
		.http
		.get_message(
			message_reference.channel_id,
			message_reference
				.message_id
				.ok_or_eyre("Reference missing message ID")?,
		)
		.await;

	let Ok(referenced_message) = referenced_message else {
		// TODO: make the bot delete its response when the initial message is deleted
		debug!("Ignoring component interaction on reply to deleted message");
		return Ok(());
	};

	// prevent other members from clicking the buttons
	if interaction.user.id != referenced_message.author.id {
		debug!(
			"Ignoring component interaction by {} on reply to message by {}",
			interaction.user.id, referenced_message.author.id
		);
		return Ok(());
	}

	if yes {
		let first_attachment = first_text_attachment(&referenced_message)
			.ok_or_eyre("Log attachment disappeared - should not be possible!")?;
		let body = data
			.http_client
			.get_request(&first_attachment.url)
			.await?
			.text()
			.await?;

		let response = mclogs::upload_log(&data.http_client, &body).await?;

		if !response.success {
			let error = response
				.error
				.ok_or_else(|| eyre!("mclo.gs gave us an error but with no message!"))?;

			interaction
				.create_response(
					&ctx.http,
					CreateInteractionResponse::Message(
						CreateInteractionResponseMessage::new()
							.ephemeral(true)
							.embed(
								CreateEmbed::new()
									.title("Upload failed")
									.description(&error)
									.color(Colors::Red),
							),
					),
				)
				.await?;

			return Err(eyre!("Failed to upload log: {}", &error));
		}

		let url = &response
			.url
			.ok_or_eyre("Missing URL in mclo.gs response!")?;

		let length = embeds.len();

		embeds[length - 1] = CreateEmbed::new()
			.title("Uploaded log")
			.description(url)
			.color(Colors::Blue);

		if matches!(info::find(&body), Some(Info::Game)) {
			interaction
				.create_response(
					ctx,
					CreateInteractionResponse::Message(
						CreateInteractionResponseMessage::new()
							.ephemeral(true)
							.embed(
								CreateEmbed::new()
									.title("Tip")
									.description(
										"Logs can be uploaded directly within the launcher.",
									)
									.color(Colors::Yellow)
									.image("https://cdn.discordapp.com/attachments/1031694870756204566/1156971972232740874/image.png"),
							),
					),
				)
				.await?;
		} else {
			interaction
				.create_response(ctx, CreateInteractionResponse::Acknowledge)
				.await?;
		}
	} else {
		embeds.pop();

		interaction
			.create_response(ctx, CreateInteractionResponse::Acknowledge)
			.await?;
	}

	if embeds.is_empty() {
		interaction.message.delete(ctx).await?;
	} else {
		ctx.http
			.edit_message(
				interaction.channel_id,
				interaction.message.id,
				&EditMessage::new().embeds(embeds).components(vec![]),
				vec![],
			)
			.await?;
	}

	Ok(())
}
