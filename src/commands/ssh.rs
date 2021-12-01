use std::io::prelude::*;
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

use crate::EnvData;

struct HostStatus {
    kind: String,
    status: bool,
}

async fn ssh_error(ctx: &Context, msg: &Message) {
    let error_msg = "Argument Error! Usage: ssh <num1>, <num2>,... or <num1>:<num2>";
    let _ = msg.reply(ctx, error_msg).await;
}

// TO DO
// スクレイピングによるssh接続先OSの識別（接続要求タイミングを考える！！！）
// ↑で取得した情報から，引数で指定されたホスト内から不必要なものを弾く
// hostnameやpasswordは通常BOT起動中は不変なので，一度BOT起動時に読み込ませておく
#[command]
#[description = "SSH into a server"]
async fn ssh_test(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    
    let data = ctx.data.read().await;
    let envdata = data.get::<EnvData>().unwrap();
    let user = envdata.user.clone();
    let host = envdata.host.clone();
    let domain = envdata.domain.clone();
    let key_pass = envdata.key_pass.clone();
    let key_path = envdata.key_path.clone();

    // get status info by scraping
    let host_status_url = envdata.host_status.clone();
    let mut host_statuses: Vec::<HostStatus> = Vec::new();
    match reqwest::get(&host_status_url).await {
        Ok(response) => {
            let doc = scraper::Html::parse_document(&response.text().await.unwrap());
            let div_selecter = scraper::Selector::parse("div.col-sm").unwrap();
            let div_element = doc.select(&div_selecter).next().unwrap();
            let li_selector = scraper::Selector::parse("li").unwrap();
            let use_selector = scraper::Selector::parse("use").unwrap();
            for li_element in div_element.select(&li_selector) {
                let status = format!("{:?}", li_element.value().attr("class").unwrap());
                if status.contains("success") {
                    let use_element = li_element.select(&use_selector).next().unwrap();
                    host_statuses.push(HostStatus {
                        kind: format!("{:?}", use_element.value().attr("href").unwrap()),
                        status: true,
                    });
                } else {
                    host_statuses.push(HostStatus {
                        kind: format!("{}", "NULL"),
                        status: false,
                    });
                }
            }
            for host_status in host_statuses {
                println!("{}", host_status.kind);
                println!("{}", host_status.status);
            }
        },
        Err(_e) => {
            let _ = msg.react(ctx, '\u{1F6AB}').await;
            return Ok(());
        },
    };

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

    let target = format!("{}1{}:22", host, domain);

    let tcp = TcpStream::connect(target).unwrap();
    let mut session = Session::new().unwrap();
    session.set_tcp_stream(tcp);
    session.handshake().unwrap();

    let key_path = Path::new(&key_path);
    session.userauth_pubkey_file(
        &user,
        None,
        key_path,
        Some(&key_pass),
    ).unwrap();

    assert!(session.authenticated());

    let mut channel = session.channel_session().unwrap();
    channel.exec("who").unwrap();
    let mut s = String::new();
    channel.read_to_string(&mut s).unwrap();
    
    msg.channel_id.say(&ctx.http, format!("Argument test! {:?}", hosts)).await?;

    channel.wait_close().unwrap();

    Ok(())
}