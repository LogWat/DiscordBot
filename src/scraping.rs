use serenity::{
    model::{
        prelude::*,
    },
};
use serenity::prelude::*;
use std::{env};
use std::sync::Arc;

// Scraping at regular intervals (every 5 minutes)
pub async fn scraping_price(ctx: Arc<Context>) -> Result<(), Box<dyn std::error::Error>> {
    let http = ctx.http.clone();
    let channel_id = ChannelId(env::var("PRICE_CHANNEL_ID")?.parse()?);
    channel_id.send_message(&http, |m| {
        m.embed(|e| {
            e.title("Price");
            e.description("Scraping...");
            e
        })
    }).await?;
    Ok(())
}