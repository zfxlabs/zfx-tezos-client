use serde_json::{Value, json};

use zfx_tezos_client::bridge::*;
use zfx_tezos_client::Result;

use zfx_michelson::*;
use zfx_tezos_client::prelude::*;

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
        _ => "https://ghostnet.tezos.marigold.dev/".to_string(),
        // _ => "https://rpc.ghostnet.teztnets.xyz".to_string(),
    };
    println!("RPC url: {}", rpc_node);

    let contract_address = match matches.value_of("contract-address") {
        Some(addr) => addr.to_string(),
        _ => "KT1BK1AK8Xx1x7uwrFeKbpcaFKuYAZhruUzT".to_string(),
    };
    println!("Contract address: {}", contract_address);

    let sys = actix::System::new();

    sys.block_on(async move { get_storage(&rpc_node, &contract_address).await });

    Ok(())
}

async fn get_storage(rpc_node: &String, contract_address: &String) {
    // Install
    install_parser().await;
    install_bridge().await;

    let confirmations: isize = 1;

    let mut bridge = Bridge::new();

    let contract_address = contract_address.to_string();

    let storage = bridge
        .storage(rpc_node.clone(), confirmations, contract_address.clone())
        .await;
    println!("Received storage: {:?}", storage);


    let mut p = Parser::new();
    let schema: Value = json!{ {"prim": "int" } };

    match storage {
        Ok(StorageResponse::success { storage }) => {
            let decoded_json = p.decode(storage, schema.clone()).await.expect("decoding failed (wrong storage schema?)");
            let decoded_storage: i64 = from_wrapped_value(decoded_json).expect("int should be i64");
            println!("decoded storage: {:?}", decoded_storage);
        }
        _ => println!("not successful!"),
    }
}
