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

// [!] TODO: Error handling

// Scraping at regular intervals (every 5 minutes)
pub async fn scraping_price(ctx: Arc<Context>) -> Result<(), Box<dyn std::error::Error>> {
    let target_url = env::var("PT_URL").unwrap();
    let target_sub_url = env::var("PTS_URL").unwrap();
    let target_query = env::var("PT_QUERY").unwrap();
    let resp = reqwest::get(&target_url).await?;
    let body = resp.text().await?;

    Ok(())
}

// Scraping weather news (every 1 day)
pub async fn scraping_weather(ctx: Arc<Context>) -> Result<(), Box<dyn std::error::Error>> {
    let channel_id: ChannelId = env::var("WEATHER_CHANNEL_ID").expect("Error: WEATHER_CHANNEL_ID is not set.").parse()?;
    let weather_news = env::var("WT_URL").unwrap();
    let mut msg = String::new();
    {
        let doc = scraping_url(&weather_news).await?;
        let selector = Selector::parse("table.forecast-point-3h tbody tr.weather").unwrap();
        for node in doc.select(&selector) {
            msg.push_str(&format!("{}", node.text().collect::<Vec<_>>().join("\n")));
        }
    }

    channel_id.send_message(&ctx, |m| m.content(msg)).await?;

    Ok(())
}

// scraping and return doc of url
async fn scraping_url(url: &str) -> Result<Html, Box<dyn std::error::Error>> {
    let resp = reqwest::get(url).await?;
    let body = resp.text().await?;
    let doc = Html::parse_document(&body);

    Ok(doc)
}