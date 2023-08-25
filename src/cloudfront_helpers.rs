use aws_sdk_cloudfront::operation::create_invalidation::CreateInvalidationError;
use aws_sdk_cloudfront::operation::get_invalidation::GetInvalidationError;
use aws_sdk_cloudfront::Client;
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};
use aws_sdk_cloudfront::error::SdkError;
use aws_sdk_cloudfront::types::{InvalidationBatch, Paths};
use tracing::info;

///
/// Invalidates all objects in the distribution. This call does not wait for the invalidation to
/// complete.
///
pub async fn invalidate_distribution(
    distribution_id: &String,
    cf_client: &Client,
) -> Result<String, SdkError<CreateInvalidationError>> {
    // Use current unix time as our invalidation reference
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("adding times together should produce a time")
        .as_millis()
        .to_string();

    let invalidation_paths = Paths::builder().items("/*").quantity(1).build();

    let invalidation_batch = InvalidationBatch::builder()
        .caller_reference(now)
        .paths(invalidation_paths)
        .build();

    let invalidation = cf_client
        .create_invalidation()
        .distribution_id(distribution_id)
        .invalidation_batch(invalidation_batch)
        .send()
        .await?;

    return Ok(invalidation
        .invalidation()
        .expect("invalidation body should be present")
        .id()
        .expect("invalidation should have an ID")
        .to_string());
}

///
/// Waits indefinitely for the given invalidation to complete. This call should likely be wrapped
/// in a timeout so we don't wait indefinitely.
///
pub async fn wait_for_invalidation(
    invalidation_id: &String,
    distribution_id: &String,
    cf_client: &Client,
) -> Result<(), SdkError<GetInvalidationError>> {
    info!("Waiting for invalidation to complete");
    loop {
        let invalidation_output = cf_client
            .get_invalidation()
            .distribution_id(distribution_id)
            .id(invalidation_id)
            .send()
            .await?;

        let status = invalidation_output
            .invalidation()
            .expect("invalidation body should be present")
            .status()
            .expect("invalidation should have a status");

        info!(status = status, "Invalidation");

        match status {
            // TODO - handle other states in here
            "Completed" => {
                return Ok(());
            }
            _s => {}
        };

        tokio::time::sleep(Duration::from_secs(10)).await;
    }
}
