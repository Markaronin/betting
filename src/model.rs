use std::fmt::Display;

use serde::{Deserialize, Serialize};
use sqlx::types::chrono::{DateTime, Utc};
use sqlx::{FromRow, Pool, Postgres, Transaction};

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
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
impl YesOrNo {
    pub fn is_yes(&self) -> bool {
        match self {
            YesOrNo::Yes => true,
            YesOrNo::No => false,
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

#[derive(Debug, Deserialize, Serialize, FromRow)]
pub struct User {
    pub id: String,
    pub name: String,
    pub money: f64,
}
impl User {
    pub async fn list(pool: &Pool<Postgres>) -> Vec<Self> {
        sqlx::query_as("SELECT * FROM betting.users")
            .fetch_all(pool)
            .await
            .unwrap()
    }
    pub async fn list_for_update(transaction: &mut Transaction<'_, Postgres>) -> Vec<Self> {
        sqlx::query_as("SELECT * FROM betting.users FOR UPDATE")
            .fetch_all(&mut **transaction)
            .await
            .unwrap()
    }
    pub async fn get_by_id(pool: &Pool<Postgres>, id: &str) -> Option<Self> {
        sqlx::query_as("SELECT * FROM betting.users WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await
            .unwrap()
    }
    pub async fn get_for_update_by_id(
        transaction: &mut Transaction<'_, Postgres>,
        id: &str,
    ) -> Option<Self> {
        sqlx::query_as("SELECT * FROM betting.users WHERE id = $1 FOR UPDATE")
            .bind(id)
            .fetch_optional(&mut **transaction)
            .await
            .unwrap()
    }
    pub async fn add_money(transaction: &mut Transaction<'_, Postgres>, id: &str, new_money: f64) {
        sqlx::query("UPDATE betting.users SET money = money + $1 WHERE id = $2")
            .bind(new_money)
            .bind(id)
            .execute(&mut **transaction)
            .await
            .unwrap();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserBet {
    pub user_id: String,
    pub bet_id: String,
    pub is_yes: bool,
    #[sqlx(try_from = "i32")]
    pub amount: usize,
    pub spent: f64,
}
impl UserBet {
    pub async fn list(pool: &Pool<Postgres>) -> Vec<Self> {
        sqlx::query_as("SELECT * FROM betting.user_bets")
            .fetch_all(pool)
            .await
            .unwrap()
    }
    pub async fn get_for_update(
        transaction: &mut Transaction<'_, Postgres>,
        user_id: &str,
        bet_id: &str,
        is_yes: bool,
    ) -> Option<Self> {
        sqlx::query_as(
            "SELECT * FROM betting.user_bets WHERE user_id = $1 AND bet_id = $2 AND is_yes = $3 FOR UPDATE",
        )
        .bind(user_id)
        .bind(bet_id)
        .bind(is_yes)
        .fetch_optional(&mut **transaction)
        .await
        .unwrap()
    }
    pub async fn insert(self, transaction: &mut Transaction<'_, Postgres>) {
        sqlx::query("INSERT INTO betting.user_bets (user_id, bet_id, is_yes, amount, spent) VALUES ($1, $2, $3, $4, $5)")
            .bind(self.user_id)
            .bind(self.bet_id)
            .bind(self.is_yes)
            .bind(self.amount as i64)
            .bind(self.spent)
            .execute(&mut **transaction)
            .await
            .unwrap();
    }
    pub async fn update_or_insert(self, transaction: &mut Transaction<'_, Postgres>) {
        sqlx::query("INSERT INTO betting.user_bets (user_id, bet_id, is_yes, amount, spent) VALUES ($1, $2, $3, $4, $5) ON CONFLICT (user_id, bet_id, is_yes) DO UPDATE SET amount = $4, spent = $5")
            .bind(self.user_id)
            .bind(self.bet_id)
            .bind(self.is_yes)
            .bind(self.amount as i64)
            .bind(self.spent)
            .execute(&mut **transaction)
            .await
            .unwrap();
    }

    pub async fn get_for_update_by_bet_id(
        transaction: &mut Transaction<'_, Postgres>,
        bet_id: &str,
    ) -> Vec<UserBet> {
        sqlx::query_as("SELECT * FROM betting.user_bets WHERE bet_id = $1")
            .bind(bet_id)
            .fetch_all(&mut **transaction)
            .await
            .unwrap()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Bet {
    pub id: String,
    pub creator_id: String,
    #[sqlx(try_from = "i32")]
    pub created_seconds_since_epoch: usize,
    pub name: String,
    pub closed: bool,
    pub yes_pool: f64,
    pub no_pool: f64,
}
impl Bet {
    pub async fn list(pool: &Pool<Postgres>) -> Vec<Self> {
        sqlx::query_as("SELECT * FROM betting.bets")
            .fetch_all(pool)
            .await
            .unwrap()
    }
    pub async fn get_for_update_by_id(
        transaction: &mut Transaction<'_, Postgres>,
        id: &str,
    ) -> Option<Self> {
        sqlx::query_as("SELECT * FROM betting.bets WHERE id = $1 FOR UPDATE")
            .bind(id)
            .fetch_optional(&mut **transaction)
            .await
            .unwrap()
    }
    pub async fn insert(self, transaction: &mut Transaction<'_, Postgres>) {
        sqlx::query("INSERT INTO betting.bets (id, creator_id, created_seconds_since_epoch, name, closed, yes_pool, no_pool) VALUES ($1, $2, $3, $4, $5, $6, $7)")
            .bind(self.id)
            .bind(self.creator_id)
            .bind(self.created_seconds_since_epoch as i64)
            .bind(self.name)
            .bind(self.closed)
            .bind(self.yes_pool)
            .bind(self.no_pool)
            .execute(&mut **transaction)
            .await
            .unwrap();
    }
    pub async fn delete(transaction: &mut Transaction<'_, Postgres>, id: &str) {
        sqlx::query("DELETE FROM betting.bets WHERE id = $1")
            .bind(id)
            .execute(&mut **transaction)
            .await
            .unwrap();
    }
    pub async fn close(transaction: &mut Transaction<'_, Postgres>, id: &str) {
        sqlx::query("UPDATE betting.bets SET closed = true WHERE id = $1")
            .bind(id)
            .execute(&mut **transaction)
            .await
            .unwrap();
    }
    pub async fn update_pools(
        transaction: &mut Transaction<'_, Postgres>,
        id: &str,
        yes_pool: f64,
        no_pool: f64,
    ) {
        sqlx::query("UPDATE betting.bets SET yes_pool = $1, no_pool = $2 WHERE id = $3")
            .bind(yes_pool)
            .bind(no_pool)
            .bind(id)
            .execute(&mut **transaction)
            .await
            .unwrap();
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct LogMessage {
    pub created_at: DateTime<Utc>,
    pub content: String,
}
impl LogMessage {
    pub async fn list(pool: &Pool<Postgres>) -> Vec<Self> {
        sqlx::query_as("SELECT * FROM betting.logs order by created_at DESC LIMIT 100")
            .fetch_all(pool)
            .await
            .unwrap()
    }
    pub async fn insert(pool: &Pool<Postgres>, content: &str) {
        sqlx::query("INSERT INTO betting.logs (content) VALUES ($1)")
            .bind(content)
            .execute(pool)
            .await
            .unwrap();
    }
}
