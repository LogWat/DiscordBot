// Scheduler, trait for .seconds(), .minutes(), etc., and trait with job scheduling methods
use clokwerk::{TimeUnits, AsyncScheduler, Job};
// Import week days and WeekDay
use std::time::Duration;
use serenity::prelude::*;
use std::sync::Arc;

use crate::scraping;

pub async fn scraping_scheduler(ctx: Arc<Context>) -> Result<(), Box<dyn std::error::Error>> {
    let mut scheduler = AsyncScheduler::new();

    // prices scraping (every 1 hour)
    let ctx_clone1 = ctx.clone();
    scheduler.every(1.hour()).run(move || {
        let inner_ctx = ctx_clone1.clone();
        async move {
            scraping::scraping_price(inner_ctx).await.unwrap();
        }
    });

    // Weather scraping (every 1 day (8:00 am))
    let ctx_clone2 = ctx.clone();
    scheduler.every(1.days()).at("08:00").run(move || {
        let inner_ctx = ctx_clone2.clone();
        async move {
            scraping::scraping_weather(inner_ctx).await.unwrap();
        }
    });

    tokio::spawn(async move {
        loop {
          scheduler.run_pending().await;
          tokio::time::sleep(Duration::from_millis(100)).await;
        }
    });

    Ok(())
}