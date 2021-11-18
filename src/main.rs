mod commands;

use std::{fs::File, io::BufReader, usize, sync::Arc, collections::{HashSet}};

use serenity::{
    async_trait,
    client::bridge::gateway::ShardManager,
    framework::{
        standard::{
            macros::{group},
        },
        StandardFramework,
    },
    http::Http,
    model::prelude::{gateway::Ready},
    prelude::*,
};
use serde::{Deserialize, Serialize};
use serde_json::Result;

use commands::{test::*, help::*, owner::*};

use tokio::sync::Mutex;

// Data that can be variable shared reference in each command
pub struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

// Create Commands Group (help command is not in this group)
#[group]
#[description("General commands")]
#[summary("General")]
#[commands(test, say, shutdown)]
struct General;

struct Handler;
#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
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
        //.unrecognised_command(unknown_command) // Add Unrecognised Command
        .help(&MY_HELP)                        // Add Help Command
        .group(&GENERAL_GROUP);                // Add General Command Group
    
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