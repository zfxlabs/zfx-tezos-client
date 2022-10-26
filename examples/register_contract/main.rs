#![recursion_limit = "256"]

/// Example retrieves contract storage through the bridge,
/// and decodes it's content with the help of the storage's schema

use std::collections::HashSet;

use serde_json::{json, Value};
use serde::{Deserialize, Serialize};

use zfx_tezos_client::bridge::*;
use zfx_tezos_client::Result;

use zfx_michelson::*;
use zfx_tezos_client::prelude::*;

use clap::{App, Arg};

type BigMapId = usize;
#[derive(PartialEq, Eq, Clone, Debug, Serialize, Deserialize)]
pub enum State {
    Genesis,
    Sealed,
    Open,
}
impl EncodeableEnum for State {}

wrapped_struct! { Storage {
  owner: String,
  state: State,
  validators: HashSet<String>,
  validator_map: BigMapId,
  old_validators: HashSet<String>,
  old_validator_map: BigMapId,
} as WrappedStorage }

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
         //_ => "https://rpc.ghostnet.teztnets.xyz".to_string(),
    };
    println!("RPC url: {}", rpc_node);

    let contract_address = match matches.value_of("contract-address") {
        Some(addr) => addr.to_string(),
        _ => "KT1WfuYfNmZVgvEgdeBM7KL73WPneiEbQT83".to_string(),
    };
    println!("Contract address: {}\n", contract_address);

    let sys = actix::System::new();

    sys.block_on(async move { get_storage(&rpc_node, &contract_address).await });

    Ok(())
}

async fn get_storage(rpc_node: &String, contract_address: &String) {
    // Install
    install_bridge().await;

    let confirmations: isize = 1;

    let mut bridge = Bridge::new();

    let contract_address = contract_address.to_string();

    let storage = bridge
        .storage(rpc_node.clone(), confirmations, contract_address.clone())
        .await;
    println!("Received storage: {:?}\n", storage);

    let mut p = Parser::new();
    let schema: Value = json! { {
        "prim": "pair",
        "args": [
          {
            "prim": "pair",
            "args": [
              {
                "prim": "pair",
                "args": [
                  {
                    "prim": "big_map",
                    "args": [
                      {
                        "prim": "key_hash"
                      },
                      {
                        "prim": "list",
                        "args": [
                          {
                            "prim": "pair",
                            "args": [
                              {
                                "prim": "pair",
                                "args": [
                                  {
                                    "prim": "pair",
                                    "args": [
                                      {
                                        "prim": "key_hash",
                                        "annots" : [
                                          "%baking_account"
                                        ]
                                      },
                                      {
                                        "prim": "key",
                                        "annots": [
                                          "%public_key"
                                        ]
                                      }
                                    ]
                                  },
                                  {
                                    "prim": "bytes",
                                    "annots": [
                                      "%tls_cert"
                                    ]
                                  }
                                ],
                                "annots": [
                                  "%register"
                                ]
                              },
                              {
                                "prim": "timestamp",
                                "annots": [
                                  "%timestamp"
                                ]
                              }
                            ]
                          }
                        ]
                      }
                    ],
                    "annots": [
                      "%old_validator_map"
                    ]
                  },
                  {
                    "prim": "set",
                    "args": [
                      {
                        "prim": "key_hash"
                      }
                    ],
                    "annots": [
                      "%old_validators"
                    ]
                  }
                ]
              },
              {
                "prim": "address",
                "annots": [
                  "%owner"
                ]
              },
              {
                "prim": "or",
                "args": [
                  {
                    "prim": "or",
                    "args": [
                      {
                        "prim": "unit",
                        "annots": [
                          "%genesis"
                        ]
                      },
                      {
                        "prim": "unit",
                        "annots": [
                          "%open"
                        ]
                      }
                    ]
                  },
                  {
                    "prim": "unit",
                    "annots": [
                      "%sealed"
                    ]
                  }
                ],
                "annots": [
                  "%state"
                ]
              }
            ]
          },
          {
            "prim": "big_map",
            "args": [
              {
                "prim": "key_hash"
              },
              {
                "prim": "pair",
                "args": [
                  {
                    "prim": "pair",
                    "args": [
                      {
                        "prim": "key_hash",
                        "annots": [
                          "%baking_account"
                        ]
                      },
                      {
                        "prim": "key",
                        "annots": [
                          "%public_key"
                        ]
                      }
                    ]
                  },
                  {
                    "prim": "bytes",
                    "annots": [
                      "%tls_cert"
                    ]
                  }
                ]
              }
            ],
            "annots": [
              "%validator_map"
            ]
          },
          {
            "prim": "set",
            "args": [
              {
                "prim" : "key_hash"
              }
            ],
            "annots": [
              "%validators"
            ]
          }
        ]
    } };

    match storage {
        Ok(StorageResponse::success { storage }) => {
            let decoded_json = p
                .decode(storage, schema.clone())
                .await
                .expect("decoding failed (wrong storage schema?)");
            let decoded_storage: Storage = from_wrapped_value(decoded_json).expect("Wrong storage structure");
            println!("decoded storage: {:?}", decoded_storage);
        }
        _ => println!("not successful!"),
    }
}
