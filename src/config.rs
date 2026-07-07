use crate::error::{AppError, AppResult};
use crate::models::AuthContext;
use std::env;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub ones_base_url: String,
    pub team_uuid: String,
}

impl AppConfig {
    pub fn from_env() -> AppResult<Self> {
        let base = env::var("ONES_BASE_URL")
            .or_else(|_| env::var("ONES_API_URL"))
            .map_err(|_| AppError::MissingConfig("ONES_BASE_URL"))?;
        let team_uuid =
            env::var("ONES_TEAM_UUID").map_err(|_| AppError::MissingConfig("ONES_TEAM_UUID"))?;

        Ok(Self {
            ones_base_url: normalize_base_url(&base),
            team_uuid,
        })
    }
}

#[derive(Debug, Clone)]
pub struct StdioAuthConfig {
    pub email: String,
    pub password: String,
}

impl StdioAuthConfig {
    pub fn from_env() -> AppResult<Self> {
        let email = env::var("ONES_EMAIL")
            .or_else(|_| env::var("ONES_USER_EMAIL"))
            .map_err(|_| AppError::MissingConfig("ONES_EMAIL"))?;
        let password = env::var("ONES_PASSWORD")
            .or_else(|_| env::var("ONES_USER_PASSWORD"))
            .map_err(|_| AppError::MissingConfig("ONES_PASSWORD"))?;
        Ok(Self { email, password })
    }

    pub fn into_auth_context(self) -> AuthContext {
        AuthContext {
            email: self.email,
            password: self.password,
        }
    }
}

fn normalize_base_url(input: &str) -> String {
    let trimmed = input.trim().trim_end_matches('/');
    trimmed
        .strip_suffix("/project/api")
        .unwrap_or(trimmed)
        .to_string()
}
