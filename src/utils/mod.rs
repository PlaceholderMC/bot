use poise::serenity_prelude::{Attachment, CreateEmbedAuthor, Message, User};

pub mod messages;

pub fn embed_author_from_user(user: &User) -> CreateEmbedAuthor {
	CreateEmbedAuthor::new(user.tag()).icon_url(
		user.avatar_url()
			.unwrap_or_else(|| user.default_avatar_url()),
	)
}

pub fn semver_split(version: &str) -> Vec<u32> {
	version
		.split('.')
		.filter_map(|s| s.parse().ok())
		.collect::<Vec<u32>>()
}

pub fn first_text_attachment(message: &Message) -> Option<&Attachment> {
	message.attachments.iter().find(|a| {
		a.content_type
			.as_ref()
			.is_some_and(|content_type| content_type.starts_with("text/"))
	})
}
