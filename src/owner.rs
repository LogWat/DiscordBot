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
        channel::GuildChannel,
    },
};
use serenity::model::prelude::*;
use serenity::prelude::*;

use crate::ShardManagerContainer;

#[command]
async fn shutdown(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    if admin(ctx, msg).await == false {
        msg.reply(ctx, "Ah... You don't get to tell me what to do.").await?;
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
pub async fn delete_msgs(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    if admin(ctx, msg).await == false {
        msg.reply(ctx, "Ah... You don't get to tell me what to do.").await?;
        return Ok(());
    }
    let num_of_delete = match args.rest().parse::<u64>() {
        Ok(num) => num,
        Err(_) => {
            msg.reply(ctx, "Invalid number of messages to delete. Please enter a number.").await?;
            return Ok(());
        }
    };
    let channel: GuildChannel = msg.channel_id.to_channel(&ctx).await.unwrap().guild().unwrap();
    let messages = match channel.messages(&ctx, |m| m.limit(num_of_delete)).await {
        Ok(messages) => messages,
        Err(_) => {
            msg.reply(ctx, "Could not find messages.").await?;
            return Ok(());
        }
    };
    if messages.is_empty() {
        msg.reply(ctx, "Could not find messages.").await?;
        return Ok(());
    }
    let mut message_ids = Vec::new();
    for message in messages {
        message_ids.push(message.id);
    }
    match channel.delete_messages(&ctx, message_ids).await {
        Ok(_) => {},
        Err(_) => {
            msg.reply(ctx, "Could not delete messages.").await?;
        }
    }

    msg.channel_id.say(&ctx, format!("Deleted {} messages.", num_of_delete)).await?;

    Ok(())
}

// judge admin
// 他のファイルから呼び出せるようにpubで宣言
pub async fn admin(ctx: &Context, msg: &Message) -> bool {
    if let Some(member) = &msg.member {
        for role in &member.roles {
            if role
                .to_role_cached(&ctx.cache)
                .await
                .map_or(false, |role| role.permissions.contains(Permissions::ADMINISTRATOR))
            {
                return true;
            }
        }
    }
    false
}