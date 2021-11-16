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
async fn am_i_admin(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    if msg.author.bot {
        return Ok(());
    }
    if let Some(member) = &msg.member {
        for role in &member.roles {
            if role
                .to_role_cached(&ctx.cache)
                .await
                .map_or(false, |r| r.has_permission(Permissions::ADMINISTRATOR))
            {
                msg.channel_id.say(ctx, "Yes!").await?;
                return Ok(());
            }
        }
    }
    msg.channel_id.say(&ctx.http, "No!").await?;
    Ok(())
}