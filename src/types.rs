use crate::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize)]
pub struct BridgeRequest {
    pub id: isize,
    pub content: RequestContent,
}

// It seems more reasonable than working with nested enums, that have weird
// tagging rules for deserialization.
#[derive(Clone, Debug, Deserialize)]
pub struct BridgeResponse<T> {
    pub id: isize,
    pub content: T, // String, // FIXME: this is a new type
}

//#[derive(Clone, Debug, Serialize)]
pub type BigMapKey = String;

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "kind")]
#[allow(non_camel_case_types)]
pub enum RequestContent {
    // NOTE: using lower case types since the value it gets converted to in a JSON,
    //       needs to be lower case.
    listen {
        rpc_node: String,
        confirmation: isize,
        destination: String,
    },
    transaction {
        rpc_node: String, // URI
        secret: String,
        confirmation: isize,
        destination: String,
        entrypoint: String,
        payload: Vec<MichelsonV1Expression>,
    },
    storage {
        rpc_node: String,
        confirmation: isize,
        destination: String,
    },
    big_map_keys {
        rpc_node: String,
        confirmation: isize,
        destination: String,
        keys: Vec<BigMapKey>,
    },
}

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "status")]
#[allow(non_camel_case_types)]
pub enum TransactionResponse {
    applied { hash: String },
    failed { hash: String },
    skipped { hash: String },
    backtracked { hash: String },
    unknown { hash: String },
    // The bridge docs say errors are strings, but sometimes they
    // send JSON structures as sub-maps in the error field
    error { error: Value },
}

#[derive(Clone, Debug, Deserialize)]
pub struct TransactionMessage {
    hash: String,
    transactions: Vec<Transaction>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Transaction {
    entrypoint: String,
    value: MichelsonV1Expression,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "status")]
#[allow(non_camel_case_types)]
pub enum StorageResponse {
    success { storage: Value }, //MichelsonV1Expression },
    // The bridge docs say errors are strings, but sometimes they
    // send JSON structures as sub-maps in the error field
    error { error: Value },
}
