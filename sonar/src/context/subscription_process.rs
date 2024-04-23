use std::time::Duration;

use crate::{Context, Result};

pub(super) async fn run(context: &Context) {
    loop {
        tokio::time::sleep(Duration::from_mins(1)).await;
        if let Err(err) = iteration(context).await {
            tracing::error!("error running subscription loop iteration: {err}");
        }
    }
}

async fn iteration(context: &Context) -> Result<()> {
    let subscriptions = super::subscription_list_all(context).await?;
    for subscription in subscriptions {
        let interval = match subscription.interval {
            Some(interval) => interval,
            None => continue,
        };
        match subscription.last_submitted {
            Some(ts) if ts.elapsed() < interval => continue,
            _ => {}
        };
        super::subscription_submit(context, subscription.id).await?;
    }
    Ok(())
}
