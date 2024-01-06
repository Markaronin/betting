use std::collections::BTreeMap;
use std::fmt::Display;

use serde::{Deserialize, Serialize};
use sqlx::types::chrono::{DateTime, Utc};
use sqlx::{FromRow, Pool, Postgres};

#[derive(Deserialize)]
pub enum YesOrNo {
    Yes,
    No,
}
impl Display for YesOrNo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            YesOrNo::Yes => f.write_str("yes"),
            YesOrNo::No => f.write_str("no"),
        }
    }
}

#[derive(Deserialize)]
pub enum YesOrNoOrNA {
    Yes,
    No,
    NA,
}
impl Display for YesOrNoOrNA {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            YesOrNoOrNA::Yes => f.write_str("yes"),
            YesOrNoOrNA::No => f.write_str("no"),
            YesOrNoOrNA::NA => f.write_str("N/A"),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct User {
    pub id: String,
    pub name: String,
    pub money: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserBet {
    pub amount: usize,
    pub spent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bet {
    pub id: String,
    pub creator_id: String,
    pub created_seconds_since_epoch: Option<usize>,
    pub name: String,
    pub closed: bool,
    pub yes_pool: f64,
    pub no_pool: f64,
    pub yes_bets: BTreeMap<String, UserBet>,
    pub no_bets: BTreeMap<String, UserBet>,
}

#[derive(Debug, Clone, FromRow)]
pub struct LogMessage {
    pub created_at: DateTime<Utc>,
    pub content: String,
}
impl LogMessage {
    pub async fn insert(pool: &Pool<Postgres>, content: &str) -> Result<(), sqlx::Error> {
        let mut conn = pool.acquire().await?;

        sqlx::query("insert into betting_logs (content) values ($1)")
            .bind(content)
            .execute(&mut *conn)
            .await?;

        Ok(())
    }
}
