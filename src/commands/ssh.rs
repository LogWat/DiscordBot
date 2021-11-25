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

async fn ssh_error(ctx: &Context, msg: &Message) {
    let error_msg = "Argument Error! Usage: ssh_error <num1>, <num2>,... or <num1>:<num2>";
    let _ = msg.reply(ctx, error_msg).await;
}

// TO DO
// スクレイピングによるssh接続先OSの識別
// ↑で取得した情報から，引数で指定されたホスト内から不必要なものを弾く
// hostnameやpasswordは通常BOT起動中は不変なので，一度BOT起動時に読み込ませておく
#[command]
#[description = "SSH into a server"]
async fn ssh_test(ctx: &Context, msg: &Message, args: Args) -> CommandResult {

    let mut args_m = args;
    let mut hosts = Vec::new();
    for arg in args_m.iter::<String>() {
        match arg {
            Ok(arg) => {
                match arg.parse::<u32>() {
                    Ok(host) => {
                        hosts.push(host);
                    },
                    Err(_) => {
                        let slice_arg = arg.as_str();
                        if slice_arg.contains(":") {
                            let nums = slice_arg.split(":").collect::<Vec<&str>>();
                            if nums.len() > 2 {
                                ssh_error(ctx, msg).await;
                                return Ok(());
                            } else {
                                let num1 = nums[0].parse::<u32>();
                                let num2 = nums[1].parse::<u32>();
                                if num1.is_ok() && num2.is_ok() {
                                    let mut a = num1.unwrap();
                                    let mut b = num2.unwrap();
                                    if a > b {
                                        std::mem::swap(&mut a, &mut b);
                                    }
                                    for i in a..(b+1) {
                                        hosts.push(i);
                                    }
                                } else {
                                    ssh_error(ctx, msg).await;
                                    return Ok(());
                                }
                            }
                        } else {
                            ssh_error(ctx, msg).await;
                            return Ok(());
                        }
                    }
                }
            },
            Err(_e) => {
                ssh_error(ctx, msg).await;
               return Ok(());
            }
        };
    }

    for i in hosts {
        println!("{}", i);
    }

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
    
    msg.channel_id.say(&ctx.http, "Argument test! This is a dummy message lol").await?;

    channel.wait_close().unwrap();

    Ok(())
}