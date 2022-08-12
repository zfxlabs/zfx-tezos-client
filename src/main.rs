//use std::io::{self, BufRead, BufReader, Error, ErrorKind, Write};

//use actix::prelude::*;
use zfx_tezos_client::bridge::Bridge;
use zfx_tezos_client::Result;

fn main() -> Result<()> {
    let sys = actix::System::new();

    sys.block_on(async move { sanity().await });

    sys.run().unwrap();
    Ok(())
}

async fn sanity() {
    //let rpc_node = "https://jakartanet.ecadinfra.com".to_string();
    let rpc_node = "https://mainnet.api.tez.ie".to_string();
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
    let contract_address = "KT1KEa8z6vWXDJrVqtMrAeDVzsvxat3kHaCE".to_string();

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
        .storage(rpc_node.clone(), confirmations, contract_address.clone())
        .await;
    println!("storage3: {:?}", storage3);

    //bridge.drop();
    std::thread::sleep(std::time::Duration::from_secs(5));
    println!("ended");
}
