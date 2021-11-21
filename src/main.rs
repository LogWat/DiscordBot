mod commands;

use std::{
    fs::File, 
    io::BufReader, 
    usize, 
    sync::Arc, 
    collections::{HashSet, HashMap},
};

use serenity::{
    async_trait,
    client::bridge::gateway::ShardManager,
    framework::{
        standard::{
            macros::{group, hook},
            StandardFramework,
        },
    },
    model::{
        channel::Message,
        gateway::Ready,
    },
    http::Http,
    prelude::*,
};
use serde::{Deserialize, Serialize};
use serde_json::Result;

use commands::{test::*, help::*, owner::*};

use tokio::sync::Mutex;

// Data that can be variable shared reference in each command
struct ShardManagerContainer;
impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

pub struct CommandCounter;
impl TypeMapKey for CommandCounter {
    type Value = HashMap<String, u64>;
}

// Create Commands Group (help command is not in this group)
#[group]
#[description("General commands")]
#[summary("General")]
#[commands(test, say, shutdown, commands)]
struct General;

struct Handler;
#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }

    async fn message(&self, ctx: Context, msg: Message) {
        // get self id
        let self_id = ctx.http.get_current_user().await.unwrap().id;
        for mention in msg.mentions.iter() {
            if mention.id == self_id {
                msg.channel_id.say(&ctx.http, format!("{}, Hello!", msg.author.mention())).await.unwrap();
            }
        }
    }
}

#[hook]
async fn unknown_command(ctx: &Context, msg: &Message, unknown_command_name: &str) {
    msg.channel_id
        .say(&ctx.http, format!("Unknown command: '{}'. Try `/help`.", unknown_command_name))
        .await.unwrap();
}

#[hook]
async fn before(ctx: &Context, _msg: &Message, command_name: &str) -> bool {
    let mut data = ctx.data.write().await;
    let counter = data.get_mut::<CommandCounter>().expect("Expected CommandCounter in TypeMap.");
    let entry = counter.entry(command_name.to_string()).or_insert(0);
    *entry += 1;

    true
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

    let http = Http::new_with_token(&token);
    let (owners, bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            if let Some(team) = info.team {
                owners.insert(team.owner_user_id);
            } else {
                owners.insert(info.owner.id);
            }
            match http.get_current_user().await {
                Ok(bot_id) => (owners, bot_id.id),
                Err(why) => panic!("Could not access user info: {:?}", why),
            }
        },
        Err(why) => panic!("Could not access application info: {:?}", why),
    };

    // Set Commands
    let framework = StandardFramework::new()
        .configure(|c| c
            .prefix("/")                       // Command Prefix
            .with_whitespace(true)             // Allow whitespace in prefix
            .delimiters(vec![",", ", "])       // Delimiters for args
            .owners(owners)                    // Owners
            .on_mention(Some(bot_id))          // Allow mentioning the bot
        )
        .unrecognised_command(unknown_command) // Add Unrecognised Command
        .help(&MY_HELP)                        // Add Help Command
        .group(&GENERAL_GROUP);                // Add General Command Group
    
    // Create Client
    let mut client = Client::builder(&token)
        .event_handler(Handler)
        .framework(framework)
        .type_map_insert::<CommandCounter>(HashMap::default())
        .await
        .expect("[?] Failed to create client");

    {
        let mut data = client.data.write().await;
        data.insert::<ShardManagerContainer>(client.shard_manager.clone());
    }

    let shard_manager = client.shard_manager.clone();

    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.expect("Could not register ctrl+c handler");
        shard_manager.lock().await.shutdown_all().await;
    });

    // Run Bot
    if let Err(why) = client.start().await {
        println!("[!] Client error: {:?}", why);
    }
}