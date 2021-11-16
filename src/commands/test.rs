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

#[command]
async fn test(ctx: &Context, msg: &Message) -> CommandResult {
    if msg.author.bot {
        return Ok(());
    }
    msg.channel_id
        .say(&ctx.http, format!("{}, Hello!", msg.author.mention()))
        .await?;
    Ok(())
}

#[command]
async fn fuckog(ctx: &Context, msg: &Message) -> CommandResult {
    if msg.author.bot {
        return Ok(());
    }
    msg.channel_id
        .say(&ctx.http, format!("F*ck you, {}", msg.author.mention()))
        .await?;
    Ok(())
}

#[command]
async fn say(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    if msg.author.bot {
        return Ok(());
    }
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