use ethers::prelude::*;

abigen!(
    LnConfig,
    "./src/abis/LnConfig.json",
    event_derives(serde::Deserialize, serde::Serialize)
);
abigen!(
    LnPerpExchange,
    "./src/abis/LnPerpExchange.json",
    event_derives(serde::Deserialize, serde::Serialize)
);
