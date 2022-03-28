// Scheduler, trait for .seconds(), .minutes(), etc., and trait with job scheduling methods
use clokwerk::{TimeUnits, AsyncScheduler};
// Import week days and WeekDay
use std::time::Duration;
use serenity::prelude::*;
use std::sync::Arc;

use crate::scraping;

pub async fn scraping_scheduler(ctx: Arc<Context>) -> Result<(), Box<dyn std::error::Error>> {
    let mut scheduler = AsyncScheduler::new();

    // Every 5 minutes (prices)
    scheduler.every(5.minutes()).run(move || {
        let inner_ctx = ctx.clone();
        async move {
            scraping::scraping_price(inner_ctx).await.unwrap();
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