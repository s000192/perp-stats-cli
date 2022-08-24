use crate::error::*;
use ethers::prelude::U256;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub struct GraphClient {
    http_client: Client,
    query_endpoint: String,
}

#[derive(Debug)]
pub struct Position {
    pub id: U256,
    // pub index: U256,
    // pub user: Address,
    // pub marker: Address,
    // pub timestamp: SystemTime,
}

#[derive(Debug, PartialEq, Clone)]
pub struct TradingHistoryItem {
    pub id: U256,
    pub isLong: bool,
    pub price: U256,
    pub size: U256,
    pub underlying: String,
}

#[derive(Serialize, Deserialize)]
struct GraphQueryRequest<T> {
    query: String,
    variables: T,
}

#[derive(Debug, Serialize, Deserialize)]
struct GraphQueryResponse<T> {
    data: T,
}

#[derive(Serialize, Deserialize)]
struct GraphQueryVariables {
    skip: u32,
    first: u32,
}

#[derive(Serialize, Deserialize)]
struct PositionsGraphQueryResponseData {
    positions: Vec<RawPosition>,
}

#[derive(Serialize, Deserialize)]
struct RawPosition {
    id: String,
    // index: String,
    // user: Address,
    // marker: Address,
    // timestamp: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct TradingHistoryItemsGraphQueryResponseData {
    tradeHistoryItems: Vec<RawTradingHistoryItem>,
}

#[derive(Debug, Serialize, Deserialize)]
struct RawTradingHistoryItem {
    id: String,
    isLong: bool,
    price: String,
    size: String,
    underlying: String,
}

const GRAPHQL_BATCH_SIZE: u32 = 500;
const GRAPHQL_TIMEOUT: Duration = Duration::from_secs(5);

impl GraphClient {
    pub fn new(query_endpoint: String) -> Self {
        GraphClient {
            http_client: Client::new(),
            query_endpoint,
        }
    }

    pub async fn get_trading_history_items(&self) -> Result<Vec<TradingHistoryItem>, GraphqlError> {
        let mut all_items: Vec<TradingHistoryItem> = vec![];

        let query_str = include_str!("./queries/trading_history_items_query.graphql");

        loop {
            let query = GraphQueryRequest {
                query: String::from(query_str),
                variables: GraphQueryVariables {
                    skip: all_items.len() as u32,
                    first: GRAPHQL_BATCH_SIZE,
                },
            };

            let res = self
                .http_client
                .post(&self.query_endpoint)
                .timeout(GRAPHQL_TIMEOUT)
                .header("Content-Type", "application/json")
                .body(serde_json::to_string(&query).map_err(GraphqlError::SerializationError)?)
                .send()
                .await
                .map_err(GraphqlError::NetworkError)?;

            let body: String = res.text().await.map_err(GraphqlError::NetworkError)?;
            let result: GraphQueryResponse<TradingHistoryItemsGraphQueryResponseData> =
                serde_json::from_str(&body).map_err(GraphqlError::SerializationError)?;

            for raw_trading_history_item in result.data.tradeHistoryItems.iter() {
                all_items.push(TradingHistoryItem {
                    id: U256::from_dec_str(&raw_trading_history_item.id).map_err(|_| {
                        GraphqlError::InvalidId(raw_trading_history_item.id.to_owned())
                    })?,
                    isLong: raw_trading_history_item.isLong,
                    price: U256::from_dec_str(&raw_trading_history_item.price).map_err(|_| {
                        GraphqlError::InvalidId(raw_trading_history_item.price.to_owned())
                    })?,
                    size: U256::from_dec_str(&raw_trading_history_item.size).map_err(|_| {
                        GraphqlError::InvalidId(raw_trading_history_item.size.to_owned())
                    })?,
                    underlying: raw_trading_history_item.underlying.clone(),
                });
            }

            if (result.data.tradeHistoryItems.len() as u32) < GRAPHQL_BATCH_SIZE {
                break;
            }
        }

        Ok(all_items)
    }
}
