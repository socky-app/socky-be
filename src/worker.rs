use crate::repository::{auth_repo::AuthRepository, RepositoryManager};
use std::time::Duration;
use tracing::{error, info};

const TOKEN_CLEANUP_INTERVAL_SECS: u64 = 60 * 60 * 6; // 6 hours
const TOKEN_CLEANUP_BATCH_SIZE: u64 = 1000;

pub fn spawn_token_cleanup_worker(rm: RepositoryManager) {
    tokio::spawn(async move {
        // Run this check every 6 hours
        let mut interval = tokio::time::interval(Duration::from_secs(TOKEN_CLEANUP_INTERVAL_SECS));

        // Skip the immediate first tick so it doesn't run during server boot
        interval.tick().await;

        loop {
            interval.tick().await;
            info!("Starting background cleanup of expired tokens");

            let mut total_deleted = 0;

            loop {
                match AuthRepository::delete_expired_token_batch(&rm, TOKEN_CLEANUP_BATCH_SIZE)
                    .await
                {
                    Ok(deleted_count) => {
                        total_deleted += deleted_count;

                        // If we deleted less than the batch size, we've processed everything
                        if deleted_count < TOKEN_CLEANUP_BATCH_SIZE {
                            break;
                        }

                        // Yield the thread briefly so we don't monopolize CPU
                        // or DB connections during a massive cleanup run.
                        tokio::time::sleep(Duration::from_millis(100)).await;
                    }
                    Err(e) => {
                        error!("Failed to clean up token batch: {}", e);
                        break; // Stop trying this cycle, try again in 6 hours
                    }
                }
            }

            if total_deleted > 0 {
                info!("Successfully deleted {} expired tokens.", total_deleted);
            } else {
                info!("No expired tokens found.");
            }
        }
    });
}
