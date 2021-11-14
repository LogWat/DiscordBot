use serenity::framework::standard::{macros::command, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;

#[command]
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