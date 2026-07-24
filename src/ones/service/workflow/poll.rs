use super::super::OnesService;
use crate::{
    error::{AppError, AppResult},
    models::AuthContext,
};
use tokio::time::{Duration, sleep, timeout};

const POLL_ATTEMPTS: usize = 30;
const POLL_TIMEOUT: Duration = Duration::from_secs(30);

pub(super) async fn wait_for_transition(
    service: &OnesService,
    auth: &AuthContext,
    async_token: &str,
    async_compute_token: Option<&str>,
) -> AppResult<()> {
    timeout(
        POLL_TIMEOUT,
        poll_until_complete(service, auth, async_token, async_compute_token),
    )
    .await
    .map_err(|_| AppError::invalid_upstream("transition async operation timed out"))?
}

async fn poll_until_complete(
    service: &OnesService,
    auth: &AuthContext,
    async_token: &str,
    async_compute_token: Option<&str>,
) -> AppResult<()> {
    for attempt in 0..POLL_ATTEMPTS {
        let response = service
            .client
            .async_status(auth, async_token, async_compute_token)
            .await?;
        if let Some(error) = response
            .results
            .values()
            .find_map(|result| result.error.clone())
        {
            return Err(AppError::invalid_upstream(format!(
                "transition failed: {error}"
            )));
        }
        if let Some(status) = response.results.values().find_map(|result| {
            matches!(result.status.as_str(), "failed" | "error" | "cancelled")
                .then_some(result.status.as_str())
        }) {
            return Err(AppError::invalid_upstream(format!(
                "transition failed with async status {status}"
            )));
        }
        let done = !response.results.is_empty()
            && response
                .results
                .values()
                .all(|result| result.status == "done");
        if done {
            return Ok(());
        }
        if attempt + 1 < POLL_ATTEMPTS {
            sleep(Duration::from_millis(100)).await;
        }
    }
    Err(AppError::invalid_upstream(
        "transition async operation timed out",
    ))
}
