pub mod authz;
pub mod blob;
pub mod csa;
pub mod messages;
pub mod mt;
pub mod outlook;
pub mod presence;

use std::time::Duration;

use reqwest::{Client, RequestBuilder, Response, StatusCode};

use crate::config::NetworkConfig;
use crate::error::{Result, TeamsError};

pub struct HttpClient {
    pub client: Client,
    pub network: NetworkConfig,
}

fn truncate_body(body: String, max_len: usize) -> String {
    if body.len() > max_len {
        format!("{}... (truncated)", &body[..max_len])
    } else {
        body
    }
}

impl HttpClient {
    pub fn new(network: &NetworkConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(network.timeout))
            .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Teams/24315.1101.3006.7571 Chrome/130.0.6723.191 Electron/33.3.1 Safari/537.36")
            .build()
            .expect("failed to build HTTP client");

        Self {
            client,
            network: network.clone(),
        }
    }

    pub async fn execute_with_retry(
        &self,
        build_request: impl Fn() -> RequestBuilder,
    ) -> Result<Response> {
        let max_attempts = self.network.max_retries + 1;
        let mut last_error = None;

        for attempt in 0..max_attempts {
            if attempt > 0 {
                let delay = Duration::from_secs(
                    self.network.retry_backoff_base.saturating_pow(attempt - 1),
                );
                let delay = delay.min(Duration::from_secs(30));
                tracing::debug!("retry attempt {attempt}, waiting {delay:?}");
                tokio::time::sleep(delay).await;
            }

            let resp = match build_request().send().await {
                Ok(resp) => resp,
                Err(e) => {
                    tracing::warn!("request failed (attempt {attempt}): {e}");
                    last_error = Some(TeamsError::NetworkError(e));
                    continue;
                }
            };

            let status = resp.status();

            match status {
                s if s.is_success() => return Ok(resp),

                StatusCode::UNAUTHORIZED => {
                    return Err(TeamsError::ApiError {
                        status: 401,
                        message: truncate_body(resp.text().await.unwrap_or_default(), 500),
                    });
                }

                StatusCode::FORBIDDEN => {
                    return Err(TeamsError::PermissionDenied(truncate_body(
                        resp.text().await.unwrap_or_default(),
                        500,
                    )));
                }

                StatusCode::NOT_FOUND => {
                    return Err(TeamsError::NotFound(truncate_body(
                        resp.text().await.unwrap_or_default(),
                        500,
                    )));
                }

                StatusCode::TOO_MANY_REQUESTS => {
                    let retry_after = resp
                        .headers()
                        .get("retry-after")
                        .and_then(|v| v.to_str().ok())
                        .and_then(|v| v.parse::<u64>().ok())
                        .unwrap_or(10);

                    if attempt + 1 < max_attempts {
                        tracing::warn!("rate limited, waiting {retry_after}s");
                        tokio::time::sleep(Duration::from_secs(retry_after)).await;
                        continue;
                    }

                    return Err(TeamsError::RateLimited {
                        retry_after_secs: retry_after,
                    });
                }

                s if s.is_server_error() => {
                    let body = truncate_body(resp.text().await.unwrap_or_default(), 500);
                    tracing::warn!("server error {s} (attempt {attempt}): {body}");
                    last_error = Some(TeamsError::ServerError {
                        status: s.as_u16(),
                        message: body,
                    });
                    continue;
                }

                s => {
                    return Err(TeamsError::ApiError {
                        status: s.as_u16(),
                        message: truncate_body(resp.text().await.unwrap_or_default(), 500),
                    });
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            TeamsError::Other(anyhow::anyhow!("request failed after all retries"))
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_body_shorter_than_max() {
        let result = truncate_body("hello".to_string(), 10);
        assert_eq!(result, "hello");
    }

    #[test]
    fn truncate_body_exactly_at_max() {
        let result = truncate_body("1234567890".to_string(), 10);
        assert_eq!(result, "1234567890");
    }

    #[test]
    fn truncate_body_longer_than_max() {
        let result = truncate_body("hello world, this is a long string".to_string(), 5);
        assert_eq!(result, "hello... (truncated)");
    }

    #[test]
    fn truncate_body_empty_string() {
        let result = truncate_body(String::new(), 10);
        assert_eq!(result, "");
    }
}
