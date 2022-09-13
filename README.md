# zfx-tezos-client

Tezos light client library for Rust. Allows interactions with Tezos chains via RPC nodes.

### Requirements

This library needs Node.js 16.

### Summary

The library builds an Rust API layer around the `taquito` library. (See: https://tezostaquito.io/)

Taquito interfaces Tezos RPC nodes, a JS module implements a node.js process to handle communication via the standard input and standard output. The node.js process is started with `tokio::process::Command` and the Rust library encodes/decodes requests into JSON maps for each request-response pair.

The JS code is included in the Rust build, so before starting the node.js process it has to be installed. Installation practically means writing the JS module into a file.

## Functionality

### Storage

Fetching smart-contract storage, returns variables stored in the smart contract.

#### Example

```
use zfx_tezos_client::bridge::*;

install_bridge().await;

let mut bridge = Bridge::new();

let rpc_node = "https://jakartanet.ecadinfra.com".to_string();
let contract_address = "KT1E9huZSqhk2FexWUQ1ckUmQZoiXeG5vFyk".to_string();
let confirmations: isize = 1;

let storage = bridge
        .storage(rpc_node.clone(), confirmations, contract_address.clone())
        .await;

```

### Listen

Listen to transactions to a specfic address.

`listen` function won't block and return a response, it returns a `tokio::broadcast` channel's receiver where the transactions for that request will be sent.

#### Example

```
use zfx_tezos_client::bridge::*;

install_bridge().await;

let mut bridge = Bridge::new();

let rpc_node = "https://jakartanet.ecadinfra.com".to_string();
let contract_address = "KT1E9huZSqhk2FexWUQ1ckUmQZoiXeG5vFyk".to_string();
let confirmations: isize = 1;

let mut listen = bridge
    .listen(rpc_node.clone(), confirmations, contract_address)
    .await
    .unwrap();

while let Ok(t) = listen.recv().await {
    println!("Listen: {:?}", t);
}
```

### Build

`cargo build`

### Test

`cargo test`
