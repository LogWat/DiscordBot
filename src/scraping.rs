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
use std::sync::{Arc, RwLock};

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
    type Value = Arc<RwLock<Vec<ItemHistory>>>;
}

// [!] TODO: Error handling

// Scraping at regular intervals (every 5 minutes)
pub async fn scraping_price(ctx: Arc<Context>) -> Result<(), Box<dyn std::error::Error>> {

    let ihc_lock = {
        let data_read = ctx.data.read().await;
        data_read.get::<ItemHistoryContainer>().expect("Failed to get ItemHistoryContainer").clone()
    };

    let mut ihc_len = 0;
    {
        let ihc_lock = ihc_lock.read().expect("Failed to get ItemHistoryContainer");
        ihc_len = ihc_lock.len();
    }

    if ihc_len == 0 {
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
                    detail_url: format!("{}/item/{}", target_url, id),
                };

                items.push(item);
            }
        }
    }

    for item in items {
        let mut values = vec![];
        let mut dates = vec![];
        {
            let mut url = item.detail_url.clone();
            url.push_str(&tds_url);
            let doc = scraping_url(&url, "shift_jis").await?;
            for cnode in doc.select(&value_selector) {
                let price_raw = cnode.text().next().unwrap();
                let price = match re_notnum.replace_all(price_raw, "").parse::<u32>() {
                    Ok(p) => p,
                    Err(_) => continue,
                };
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
            
        let ihc_lock = {
            let data_read = ctx.data.read().await;
            data_read.get::<ItemHistoryContainer>().expect("Failed to get ItemHistoryContainer").clone()
        };

        {
            let mut ihc = ihc_lock.write().expect("Failed to get ItemHistoryContainer");
            ihc.push(
                ItemHistory {
                    item: item.clone(),
                    min_price: values.iter().min().unwrap().clone(),
                    date_range: format!("{} ~ {}", dates[dates.len() - 1], dates[0]),
                }
            );
        }

        let mut msg = String::new();
        msg.push_str(&format!("Item Name: {}\n", item.name));
        msg.push_str(&format!("Date Range: {} ~ {}\n", dates[dates.len() - 1], dates[0]));
        msg.push_str(&format!("Min Price: {} yen\n", values.iter().min().unwrap()));
        msg.push_str("\n");
        println!("{}", msg);
    }

    Ok(())
}

// Price History Container に アイテムが存在する場合は各アイテムの最小値を比較して更新点があれば通知
async fn price_scrape_update(ctx: Arc<Context>) -> Result<(), Box<dyn std::error::Error>> {

    let ihc_lock = {
        let data_read = ctx.data.read().await;
        data_read.get::<ItemHistoryContainer>().expect("Failed to get ItemHistoryContainer").clone()
    };

    let mut ihc_copy = Vec::new();
    {
        let ihc = ihc_lock.read().expect("Failed to read ItemHistoryContainer");
        ihc_copy.extend(ihc.clone());
    }

    let price_selctor = Selector::parse(
        r#"div#all div#main div.contents930 div.outerRecommend div#itmBoxMax div.itmBoxBottom div.itmBoxln div#productAll
        div#ProductInfoBox div.priceBoxWrap div.priceWrap div.subInfoObj1 p span.priceTxt"#
    ).unwrap();
    let re_notnum = Regex::new(r"\D").unwrap();
    let channel_id: ChannelId = env::var("PRICE_CHANNEL_ID").expect("PRICE_CHANNEL_ID not found").parse().unwrap();

    for item in ihc_copy {
        let url = item.item.detail_url.clone();
        let mut price = 0;
        {
            let doc = scraping_url(&url, "shift_jis").await?;
            let price_raw = match doc.select(&price_selctor).next() {
                Some(node) => node.text().next().unwrap(),
                None => continue,
            };
            price = match re_notnum.replace_all(price_raw, "").parse::<u32>() {
                Ok(p) => p,
                Err(_) => continue,
            };
        }
        if price < item.min_price {
            let msg = format!("{} の最小価格が {} yen から {} yen に更新されました！\n", item.item.name, item.min_price, price);
            channel_id.say(&ctx.http, msg).await?;
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
    /*
    for hour in hours.iter() {
        msg.push_str(&format!("　{}　", hour));
    }
    */
    msg.push_str("\n");
    {
        let doc = scraping_url(&weather_news, "utf-8").await?;
        let rainy_p_selector = Selector::parse(r#"table[id="forecast-point-3h-today"] tbody tr.prob-precip td span"#).unwrap();
        let mut rainy_p = vec![];
        for node in doc.select(&rainy_p_selector) {
            rainy_p.push(node.text().next().unwrap().to_string());
        }
        let selector = Selector::parse(r#"table[id="forecast-point-3h-today"] tbody tr.weather td p"#).unwrap();
        for (i, node) in doc.select(&selector).enumerate() {
            let c = node.text().collect::<Vec<_>>();
            if c.len() < 1 {
                continue;
            }
            if i > rainy_p.len() {
                msg.push_str("Error: rainy_p is shorter than weather elements.\n");
                break;
            }
            let w = c[0];
            let mut rainy_p_warning = String::new();
            let rainy_p_num = match rainy_p[i].parse::<u32>() {
                Ok(p) => p,
                Err(_) => {
                    msg.push_str(&format!("Error: {} is not a number.\n", rainy_p[i]));
                    continue;
                }
            };
            if rainy_p_num >= 50 {
                rainy_p_warning.push_str("<-- 降水確率50%以上");
            }
            match w {
                "晴れ" => {
                    if i >= 6 {
                        msg.push_str(&format!("{}: {} {}\n", hours[i], ":crescent_moon:", rainy_p_warning)); // 夜は月
                    } else {
                        msg.push_str(&format!("{}: {} {}\n", hours[i], ":sunny:", rainy_p_warning));
                    }
                },
                "曇り" => msg.push_str(&format!("{}: {} {}\n", hours[i], ":cloud:", rainy_p_warning)),
                "雨" => msg.push_str(&format!("{}: {} {}\n", hours[i], ":umbrella:", rainy_p_warning)),
                "小雨" => msg.push_str(&format!("{}: {} {}\n", hours[i], ":closed_umbrella:", rainy_p_warning)),
                "弱雨" => msg.push_str(&format!("{}: {} {}\n", hours[i], ":umbrella2:", rainy_p_warning)),
                "雪" => msg.push_str(&format!("{}: {} {}\n", hours[i], ":snowflake:", rainy_p_warning)),
                _ => msg.push_str(&format!("{}: {} {}\n", hours[i], ":question:", rainy_p_warning)),        // TODO: 大雨とかの追加
            }
        }
        msg.push_str("\n");
        let max_temp_selctor = Selector::parse(
            r#"header[class="header clearfix"] div#hd ul#history-entries li#history-entry-0 a[class="history-entries-link clearfix"] div.info-box span.temp-box smap.max_t"#
        ).unwrap();
        let min_temp_selctor = Selector::parse(
            r#"header[class="header clearfix"] div#hd ul#history-entries li#history-entry-0 a[class="history-entries-link clearfix"] div.info-box span.temp-box smap.min_t"#
        ).unwrap();
        for node in doc.select(&max_temp_selctor) {
            let c = node.text().collect::<Vec<_>>();
            if c.len() < 1 {
                continue;
            }
            let max_temp = c[0];
            msg.push_str(&format!("最高気温: {}, ", max_temp));
            break;
        }
        for node in doc.select(&min_temp_selctor) {
            let c = node.text().collect::<Vec<_>>();
            if c.len() < 1 {
                continue;
            }
            let min_temp = c[0];
            msg.push_str(&format!("最低気温: {}\n", min_temp));
            break;
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
    println!("Scraping {}", url);
    let resp = reqwest::get(url).await?;
    let body = resp.text_with_charset(charset).await?; // Content-Type の charsetが utf-8 以外でも取得できる
    let doc = Html::parse_document(&body);

    Ok(doc)
}