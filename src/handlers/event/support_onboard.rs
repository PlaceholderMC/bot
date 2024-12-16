use eyre::{eyre, OptionExt, Result};
use log::{debug, trace};
use poise::serenity_prelude::{
	ChannelType, Context, CreateAllowedMentions, CreateMessage, GuildChannel,
};

pub async fn handle(ctx: &Context, thread: &GuildChannel) -> Result<()> {
	if thread.kind != ChannelType::PublicThread {
		trace!("Not doing support onboard in non-public thread channel");
		return Ok(());
	}

	if thread.last_message_id.is_some() {
		debug!("Ignoring duplicate thread creation event");
		return Ok(());
	}

	if thread
		.parent_id
		.ok_or_else(|| eyre!("Couldn't get parent ID from thread {}!", thread.name))?
		.name(ctx)
		.await
		.unwrap_or_default()
		!= "launcher-support"
		&& thread
			.parent_id
			.ok_or_else(|| eyre!("Couldn't get parent ID from thread {}!", thread.name))?
			.name(ctx)
			.await
			.unwrap_or_default()
			!= "mod-support"
	{
		debug!("Not posting onboarding message to threads outside of support");
		return Ok(());
	}

	let owner = thread
		.owner_id
		.ok_or_eyre("Couldn't get owner of thread!")?;

	let msg = format!(
    "<@{}> We've received your support ticket! {} {}",
    owner,
    "Please upload your logs and post the link here if possible (run `tag log` to find out how).",
    "Please don't ping people for support questions, unless you have their permission."
    );

	let allowed_mentions = CreateAllowedMentions::new()
		.replied_user(true)
		.users(Vec::from([owner]));

	let message = CreateMessage::new()
		.content(msg)
		.allowed_mentions(allowed_mentions);

	thread.send_message(ctx, message).await?;

	Ok(())
}
