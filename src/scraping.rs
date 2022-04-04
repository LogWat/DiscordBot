use serenity::{
    model::{
        prelude::*,
    },
};
use serenity::prelude::*;
use reqwest;
use scraper::{
    Html,
    Selector,
};
use std::{env};
use std::sync::Arc;

struct Item {
    name: String,       // Item name
    type_id: u32,       // Item type id (for each spec)
    id: String,         // Item id (for each item to use in the details page)
    detail_url: String, // Item detail url
}

// [!] TODO: Error handling

// Scraping at regular intervals (every 5 minutes)
pub async fn scraping_price(ctx: Arc<Context>) -> Result<(), Box<dyn std::error::Error>> {
    let target_url = env::var("PT_URL").unwrap();
    let target_sub_url = env::var("PTS_URL").unwrap();
    let target_query = env::var("PT_QUERY").unwrap();
    let tds_url = env::var("PTSH_URL").unwrap();

    let mut items: Vec<Item> = Vec::new();
    {
        let types = vec![481, 485, 480, 486, 479, 482, 484, 487];
        let selector = Selector::parse(
            r#"table.tbl-compare02 tbody tr.tr-border td.td-price ul li.pryen a"#
        ).unwrap();
        for type_id in types {
            let url = format!("{}{}?{}={}", target_url, target_sub_url, target_query, type_id);
            let doc = scraping_url(&url).await?;
            for (i, node) in doc.select(&selector).enumerate() {
                if i > 4 {
                    // 上位5件のみ
                    break;
                }
                let item_value = node.text().next().unwrap();
                let item_href = node.value().attr("href").unwrap();

                // extract detail_id from href (detail_id = KXXXXX)
                let id_index = item_href.find("K").unwrap();
                let mut id = item_href[id_index..].to_string();
                let id_end_index = id.find("/").unwrap();
                id = id[..id_end_index].to_string();

                items.push(Item {
                    name: item_value.to_string(),
                    type_id: type_id,
                    id: id.clone(),
                    detail_url: format!("{}/item/{}{}", target_url, id, tds_url),
                });
            }
        }
    }

    Ok(())
}

// Scraping weather news (every 1 day)
pub async fn scraping_weather(ctx: Arc<Context>) -> Result<(), Box<dyn std::error::Error>> {
    let channel_id: ChannelId = env::var("WEATHER_CHANNEL_ID").expect("Error: WEATHER_CHANNEL_ID is not set.").parse()?;
    let weather_news = env::var("WT_URL").unwrap();
    let mut msg = String::new();
    let hours = ["03", "06", "09", "12", "15", "18", "21", "24"];
    for hour in hours.iter() {
        msg.push_str(&format!("{}", hour));
    }
    msg.push_str("\n");
    {
        let doc = scraping_url(&weather_news).await?;
        let selector = Selector::parse(r#"table[id="forecast-point-3h-today"] tbody tr.weather td p"#).unwrap();
        for (i, node) in doc.select(&selector).enumerate() {
            let c = node.text().collect::<Vec<_>>();
            if c.len() < 1 {
                continue;
            }
            let w = c[0];
            match w {
                "晴れ" => {
                    if i >= 6 {
                        msg.push_str(":crescent_moon:"); // 夜は月
                    } else {
                        msg.push_str(":sunny:");
                    }
                },
                "曇り" => msg.push_str(":cloud:"),
                "雨" => msg.push_str(":umbrella:"),
                "小雨" => msg.push_str(":closed_umbrella:"),
                "弱雨" => msg.push_str(":umbrella2:"),
                "雪" => msg.push_str(":snowflake:"),
                _ => msg.push_str(":question:"),        // TODO: 大雨とかの追加
            }
            msg.push_str(" ");
        }
    }
    channel_id.send_message(&ctx.http, |m| {
        m.embed(|e| {
            e.title("今日の天気予報");
            e.description(msg);
            e
        })
    }).await?;

    Ok(())
}

// scraping and return doc of url
async fn scraping_url(url: &str) -> Result<Html, Box<dyn std::error::Error>> {
    let resp = reqwest::get(url).await?;
    let body = resp.text().await?;
    let doc = Html::parse_document(&body);

    Ok(doc)
}