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
    let testnet_rpc_node = "https://jakartanet.ecadinfra.com".to_string();
    let rpc_node = "https://mainnet.api.tez.ie".to_string();
    let local_node = "http://localhost:8732".to_string();
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
    // My magic testnet contract
    let testnet_contract_address = "KT1E9huZSqhk2FexWUQ1ckUmQZoiXeG5vFyk".to_string();

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
            testnet_contract_address,
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
