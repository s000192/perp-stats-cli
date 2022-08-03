use ethers::prelude::*;

#[derive(Debug)]
pub enum SettlerError {
    GraphqlError(GraphqlError),
}

#[derive(Debug)]
pub enum GraphqlError {
    SerializationError(serde_json::Error),
    NetworkError(reqwest::Error),
    InvalidId(String),
    InvalidTimestamp(String),
}

#[derive(Debug)]
pub enum ParseSettingsError {
    InvalidHexFormat,
    InvalidNumberFormat,
    InvalidAddress,
    InvalidPrivateKey,
}

#[derive(Debug)]
pub struct PrivateKeyAddressMismatchError {
    pub expected: Address,
    pub actual: Address,
}
