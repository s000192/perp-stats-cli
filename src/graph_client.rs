use crate::error::*;
use ethers::prelude::I256;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub struct GraphClient {
    http_client: Client,
    perp_ui_query_endpoint: String,
    legacy_band_query_endpoint: String,
}

#[derive(Debug, PartialEq, Clone)]
pub struct TradingHistoryItem {
    pub id: I256,
    pub isLong: bool,
    pub price: I256,
    pub size: I256,
    pub underlying: String,
}

#[derive(Debug)]
pub struct PricesLasts {
    pub lbtc: I256,
    pub leth: I256,
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

#[derive(Debug, Serialize, Deserialize)]
struct TradingHistoryItemsGraphQueryResponseData {
    tradeHistoryItems: Vec<RawTradingHistoryItem>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PricesLastsGraphQueryResponseData {
    pricesLasts: Vec<RawPricesLasts>,
}

#[derive(Debug, Serialize, Deserialize)]
struct RawTradingHistoryItem {
    id: String,
    isLong: bool,
    price: String,
    size: String,
    underlying: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct RawPricesLasts {
    id: String,
    currentPrice: String,
}

const GRAPHQL_BATCH_SIZE: u32 = 500;
const GRAPHQL_TIMEOUT: Duration = Duration::from_secs(5);

impl GraphClient {
    pub fn new(perp_ui_query_endpoint: String, legacy_band_query_endpoint: String) -> Self {
        GraphClient {
            http_client: Client::new(),
            perp_ui_query_endpoint,
            legacy_band_query_endpoint,
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
                .post(&self.perp_ui_query_endpoint)
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
                    id: I256::from_dec_str(&raw_trading_history_item.id).map_err(|_| {
                        GraphqlError::InvalidId(raw_trading_history_item.id.to_owned())
                    })?,
                    isLong: raw_trading_history_item.isLong,
                    price: I256::from_dec_str(&raw_trading_history_item.price).map_err(|_| {
                        GraphqlError::InvalidId(raw_trading_history_item.price.to_owned())
                    })?,
                    size: I256::from_dec_str(&raw_trading_history_item.size).map_err(|_| {
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

    pub async fn get_lbtc_current_price(&self) -> Result<I256, GraphqlError> {
        let lbtc_query_str = include_str!("./queries/lbtc_prices_lasts_query.graphql");

        self.get_prices_lasts(String::from(lbtc_query_str)).await
    }

    pub async fn get_leth_current_price(&self) -> Result<I256, GraphqlError> {
        let leth_query_str = include_str!("./queries/leth_prices_lasts_query.graphql");

        self.get_prices_lasts(String::from(leth_query_str)).await
    }

    async fn get_prices_lasts(&self, query: String) -> Result<I256, GraphqlError> {
        let query = GraphQueryRequest {
            query,
            variables: GraphQueryVariables {
                skip: 0 as u32,
                first: 1 as u32,
            },
        };

        let res = self
            .http_client
            .post(&self.legacy_band_query_endpoint)
            .timeout(GRAPHQL_TIMEOUT)
            .header("Content-Type", "application/json")
            .body(serde_json::to_string(&query).map_err(GraphqlError::SerializationError)?)
            .send()
            .await
            .map_err(GraphqlError::NetworkError)?;

        let body: String = res.text().await.map_err(GraphqlError::NetworkError)?;
        let result: GraphQueryResponse<PricesLastsGraphQueryResponseData> =
            serde_json::from_str(&body).map_err(GraphqlError::SerializationError)?;

        let current_price =
            I256::from_dec_str(&result.data.pricesLasts[0].currentPrice).map_err(|_| {
                GraphqlError::InvalidId(result.data.pricesLasts[0].currentPrice.to_owned())
            })?;

        Ok(current_price)
    }
}
