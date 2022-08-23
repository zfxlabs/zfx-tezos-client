//use std::io::{self, BufRead, BufReader, Error, ErrorKind, Write};

use std::env;

//use actix::prelude::*;
use zfx_tezos_client::bridge::Bridge;
use zfx_tezos_client::Result;

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
    //let testnet_rpc_node = "https://jakartanet.ecadinfra.com".to_string();
    //let rpc_node = "https://mainnet.api.tez.ie".to_string();
    let local_node = rpc_node.to_string();
    let secret = "".to_string();
    let confirmations: isize = 1;
    let destination = "".to_string();
    let entrypoint = "".to_string();
    let big_map_keys = vec!["stuff".to_string()];

    let mut bridge = Bridge::new().await;
    //println!("bridge: {:?}", bridge);

    // this is an existing contract for kolibri on mainnet
    // let contract_address = "KT18muiNLcRnDqF1y7yowgea3iBU7QZXFzTD".to_string();
    // FXHASH contract - on mainnet
    //let contract_address = "KT1KEa8z6vWXDJrVqtMrAeDVzsvxat3kHaCE".to_string();
    let contract_address = contract_address.to_string();
    // My magic testnet contract
    //let testnet_contract_address = "KT1E9huZSqhk2FexWUQ1ckUmQZoiXeG5vFyk".to_string();

    // println!("before storage1");
    // let storage1 = bridge
    //     .storage(rpc_node.clone(), confirmations, contract_address.clone())
    //     .await;
    // println!("storage1: {:?}", storage1);
    // let storage2 = bridge
    //     .storage(rpc_node.clone(), confirmations, contract_address.clone())
    //     .await;
    // println!("storage2: {:?}", storage2);
    let storage3 = bridge
        .storage(local_node.clone(), confirmations, contract_address.clone())
        .await;
    println!(">>> local storage3: {:?}", storage3);

    let mut listen = bridge
        .listen(
            local_node.clone(),
            //testnet_rpc_node.clone(),
            //rpc_node,
            confirmations,
            //burn_address.clone(),
            //bob_account.clone(),
            contract_address,
        )
        .await
        .unwrap();

    println!("listening!");
    while let Ok(stuff) = listen.recv().await {
        println!("Listen: {:?}", stuff);
    }

    //bridge.drop();
    std::thread::sleep(std::time::Duration::from_secs(5));
    println!("ended");
}
