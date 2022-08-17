use crate::{
    error::*,
    graph_client::GraphClient,
    settings::GRAPHQL_QUERY_URL,
};
use log::{debug, info};

pub async fn run() -> Result<(), SettlerError>
{
    debug!("Fetching trading history items...");
    let graphql_query: String = String::from(GRAPHQL_QUERY_URL);
    let graphql_client = GraphClient::new(graphql_query);

    let trading_history_items = &graphql_client
        .get_trading_history_items()
        .await
        .map_err(SettlerError::GraphqlError)?;

    println!("{:#?}", trading_history_items);
    info!("{} trading history items found", trading_history_items.len());

    Ok(())
}
