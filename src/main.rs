use std::{
    collections::BTreeMap,
    time::{SystemTime, UNIX_EPOCH},
};

use axum::{
    debug_handler,
    extract::{FromRef, State},
    http::{header, StatusCode},
    response::{Html, IntoResponse, Redirect, Response},
    routing::{get, post},
    Form, Router,
};
use axum_lambda_util::run_router;
use futures::future::join_all;
use log_util::init_default_debug_logger;
use login::login_page;
use model::{Bet, LogMessage, User, UserBet, YesOrNo, YesOrNoOrNA};
use secrets::Secrets;
use serde::{Deserialize, Serialize};
use sql_util::get_db_connection_pool;
use sqlx::{Pool, Postgres};
use tera::Tera;
use user_id_cookie::ExtractUserId;
use uuid::Uuid;

mod axum_lambda_util;
mod jwt;
mod leaderboard;
mod log_util;
mod login;
mod model;
mod secrets;
mod sql_util;
mod user_id_cookie;

#[derive(Serialize)]
struct DashboardBetInfo {
    bet_id: String,
    name: String,
    creator_id: String,
    creator_name: String,
    created_seconds_since_epoch: usize,
    yes_pool: f64,
    no_pool: f64,
    probability_of_yes: f64,
    user_yes: Option<UserBet>,
    user_no: Option<UserBet>,
    closed: bool,
}
async fn dashboard(
    ExtractUserId(user_id): ExtractUserId,
    State(app_state): State<AppState>,
) -> impl IntoResponse {
    let logs = LogMessage::list(&app_state.pool).await;

    let mut bets = Bet::list(&app_state.pool).await;

    bets.sort_by(|a, b| {
        a.created_seconds_since_epoch
            .cmp(&b.created_seconds_since_epoch)
            .reverse()
    });

    let mut context = tera::Context::new();

    let users = User::list(&app_state.pool)
        .await
        .into_iter()
        .map(|user| (user.id.clone(), user))
        .collect::<BTreeMap<String, User>>();

    let user_bets = UserBet::list(&app_state.pool).await;

    let mut processed_bets = Vec::new();

    for bet in bets {
        let creator = users
            .get(&bet.creator_id)
            .expect("Bet can't be created by nonexistent user");

        let processed_bet = DashboardBetInfo {
            bet_id: bet.id.clone(),
            name: bet.name.clone(),
            creator_id: bet.creator_id.clone(),
            creator_name: creator.name.clone(),
            created_seconds_since_epoch: bet.created_seconds_since_epoch,
            yes_pool: bet.yes_pool,
            no_pool: bet.no_pool,
            probability_of_yes: bet.no_pool / (bet.yes_pool + bet.no_pool),
            user_yes: user_bets
                .iter()
                .find(|user_bet| {
                    user_bet.is_yes && user_bet.bet_id == bet.id && user_bet.user_id == user_id
                })
                .cloned(),
            user_no: user_bets
                .iter()
                .find(|user_bet| {
                    !user_bet.is_yes && user_bet.bet_id == bet.id && user_bet.user_id == user_id
                })
                .cloned(),
            closed: bet.closed,
        };

        processed_bets.push(processed_bet);
    }

    context.insert("bets", &processed_bets);
    context.insert(
        "user",
        users
            .get(&user_id)
            .expect("Can't be logged in if user doesn't exist"),
    );
    context.insert(
        "logs",
        &logs
            .into_iter()
            .map(|log| (log.created_at.timestamp(), log.content))
            .collect::<Vec<_>>(),
    );

    Html(app_state.engine.render("dashboard", &context).unwrap())
}

fn share_price(amount: usize, which: &YesOrNo, yes_pool: f64, no_pool: f64) -> Result<f64, ()> {
    // For NO: X^2+(YES+NO-N)*X-N*YES=0
    // For YES: X^2+(YES+NO-N)*X-N*NO=0
    let a = 1.0;
    let b = yes_pool + no_pool - amount as f64;
    let c = -(amount as f64)
        * match which {
            YesOrNo::Yes => no_pool,
            YesOrNo::No => yes_pool,
        };

    let sqrt = (b.powi(2) - (4.0 * a * c)).sqrt();

    if sqrt.is_nan() {
        Err(())
    } else {
        let price = (-b + sqrt) / (2.0 * a);

        // Round upwards to nearest 100 to make it unprofitable to exploit floating point integer bugs
        let price = (price * 100.0).ceil() / 100.0;

        Ok(price)
    }
}

#[derive(Deserialize)]
struct PlaceBetRequest {
    bet_id: String,
    amount: usize,
    which: YesOrNo,
}
#[debug_handler]
async fn place_bet(
    ExtractUserId(user_id): ExtractUserId,
    State(app_state): State<AppState>,
    Form(request): Form<PlaceBetRequest>,
) -> Response {
    let user = User::get_by_id(&app_state.pool, &user_id).await.unwrap();

    if request.amount > 0 {
        match Bet::get_by_id(&app_state.pool, &request.bet_id).await {
            Some(mut bet) => {
                if let Ok(spent) =
                    share_price(request.amount, &request.which, bet.yes_pool, bet.no_pool)
                {
                    if user.money >= spent {
                        if bet.closed {
                            StatusCode::BAD_REQUEST.into_response()
                        } else {
                            let is_yes = request.which.is_yes();
                            let mut user_bet =
                                UserBet::get(&app_state.pool, &user_id, &request.bet_id, is_yes)
                                    .await
                                    .unwrap_or(UserBet {
                                        user_id: user_id.clone(),
                                        bet_id: request.bet_id.clone(),
                                        is_yes,
                                        amount: 0,
                                        spent: 0.0,
                                    });

                            user_bet.amount += request.amount;
                            user_bet.spent += spent;

                            match request.which {
                                YesOrNo::Yes => {
                                    bet.yes_pool -= request.amount as f64;
                                }
                                YesOrNo::No => {
                                    bet.no_pool -= request.amount as f64;
                                }
                            };
                            bet.yes_pool += spent;
                            bet.no_pool += spent;

                            // All of the DB updates here
                            User::add_money(&app_state.pool, &user_id, -spent).await;
                            user_bet.update_or_insert(&app_state.pool).await;
                            Bet::update_pools(
                                &app_state.pool,
                                &request.bet_id,
                                bet.yes_pool,
                                bet.no_pool,
                            )
                            .await;

                            LogMessage::insert(
                                &app_state.pool,
                                &format!(
                                    "{} bought {} {} shares in \"{}\" for ${}",
                                    user.name, request.amount, request.which, bet.name, spent
                                ),
                            )
                            .await;

                            Redirect::to("/").into_response()
                        }
                    } else {
                        (StatusCode::BAD_REQUEST, "Not enough money").into_response()
                    }
                } else {
                    (
                        StatusCode::BAD_REQUEST,
                        "Bet was too big for such a small starting pool",
                    )
                        .into_response()
                }
            }
            None => StatusCode::NOT_FOUND.into_response(),
        }
    } else {
        (StatusCode::BAD_REQUEST, "Can't buy 0 shares").into_response()
    }
}

#[derive(Deserialize)]
struct CreateBetRequest {
    name: String,
    starting_money: usize,
}
async fn create_bet(
    ExtractUserId(user_id): ExtractUserId,
    State(app_state): State<AppState>,
    Form(request): Form<CreateBetRequest>,
) -> Response {
    let bet_id = Uuid::new_v4().to_string();

    let user = User::get_by_id(&app_state.pool, &user_id).await.unwrap();

    if request.starting_money < 20 {
        (
            StatusCode::BAD_REQUEST,
            "You need to put in at least $20 of starting money",
        )
            .into_response()
    } else if user.money >= request.starting_money as f64 {
        User::add_money(&app_state.pool, &user_id, -(request.starting_money as f64)).await;

        let now = SystemTime::now();
        let duration = now.duration_since(UNIX_EPOCH).expect("Time went backwards");
        let created_seconds_since_epoch = duration.as_secs() as usize;

        Bet {
            id: bet_id.clone(),
            creator_id: user_id.clone(),
            name: request.name.clone(),
            created_seconds_since_epoch,
            closed: false,
            yes_pool: request.starting_money as f64,
            no_pool: request.starting_money as f64,
        }
        .insert(&app_state.pool)
        .await;

        // When a user starts a bet, they use the money to buy equal amounts of yes shares and no shares
        // (price of yes share + price of no share = 1)
        // Those shares are not "owned" by the creator, but are instead used to provide liquidity
        UserBet {
            user_id: user_id.clone(),
            bet_id: bet_id.clone(),
            is_yes: true,
            amount: 0,
            spent: request.starting_money as f64 / 2.0,
        }
        .insert(&app_state.pool)
        .await;
        UserBet {
            user_id: user_id.clone(),
            bet_id: bet_id.clone(),
            is_yes: false,
            amount: 0,
            spent: request.starting_money as f64 / 2.0,
        }
        .insert(&app_state.pool)
        .await;

        LogMessage::insert(
            &app_state.pool,
            &format!(
                "{} created a new market, \"{}\", with a starting pool of {}",
                user.name, &request.name, request.starting_money
            ),
        )
        .await;

        Redirect::to("/").into_response()
    } else {
        (
            StatusCode::BAD_REQUEST,
            "You don't have enough money to create this bet",
        )
            .into_response()
    }
}

#[derive(Deserialize)]
struct CloseBetRequest {
    bet_id: String,
}
async fn close_bet(
    ExtractUserId(user_id): ExtractUserId,
    State(app_state): State<AppState>,
    Form(request): Form<CloseBetRequest>,
) -> Response {
    let bet = Bet::get_by_id(&app_state.pool, &request.bet_id).await;

    match bet {
        Some(bet) => {
            if bet.creator_id == user_id {
                let user = User::get_by_id(&app_state.pool, &user_id).await.unwrap();

                Bet::close(&app_state.pool, &bet.id).await;

                LogMessage::insert(
                    &app_state.pool,
                    &format!("{} closed the market \"{}\"", user.name, bet.name),
                )
                .await;

                Redirect::to("/").into_response()
            } else {
                StatusCode::NOT_FOUND.into_response()
            }
        }
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

#[derive(Deserialize)]
struct ResolveBetRequest {
    bet_id: String,
    which: YesOrNoOrNA,
}
async fn resolve_bet(
    ExtractUserId(user_id): ExtractUserId,
    State(app_state): State<AppState>,
    Form(request): Form<ResolveBetRequest>,
) -> Response {
    let bet = Bet::get_by_id(&app_state.pool, &request.bet_id).await;

    match bet {
        Some(bet) => {
            if bet.creator_id == user_id {
                let user = User::get_by_id(&app_state.pool, &user_id).await.unwrap();
                let user_bets = UserBet::get_by_bet_id(&app_state.pool, &request.bet_id).await;

                Bet::delete(&app_state.pool, &request.bet_id).await;

                match request.which {
                    YesOrNoOrNA::Yes => {
                        User::add_money(&app_state.pool, &bet.creator_id, bet.yes_pool).await;
                        join_all(user_bets.iter().filter(|user_bet| user_bet.is_yes).map(
                            |user_bet| {
                                User::add_money(
                                    &app_state.pool,
                                    &user_bet.user_id,
                                    user_bet.amount as f64,
                                )
                            },
                        ))
                        .await;
                    }
                    YesOrNoOrNA::No => {
                        User::add_money(&app_state.pool, &bet.creator_id, bet.no_pool).await;
                        join_all(user_bets.iter().filter(|user_bet| !user_bet.is_yes).map(
                            |user_bet| {
                                User::add_money(
                                    &app_state.pool,
                                    &user_bet.user_id,
                                    user_bet.amount as f64,
                                )
                            },
                        ))
                        .await;
                    }
                    YesOrNoOrNA::NA => {
                        join_all(user_bets.iter().map(|user_bet| {
                            User::add_money(&app_state.pool, &user_bet.user_id, user_bet.spent)
                        }))
                        .await;
                    }
                }

                LogMessage::insert(
                    &app_state.pool,
                    &format!(
                        "{} resolved the market \"{}\" with a result of {}",
                        user.name, bet.name, request.which
                    ),
                )
                .await;

                Redirect::to("/").into_response()
            } else {
                StatusCode::NOT_FOUND.into_response()
            }
        }
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn give_money(ExtractUserId(user_id): ExtractUserId, State(app_state): State<AppState>) {
    let users = User::list(&app_state.pool).await;
    if user_id
        == users
            .iter()
            .find(|user| user.name == "Jefferson")
            .unwrap()
            .id
    {
        for user in users {
            User::add_money(&app_state.pool, &user.id, 100.0).await
        }

        LogMessage::insert(
            &app_state.pool,
            "$100 has been added to everybody's account",
        )
        .await;
    }
}

pub async fn changelog(
    ExtractUserId(_user_id): ExtractUserId,
    State(app_state): State<AppState>,
) -> impl IntoResponse {
    Html(
        app_state
            .engine
            .render("changelog", &tera::Context::new())
            .unwrap(),
    )
}

pub async fn about(
    ExtractUserId(_user_id): ExtractUserId,
    State(app_state): State<AppState>,
) -> impl IntoResponse {
    Html(
        app_state
            .engine
            .render("about", &tera::Context::new())
            .unwrap(),
    )
}

type AppEngine = Tera;

// Define your application shared state
#[derive(Clone, FromRef)]
pub struct AppState {
    engine: AppEngine,
    secret: String,
    pool: Pool<Postgres>,
}

#[tokio::main]
async fn main() {
    init_default_debug_logger();

    let env = Secrets::load().await;

    let pool = get_db_connection_pool(&env.db_username, &env.db_password)
        .await
        .unwrap();

    // Set up the Handlebars engine with the same route paths as the Axum router
    let mut hbs = Tera::default();
    hbs.add_raw_templates(vec![
        ("base", include_str!("../data/base.tera")),
        ("dashboard", include_str!("../data/dashboard.tera")),
        ("login", include_str!("../data/login.tera")),
        ("leaderboard", include_str!("../data/leaderboard.tera")),
        ("changelog", include_str!("../data/changelog.tera")),
        ("about", include_str!("../data/about.tera")),
    ])
    .unwrap();

    let app = Router::new()
        .route("/", get(dashboard))
        .route("/leaderboard", get(leaderboard::leaderboard))
        .route("/changelog", get(changelog))
        .route("/about", get(about))
        .route("/login", get(login_page).post(login::login))
        .route("/place", post(place_bet))
        .route("/create", post(create_bet))
        .route("/close", post(close_bet))
        .route("/resolve", post(resolve_bet))
        .route("/give_money", post(give_money))
        .route(
            "/favicon.png",
            get(|| async {
                (
                    [(header::CONTENT_TYPE, "image/png")],
                    include_bytes!("../data/favicon.png"),
                )
            }),
        )
        .with_state(AppState {
            engine: hbs,
            secret: env.auth_secret,
            pool,
        });

    run_router(app).await;
}
