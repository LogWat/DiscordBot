use serenity::framework::standard::{CommandResult};
use serenity::prelude::*;
use std::collections::HashSet;
use serenity::framework::standard::{
    help_commands,
    macros::{help, hook},
    Args, CommandGroup, HelpOptions,
};
use serenity::model::prelude::{channel::Message, id::UserId};

#[help]
#[individual_command_tip = "Help Info"]
#[strikethrough_commands_tip_in_guild = ""]
// #[lacking_permissions = "Hide"]
async fn my_help(
    ctx: &Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    let _ = help_commands::with_embeds(ctx, msg, args, help_options, groups, owners).await;
    Ok(())
}

#[hook]
async fn unknown_command(ctx: &Context, msg: &Message, _: &str) -> CommandResult {
    msg.channel_id
        .say(&ctx.http, "Unknown command. Try `/help`.")
        .await?;
    Ok(())
}