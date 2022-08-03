use crate::{
    error::*,
    graph_client::GraphClient,
    settings::GRAPHQL_QUERY_URL,
};
use log::{debug, info};

pub async fn run() -> Result<(), SettlerError>
{
    debug!("Fetching positions...");
    let graphql_query: String = String::from(GRAPHQL_QUERY_URL);
    let graphql_client = GraphClient::new(graphql_query);

    let positions = &graphql_client
        .get_positions()
        .await
        .map_err(SettlerError::GraphqlError)?;

    println!("{:#?}", positions);
    info!("{} positions found", positions.len());

    Ok(())
}
