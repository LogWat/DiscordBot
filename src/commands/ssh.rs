use std::io::prelude::*;
use std::env;
use std::path::Path;
use std::net::{TcpStream};
use ssh2::Session;
use serenity::{
    framework::{
        standard::{
            macros::command,
            CommandResult,
            Args,
        },
    },
};
use serenity::model::prelude::*;
use serenity::prelude::*;

#[command]
#[description = "SSH into a server"]
async fn ssh_test(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    dotenv::dotenv().expect("Failed to load .env file");

    let username = env::var("USERNAME").expect("USERNAME not set");
    let hostname = env::var("HOSTNAME").expect("HOSTNAME not set");
    let domainname = env::var("DOMAINNAME").expect("DOMAINNAME not set");
    let password = env::var("PASSWORD").expect("PASSWORD not set");
    let host = format!("{}1{}:22", hostname, domainname);

    let tcp = TcpStream::connect(host).unwrap();
    let mut session = Session::new().unwrap();
    session.set_tcp_stream(tcp);
    session.handshake().unwrap();

    let tmp = env::var("KEY_PATH").expect("KEY_PATH not set");
    let key_path = Path::new(&tmp);
    session.userauth_pubkey_file(
        &username,
        None,
        key_path,
        Some(&password),
    ).unwrap();

    assert!(session.authenticated());

    let mut channel = session.channel_session().unwrap();
    channel.exec("who").unwrap();
    let mut s = String::new();
    channel.read_to_string(&mut s).unwrap();
    
    msg.channel_id.say(&ctx.http, &s).await?;

    channel.wait_close().unwrap();

    Ok(())
}