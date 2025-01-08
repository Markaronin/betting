use axum::{
    extract::State,
    response::{Html, IntoResponse},
};
use serde::Serialize;

use crate::{
    model::{Bet, User, UserBet},
    user_id_cookie::ExtractUserId,
    AppState,
};

#[derive(Debug, Serialize)]
struct LeaderboardEntry {
    name: String,
    liquid_money: f64,
    expected_money: f64,
    max_money: f64,
}

pub async fn leaderboard(
    ExtractUserId(_user_id): ExtractUserId,
    State(app_state): State<AppState>,
) -> impl IntoResponse {
    let bets = Bet::list(&app_state.pool).await;

    let users = User::list(&app_state.pool).await;

    let user_bets = UserBet::list(&app_state.pool).await;

    let mut context = tera::Context::new();

    let mut leaderboard_entries = vec![];

    for user in users {
        leaderboard_entries.push(LeaderboardEntry {
            name: user.name,
            liquid_money: user.money,
            expected_money: user.money
                + bets
                    .iter()
                    .map(|bet| {
                        let probability_of_yes = bet.no_pool / (bet.yes_pool + bet.no_pool);

                        let possible_starting_pool_money = if bet.creator_id == user.id {
                            (bet.yes_pool * probability_of_yes)
                                + (bet.no_pool * (1.0 - probability_of_yes))
                        } else {
                            0.0
                        };

                        let yes_bet_money = user_bets
                            .iter()
                            .find(|user_bet| {
                                user_bet.is_yes
                                    && user_bet.user_id == user.id
                                    && user_bet.bet_id == bet.id
                            })
                            .map(|yes_bet| yes_bet.amount as f64 * probability_of_yes)
                            .unwrap_or(0.0);

                        let no_bet_money = user_bets
                            .iter()
                            .find(|user_bet| {
                                !user_bet.is_yes
                                    && user_bet.user_id == user.id
                                    && user_bet.bet_id == bet.id
                            })
                            .map(|no_bet| no_bet.amount as f64 * (1.0 - probability_of_yes))
                            .unwrap_or(0.0);

                        yes_bet_money + no_bet_money + possible_starting_pool_money
                    })
                    .sum::<f64>(),
            max_money: user.money
                + bets
                    .iter()
                    .map(|bet| {
                        let yes_amount = user_bets
                            .iter()
                            .find(|user_bet| {
                                user_bet.is_yes
                                    && user_bet.user_id == user.id
                                    && user_bet.bet_id == bet.id
                            })
                            .map(|yes_bet| {
                                yes_bet.amount as f64
                                    + if bet.creator_id == user.id {
                                        bet.yes_pool
                                    } else {
                                        0.0
                                    }
                            })
                            .unwrap_or(0.0);

                        let no_amount = user_bets
                            .iter()
                            .find(|user_bet| {
                                !user_bet.is_yes
                                    && user_bet.user_id == user.id
                                    && user_bet.bet_id == bet.id
                            })
                            .map(|no_bet| {
                                no_bet.amount as f64
                                    + if bet.creator_id == user.id {
                                        bet.no_pool
                                    } else {
                                        0.0
                                    }
                            })
                            .unwrap_or(0.0);
                        yes_amount.max(no_amount)
                    })
                    .sum::<f64>(),
        })
    }

    context.insert("leaderboard_entries", &leaderboard_entries);

    Html(app_state.engine.render("leaderboard", &context).unwrap())
}
