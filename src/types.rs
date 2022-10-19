use crate::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize)]
pub struct BridgeRequest {
    pub id: isize,
    pub content: RequestContent,
}

/// Container type for the response from the bridge
#[derive(Clone, Debug, Deserialize)]
pub struct BridgeResponse<T> {
    pub id: isize,
    pub content: T,
}

// This is a correct, but simplistic definition,
// which works well for this library as the code is very generic,
// users should convert and validate data when interacting with this library
pub type MichelsonV1Expression = Value;

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
    subscribe {
        rpc_node: String,
        confirmation: isize,
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
#[serde(tag = "status")]
#[allow(non_camel_case_types)]
pub enum StorageResponse {
    success { storage: MichelsonV1Expression },
    // The bridge docs say errors are strings, but sometimes they
    // send JSON structures as sub-maps in the error field
    error { error: Value },
}

/// Unused for the time being
#[derive(Clone, Debug, Deserialize)]
pub struct TransactionMessage {
    pub hash: String,
    pub transactions: Vec<Transaction>,
}

/// Unused for the time being
#[derive(Clone, Debug, Deserialize)]
pub struct Transaction {
    pub entrypoint: String,
    pub value: MichelsonV1Expression,
}
