use serenity::framework::standard::{CommandResult};
use serenity::prelude::*;
use std::collections::HashSet;
use serenity::framework::standard::{
    help_commands,
    macros::help,
    Args, CommandGroup, HelpOptions,
};
use serenity::model::prelude::{channel::Message, id::UserId};

#[help]
#[individual_command_tip = "Help Command"]
#[strikethrough_commands_tip_in_guild = ""]
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