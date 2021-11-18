use serenity::{
    framework::{
        standard::{
            macros::command,
            CommandResult,
            Args,
        },
    },
    model::{
        permissions::Permissions,
    },
};
use serenity::model::prelude::*;
use serenity::prelude::*;

use crate::ShardManagerContainer;

#[command]
#[description = "Under Construction"]
async fn shutdown(ctx: &Context, msg: &Message) -> CommandResult {
    if msg.author.bot {
        return Ok(());
    }
    let data = ctx.data.read().await;

    if let Some(manager) = data.get::<ShardManagerContainer>() {
        msg.reply(ctx, "Shutting Down!").await?;
        manager.lock().await.shutdown_all().await;
    } else {
        msg.reply(ctx, "No ShardManager found in context").await?;
        return Ok(());
    }
    Ok(())
}

#[command]
#[required_permissions(ADMINISTRATOR)]
async fn admin(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    msg.channel_id.say(&ctx.http, "ok").await?;
    Ok(())
}