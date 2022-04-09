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
use regex::Regex;
use std::{env};
use std::sync::Arc;

#[derive(Clone)]
pub struct Item {
    pub name: String,       // Item name
    pub type_id: u32,       // Item type id (for each spec)
    pub id: String,         // Item id (for each item to use in the details page)
    pub detail_url: String, // Item detail url
}

#[derive(Clone)]
pub struct ItemHistory {
    pub item: Item,
    pub min_price: u32,
    pub date_range: String,
}

pub struct ItemHistoryContainer;
impl TypeMapKey for ItemHistoryContainer {
    type Value = Arc<Mutex<Vec<ItemHistory>>>;
}

// [!] TODO: Error handling

// Scraping at regular intervals (every 5 minutes)
pub async fn scraping_price(ctx: Arc<Context>) -> Result<(), Box<dyn std::error::Error>> {

    let ihc_lock = {
        let data_read = ctx.data.read().await;
        data_read.get::<ItemHistoryContainer>().unwrap().clone()
    };

    let ihc = ihc_lock.lock().await;
    if ihc.len() == 0 {
        price_scrape_first(ctx).await?;
    } else {
        price_scrape_update(ctx).await?;
    }

    Ok(())
}


// Price History Container に 何もない場合は各アイテムの最小値を表から作成して追加 宣言
async fn price_scrape_first(ctx: Arc<Context>) -> Result<(), Box<dyn std::error::Error>> {
    let target_url = env::var("PT_URL").expect("PT_URL not found");
    let target_sub_url = env::var("PTS_URL").expect("PTS_URL not found");
    let target_query = env::var("PT_QUERY").expect("PT_QUERY not found");
    let tds_url = env::var("PTSH_URL").expect("PTSH_URL not found");

    let types = vec![481, 485, 480, 486, 479, 482, 484, 487];
    let selector = Selector::parse(
        r#"table.tbl-compare02 tbody tr.tr-border td[class="end checkItem"] table tbody tr td.ckitemLink a.ckitanker"# // KXXXX, Item Name
    ).unwrap();
    let value_selector = Selector::parse(
        r#"table[id="priceHistoryTbl2"] tbody tr td[class="alignR itemviewColor06"] strong"#                           // Item Price
    ).unwrap();
    let date_selector = Selector::parse(
        r#"table[id="priceHistoryTbl2"] tbody tr td.alignL"#                                                           // Date
    ).unwrap();
    let re_notnum = Regex::new(r"\D").unwrap();

    let mut items = Vec::new();
    {
        for type_id in types {
            let url = format!("{}{}?{}={}", target_url, target_sub_url, target_query, type_id);
            let doc = scraping_url(&url, "shift_jis").await?;
            for (i, node) in doc.select(&selector).enumerate() {
                if i > 4 {
                    // 上位5件のみ
                    break;
                }
                let name = node.text().collect::<Vec<_>>().join("").replace("　", " ");

                let item_href = node.value().attr("href").unwrap();
                // extract detail_id from href (detail_id = KXXXXX)
                let id_index = item_href.find("K").unwrap();
                let mut id = item_href[id_index..].to_string();
                let id_end_index = id.find("/").unwrap();
                id = id[..id_end_index].to_string();

                let item = Item {
                    name: name,
                    type_id: type_id,
                    id: id.clone(),
                    detail_url: format!("{}/item/{}{}", target_url, id, tds_url),
                };

                items.push(item);
            }
        }
    }

    let mut msg = String::new();
    let ihc_lock = {
        let data_read = ctx.data.read().await;
        data_read.get::<ItemHistoryContainer>().unwrap().clone()
    };
    for item in items {
        let mut values = vec![];
        let mut dates = vec![];

        {
            let doc = scraping_url(&item.detail_url, "shift_jis").await?;
            for cnode in doc.select(&value_selector) {
                let price_raw = cnode.text().next().unwrap();
                let price = re_notnum.replace_all(price_raw, "").parse::<u32>().unwrap();
                values.push(price);
            }
            for node in doc.select(&date_selector) {
                for n in node.text() {
                    if n.to_string().contains("日") {
                        dates.push(n.to_string());
                    }
                }
            }
        }
            
        let mut ihc = ihc_lock.lock().await;
        ihc.push(
            ItemHistory {
                item: item.clone(),
                min_price: values.iter().min().unwrap().clone(),
                date_range: format!("{} ~ {}", dates[dates.len() - 1], dates[0]),
            }
        );

        msg.push_str(&format!("Item Name: {}\n", item.name));
        msg.push_str(&format!("Date Range: {} ~ {}\n", dates[dates.len() - 1], dates[0]));
        msg.push_str(&format!("Min Price: {} yen\n", values.iter().min().unwrap()));
        msg.push_str("\n");
    }

    let channel_id: ChannelId = env::var("PRICE_CHANNEL_ID").unwrap().parse().unwrap();

    channel_id.send_message(&ctx.http, |m| m
        .embed(|e| e
            .title("Price History")
            .description(msg)
            .color(0x00FF00)
        )
    ).await?;
    Ok(())
}

// Price History Container に アイテムが存在する場合は各アイテムの最小値を比較して更新点があれば通知
async fn price_scrape_update(ctx: Arc<Context>) -> Result<(), Box<dyn std::error::Error>> {

    let mut msg = String::new();

    msg.push_str("GnTest");

    let channel_id: ChannelId = env::var("PRICE_CHANNEL_ID").unwrap().parse().unwrap();

    channel_id.send_message(&ctx.http, |m| m
        .embed(|e| e
            .title("Price History")
            .description(msg)
            .color(0x00FF00)
        )
    ).await?;
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
        let doc = scraping_url(&weather_news, "utf-8").await?;
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
async fn scraping_url(url: &str, charset: &str) -> Result<Html, Box<dyn std::error::Error>> {
    let resp = reqwest::get(url).await?;
    let body = resp.text_with_charset(charset).await?; // Content-Type の charsetが utf-8 以外でも取得できる
    let doc = Html::parse_document(&body);

    Ok(doc)
}