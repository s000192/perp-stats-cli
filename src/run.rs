use crate::{
    aggregate::aggregate,
    error::*,
    graph_client::GraphClient,
    settings::{GRAPHQL_QUERY_LEGACY_BAND_URL, GRAPHQL_QUERY_PERP_UI_URL},
};
use log::{debug, info};

pub async fn run(user: &String) -> Result<(), SettlerError> {
    debug!("Fetching trading history items...");
    let perp_ui_graphql_query: String = String::from(GRAPHQL_QUERY_PERP_UI_URL);
    let legacy_band_graphql_query: String = String::from(GRAPHQL_QUERY_LEGACY_BAND_URL);
    let graphql_client = GraphClient::new(perp_ui_graphql_query, legacy_band_graphql_query);

    let mut trading_history_items = graphql_client
        .get_trading_history_items(&user)
        .await
        .map_err(SettlerError::GraphqlError)?
        .to_vec();

    let lbtc_current_price = graphql_client
        .get_prices_lasts("lBTC")
        .await
        .map_err(SettlerError::GraphqlError)?;

    let leth_current_price = graphql_client
        .get_prices_lasts("lETH")
        .await
        .map_err(SettlerError::GraphqlError)?;

    aggregate(
        &mut trading_history_items,
        &lbtc_current_price,
        &leth_current_price,
    );
    // println!("{:#?}", trading_history_items);
    info!(
        "{} trading history items found",
        trading_history_items.len()
    );

    Ok(())
}
