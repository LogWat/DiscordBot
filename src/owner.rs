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
async fn shutdown(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    if admin(ctx, msg, args).await == false {
        msg.reply(ctx, "You don't have permission to do that!").await?;
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

// judge admin
// 他のファイルから呼び出せるようにpubで宣言
#[warn(dead_code)]
pub async fn admin(ctx: &Context, msg: &Message, _args: Args) -> bool {
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