use std::env;

use serde_json::Value;

use zfx_tezos_client::bridge::*;
use zfx_tezos_client::Result;

use zfx_michelson::michelson::*;
use zfx_tezos_client::prelude::*;

use clap::{value_t, values_t, App, Arg};

fn main() -> Result<()> {
    let matches = App::new("zfx-tezos-client")
        .version("0.1")
        .author("zero.fx labs ltd.")
        .about("Runs a zfx-tezos bridge")
        .arg(
            Arg::with_name("rpc-address")
                .short("r")
                .long("rpc-address")
                .value_name("RPC_ADDRESS")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("contract-address")
                .short("c")
                .long("contract-address")
                .value_name("CONTRACT_ADDRESS")
                .takes_value(true),
        )
        .get_matches();

    let rpc_node = match matches.value_of("rpc-address") {
        Some(node_str) => node_str.to_string(),
        _ => "https://jakartanet.ecadinfra.com".to_string(),
    };
    println!("RPC url: {}", rpc_node);

    let contract_address = match matches.value_of("contract-address") {
        Some(addr) => addr.to_string(),
        _ => "KT1E9huZSqhk2FexWUQ1ckUmQZoiXeG5vFyk".to_string(),
    };
    println!("Contract address: {}", contract_address);

    let sys = actix::System::new();

    sys.block_on(async move { sanity(&rpc_node, &contract_address).await });

    sys.run().unwrap();
    Ok(())
}

async fn sanity(rpc_node: &String, contract_address: &String) {
    // Install
    install_parser().await;
    install_bridge().await;

    let local_node = rpc_node.to_string();
    let confirmations: isize = 1;

    let mut bridge = Bridge::new();

    let contract_address = contract_address.to_string();

    println!("before storage1");
    let storage1 = bridge
        .storage(rpc_node.clone(), confirmations, contract_address.clone())
        .await;
    println!("storage1: {:?}", storage1);
    let storage2 = bridge
        .storage(rpc_node.clone(), confirmations, contract_address.clone())
        .await;
    println!("storage2: {:?}", storage2);
    let storage3 = bridge
        .storage(local_node.clone(), confirmations, contract_address.clone())
        .await
        .unwrap();
    println!(">>> local storage3: {:?}", storage3);

    let schema_str = "{ \"prim\": \"int\" }".to_string();
    let schema: Value = serde_json::from_str(&schema_str).unwrap();
    println!("schema: {:?}", schema);

    let mut p = Parser::new();

    match storage3 {
        StorageResponse::success { storage } => {
            let decoded = p.decode(storage, schema.clone()).await;
            println!("decoded: {:?}", decoded);
        }
        _ => println!("not successful!"),
    }

    let param_schema_str = "{ \"prim\": \"or\",
          \"args\":
            [ { \"prim\": \"or\",
                \"args\":
                  [ { \"prim\": \"int\", \"annots\": [ \"%decrement\" ] },
                    { \"prim\": \"int\", \"annots\": [ \"%increment\" ] } ] },
              { \"prim\": \"unit\", \"annots\": [ \"%reset\" ] } ] }"
        .to_string();
    let param_schema: Value = serde_json::from_str(&param_schema_str).unwrap();

    //let alice = "tz1PiXkj6iCdqrVJtQAFkWptG3AeWEjqVPeZ".to_string();

    //let mut listen = bridge
    //    .listen(local_node.clone(), confirmations, contract_address)
    //    .await
    //    .unwrap();
    let mut listen = bridge
        .subscribe(local_node.clone(), confirmations)
        .await
        .unwrap();

    println!("listening!");
    while let Ok(stuff) = listen.recv().await {
        println!("Listen: {:?}", stuff);
        let v = stuff
            .get("transactions")
            .unwrap()
            .get(0)
            .unwrap()
            .get("value")
            .unwrap();
        println!("V {:?}", v.clone());
        let decoded_tx = p.decode(v.clone(), param_schema.clone()).await;
        println!("decoded tx: {:?}", decoded_tx);
    }

    std::thread::sleep(std::time::Duration::from_secs(5));
    println!("ended");
}
