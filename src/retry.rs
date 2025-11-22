use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, warn};

pub struct RetryConfig {
    pub max_retries: Option<u32>,
    pub base_delay: Duration,
    pub max_delay: Duration,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: None, // Infinite retries
            base_delay: Duration::from_secs(2),
            max_delay: Duration::from_secs(60),
        }
    }
}

pub async fn retry_with_backoff<F, T, E>(
    mut operation: F,
    config: RetryConfig,
) -> Result<T, E>
where
    F: FnMut() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, E>> + Send>>,
    E: std::fmt::Display,
{
    let mut attempt = 0;

    loop {
        match operation().await {
            Ok(result) => {
                if attempt > 0 {
                    info!("✅ Operation succeeded after {} attempts", attempt + 1);
                }
                return Ok(result);
            }
            Err(e) => {
                if let Some(max) = config.max_retries {
                    if attempt >= max {
                        warn!("❌ Max retries ({}) reached", max);
                        return Err(e);
                    }
                }

                let delay = std::cmp::min(
                    config.base_delay * 2_u32.pow(attempt),
                    config.max_delay,
                );

                warn!(
                    "⚠️  Attempt {} failed: {}. Retrying in {:?}...",
                    attempt + 1,
                    e,
                    delay
                );

                sleep(delay).await;
                attempt += 1;
            }
        }
    }
}
