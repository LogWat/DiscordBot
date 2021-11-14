mod commands;

use std::{collections::HashSet, fs::File, io::BufReader, usize};

use serenity::async_trait;
use serenity::framework::standard::{
    help_commands,
    macros::{group, help},
    Args, CommandGroup, CommandResult, HelpOptions,
};
use serenity::framework::StandardFramework;
use serenity::model::prelude::{channel::Message, gateway::Ready, id::UserId};
use serenity::prelude::{Client, Context, EventHandler};

use serde::{Deserialize, Serialize};
use serde_json::Result;

use commands::{test::*};

#[group]
#[description("General Command")]
#[summary("General")]
#[commands(test)]
struct General;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[help]
#[individual_command_tip = "Help Command"] // Discription of the help command
#[strikethrough_commands_tip_in_guild = ""]// Strikethrough commands in guilds
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

#[derive(Serialize, Deserialize)]
struct Token {
    token: String,
}

// func to extract the token
fn get_token(file_name: &str) -> Result<String> {
    let file = File::open(file_name).unwrap();
    let reader = BufReader::new(file);
    let t: Token = serde_json::from_reader(reader).unwrap();
    Ok(t.token)
}

#[tokio::main]
async fn main() {
    // Set the token
    let token = get_token("config.json").expect("[?] Token Not Found");
    // Set Commands
    let framework = StandardFramework::new()
        .configure(|c| c.prefix("/")) // Command Prefix
        .help(&MY_HELP)               // Add Help Command
        .group(&GENERAL_GROUP);       // Add General Command Group
    
    // Create Client
    let mut client = Client::builder(&token)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("[?] Failed to create client");

    // Run Bot
    if let Err(why) = client.start().await {
        println!("[!] Client error: {:?}", why);
    }
}