use std::{collections::HashMap, error::Error};

use aws_sdk_dynamodb::types::AttributeValue;
use futures::future::join_all;

use crate::model::{Bet, User};

const USERS_TABLE_NAME: &str = "betting-users";
const BETS_TABLE_NAME: &str = "betting-bets";

async fn scan_table(
    client: &aws_sdk_dynamodb::Client,
    table_name: &str,
) -> Result<Vec<HashMap<String, AttributeValue>>, Box<dyn Error>> {
    const TOTAL_SEGMENTS: i32 = 8;

    let all_items: Vec<HashMap<String, AttributeValue>> = join_all(
        (0..TOTAL_SEGMENTS)
            .map(|num| async move {
                client
                    .scan()
                    .table_name(table_name)
                    .segment(num)
                    .total_segments(TOTAL_SEGMENTS)
                    .into_paginator()
                    .send()
                    .collect::<Result<Vec<_>, _>>()
                    .await
                    .unwrap()
                    .into_iter()
                    .flat_map(|scan_output| scan_output.items.into_iter())
                    .flatten()
                    .collect::<Vec<HashMap<String, AttributeValue>>>()
            })
            .collect::<Vec<_>>(),
    )
    .await
    .into_iter()
    .flatten()
    .collect();

    Ok(all_items)
}

pub async fn list_users(client: &aws_sdk_dynamodb::Client) -> Vec<User> {
    serde_dynamo::from_items(scan_table(client, USERS_TABLE_NAME).await.unwrap()).unwrap()
}

pub async fn get_user(client: &aws_sdk_dynamodb::Client, user_id: &str) -> Option<User> {
    client
        .get_item()
        .table_name(USERS_TABLE_NAME)
        .key("id", AttributeValue::S(user_id.to_string()))
        .send()
        .await
        .unwrap()
        .item
        .map(|item| serde_dynamo::from_item(item).unwrap())
}

pub async fn set_user_money(client: &aws_sdk_dynamodb::Client, user_id: &str, amount: f64) {
    client
        .update_item()
        .table_name(USERS_TABLE_NAME)
        .key("id", AttributeValue::S(user_id.to_string()))
        .update_expression("SET money = :val")
        .expression_attribute_values(":val", AttributeValue::N(amount.to_string()))
        .send()
        .await
        .unwrap();
}

pub async fn add_user_money(client: &aws_sdk_dynamodb::Client, user_id: &str, amount: f64) {
    client
        .update_item()
        .table_name(USERS_TABLE_NAME)
        .key("id", AttributeValue::S(user_id.to_string()))
        .update_expression("SET money = money + :val")
        .expression_attribute_values(":val", AttributeValue::N(amount.to_string()))
        .send()
        .await
        .unwrap();
}

pub async fn list_bets(client: &aws_sdk_dynamodb::Client) -> Vec<Bet> {
    serde_dynamo::from_items(scan_table(client, BETS_TABLE_NAME).await.unwrap()).unwrap()
}

pub async fn get_bet(client: &aws_sdk_dynamodb::Client, bet_id: &str) -> Option<Bet> {
    client
        .get_item()
        .table_name(BETS_TABLE_NAME)
        .key("id", AttributeValue::S(bet_id.to_string()))
        .send()
        .await
        .unwrap()
        .item
        .map(|item| serde_dynamo::from_item(item).unwrap())
}

pub async fn put_bet(client: &aws_sdk_dynamodb::Client, bet: Bet) {
    client
        .put_item()
        .table_name(BETS_TABLE_NAME)
        .set_item(Some(serde_dynamo::to_item(bet).unwrap()))
        .send()
        .await
        .unwrap();
}

pub async fn delete_bet(client: &aws_sdk_dynamodb::Client, bet_id: &str) {
    client
        .delete_item()
        .table_name(BETS_TABLE_NAME)
        .key("id", AttributeValue::S(bet_id.to_string()))
        .send()
        .await
        .unwrap();
}
