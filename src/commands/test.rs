use serenity::model::prelude::*;
use serenity::prelude::*;
use serenity::{
    framework::standard::{
        macros::{command},
        Args, 
        CommandResult
    },
    model::{
        channel::Message,
    },
    utils::{content_safe, ContentSafeOptions},
};
use std::fmt::Write;

use crate::CommandCounter;

#[command]
#[description("Say hello")]
async fn test(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id
        .say(&ctx.http, format!("{}, Hello!", msg.author.mention()))
        .await?;
    Ok(())
}

#[command]
async fn fuckog(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id
        .say(&ctx.http, format!("F*ck you, {}", msg.author.mention()))
        .await?;
    Ok(())
}

#[command]
#[description("Repeat (Converting content to secure text)")]
async fn say(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let settings = if let Some(guild_id) = msg.guild_id {
        ContentSafeOptions::default()
            .clean_channel(false)
            .display_as_member_from(guild_id)
    } else {
        ContentSafeOptions::default()
            .clean_channel(false)
            .clean_role(false)
    };

    let content = content_safe(&ctx.cache, &args.rest(), &settings).await;
    msg.channel_id
        .say(&ctx.http, content)
        .await?;
    Ok(())
}

#[command]
#[bucket = "complicated"]
async fn commands(ctx: &Context, msg: &Message) -> CommandResult {
    let mut contents = "Commands used:\n".to_string();
    let data = ctx.data.read().await;
    let counter = data.get::<CommandCounter>().expect("Expected CommandCounter in TypeMap.");

    for (k, v) in counter {
        writeln!(contents, "{}: {}", k, v)?;
    }
    msg.channel_id.say(&ctx.http, &contents).await?;
    Ok(())
}
