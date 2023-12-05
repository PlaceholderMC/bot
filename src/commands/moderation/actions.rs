use crate::{consts::COLORS, Context};

use color_eyre::eyre::{eyre, Result};
use poise::serenity_prelude::{CreateEmbed, User};

fn create_moderation_embed(
    title: String,
    user: &User,
    delete_messages_days: Option<u8>,
    reason: Option<String>,
) -> impl FnOnce(&mut CreateEmbed) -> &mut CreateEmbed {
    let fields = [
        ("User", format!("{} ({})", user.name, user.id), false),
        ("Reason", reason.unwrap_or("None".to_string()), false),
        (
            "Deleted messages",
            format!("Last {} days", delete_messages_days.unwrap_or(0)),
            false,
        ),
    ];

    |e: &mut CreateEmbed| e.title(title).fields(fields).color(COLORS["red"])
}

/// Ban a user
#[poise::command(
    slash_command,
    prefix_command,
    default_member_permissions = "BAN_MEMBERS"
)]
pub async fn ban_user(
    ctx: Context<'_>,
    user: User,
    delete_messages_days: Option<u8>,
    reason: Option<String>,
) -> Result<()> {
    let days = delete_messages_days.unwrap_or(1);
    let guild = ctx
        .guild()
        .ok_or_else(|| eyre!("Couldn't get guild from message; Unable to ban!"))?;

    guild
        .ban_with_reason(ctx, &user, days, reason.clone().unwrap_or_default())
        .await?;

    let embed = create_moderation_embed("User banned!".to_string(), &user, Some(days), reason);

    ctx.send(|m| m.embed(embed)).await?;

    Ok(())
}

/// Kick a user
#[poise::command(
    slash_command,
    prefix_command,
    default_member_permissions = "KICK_MEMBERS"
)]
pub async fn kick_user(ctx: Context<'_>, user: User, reason: Option<String>) -> Result<()> {
    let guild = ctx
        .guild()
        .ok_or_else(|| eyre!("Couldn't get guild from message; Unable to ban!"))?;

    guild
        .kick_with_reason(ctx, &user, &reason.clone().unwrap_or_default())
        .await?;

    let embed = create_moderation_embed("User kicked!".to_string(), &user, None, reason);

    ctx.send(|m| m.embed(embed)).await?;

    Ok(())
}