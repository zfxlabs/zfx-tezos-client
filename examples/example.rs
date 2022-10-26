/// Example listenes to a contract on mainnet and prints transaction details

use zfx_tezos_client::bridge::*;
use zfx_tezos_client::Result;
use zfx_michelson::michelson::*;
use clap::{App, Arg};

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
        _ => "https://rpc.tzstats.com".to_string(),
        // _ => "https://rpc.ghostnet.teztnets.xyz".to_string(),
    };
    println!("RPC url: {}", rpc_node);

    // objkt.com Marketplace 2.0 contract
    // https://tzstats.com/KT1WvzYHCNBvDSdwafTHv7nJ1dWmZ8GCYuuC
    let contract_address = match matches.value_of("contract-address") {
        Some(addr) => addr.to_string(),
        _ => "KT1WvzYHCNBvDSdwafTHv7nJ1dWmZ8GCYuuC".to_string(),
    };
    println!("Contract address: {}", contract_address);

    let sys = actix::System::new();

    sys.block_on(async move { sanity(&rpc_node, &contract_address).await });

    sys.run().unwrap();
    Ok(())
}

async fn sanity(rpc_node: &String, contract_address: &String) {
    //Install
    install_parser().await;
    install_bridge().await;

    let confirmations: isize = 1;

    let mut bridge = Bridge::new();

    let contract_address = contract_address.to_string();

    let storage1 = bridge
        .storage(rpc_node.clone(), confirmations, contract_address.clone())
        .await;
    println!("initial storage: {:?}", storage1);

    let mut listen = bridge
        .subscribe(rpc_node.clone(), confirmations)
        .await
        .unwrap();

    println!("listening!");

    while let Ok(stuff) = listen.recv().await {
        let dest = stuff.get("destination").unwrap().to_string().replace("\"", "");
        if dest == contract_address {
            println!("{}\n", serde_json::to_string_pretty(&stuff).unwrap());
            println!("-------------------------------------------------\n");
        }     
    };

    println!("ended");
}