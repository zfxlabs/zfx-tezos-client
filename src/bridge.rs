use crate::prelude::*;

use serde::Deserialize;

use std::collections::HashMap;

use include_dir::{include_dir, Dir};

use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{ChildStdin, Command};
use tokio::sync::broadcast;
use tokio::sync::oneshot;

use std::process::Stdio;

static SCRIPTS_DIR: Dir<'_> = include_dir!("./scripts");
static BUNDLE_NAME: &str = "tezos_js_bridge.bundle.js";

pub async fn install() {
    let bridge_js = SCRIPTS_DIR.get_file(BUNDLE_NAME).unwrap();

    let mut file_to_deploy = File::create(BUNDLE_NAME).await.unwrap();
    file_to_deploy
        .write_all(bridge_js.contents())
        .await
        .unwrap();
}

// Internal messages
#[derive(Message)]
#[rtype(result = "isize")]
struct SubscribeToResponse {
    sender: oneshot::Sender<Value>,
}

#[derive(Message)]
#[rtype(result = "isize")]
struct SubscribeToListen {
    sender: broadcast::Sender<Value>,
}

#[derive(Message)]
#[rtype(result = "ChannelForId")]
struct GetChannelForId {
    id: isize,
}

#[derive(Debug, MessageResponse)]
enum ChannelForId {
    Oneshot { sender: oneshot::Sender<Value> },
    Broadcast { sender: broadcast::Sender<Value> },
    None,
}

#[derive(Clone, Debug, Deserialize)]
struct BridgeResponse {
    id: isize,
    content: Value,
}

#[derive(Debug)]
pub struct Bridge {
    addr: Addr<BridgeActor>,
    stdin: ChildStdin,
}

impl Bridge {
    pub fn new() -> Bridge {
        let mut child = Command::new("node")
            .current_dir("./src")
            .args(&["tezos_js_bridge.js"]) //FIXME: config
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("bridge::command failed");

        let actor: BridgeActor = BridgeActor::new();
        let addr = actor.start();
        let bridge_manager = addr.clone();

        let stdout = child
            .stdout
            .take()
            .expect("child did not have a handle to stdout");

        tokio::spawn(async move {
            let mut reader = BufReader::new(stdout).lines();

            while let Ok(r) = reader.next_line().await {
                match r {
                    None => (),
                    Some(line) => process_response(bridge_manager.clone(), line).await,
                }
            }

            println!("listener ending");
        });

        let stdin = child.stdin.take().expect("couldn't get stdin");
        Bridge { addr, stdin }
    }

    pub async fn inject_transaction(
        &mut self,
        rpc_node: String,
        secret: String,
        confirmation: isize,
        destination: String,
        entrypoint: String,
        payload: Vec<MichelsonV1Expression>,
    ) -> Result<TransactionResponse> {
        let content = RequestContent::transaction {
            rpc_node,
            secret,
            confirmation,
            destination,
            entrypoint,
            payload, //: vec![], // FIXME
        };
        let (sender, receiver) = oneshot::channel::<Value>();
        let id = self
            .addr
            .send(SubscribeToResponse { sender })
            .await
            .unwrap();

        submit_request(&mut self.stdin, id, content).await;

        let data = receiver.await;
        let response: TransactionResponse = serde_json::from_value(data.unwrap()).unwrap();
        Ok(response)
    }

    pub async fn storage(
        &mut self,
        rpc_node: String,
        confirmation: isize,
        destination: String,
    ) -> Result<StorageResponse> {
        let (sender, receiver) = oneshot::channel();
        let id = self.addr.send(SubscribeToResponse { sender }).await;

        let content = RequestContent::storage {
            rpc_node,
            confirmation,
            destination,
        };
        submit_request(&mut self.stdin, id.unwrap(), content).await;

        let data = receiver.await;
        let response: StorageResponse = serde_json::from_value(data.unwrap()).unwrap();
        Ok(response)
    }

    pub async fn big_map_keys(
        &mut self,
        rpc_node: String,
        confirmation: isize,
        destination: String,
        keys: Vec<BigMapKey>,
    ) -> Result<StorageResponse> {
        let (sender, receiver) = oneshot::channel();
        let id = self.addr.send(SubscribeToResponse { sender }).await;

        let content = RequestContent::big_map_keys {
            rpc_node,
            confirmation,
            destination,
            keys,
        };
        submit_request(&mut self.stdin, id.unwrap(), content).await;

        let data = receiver.await;
        let response: StorageResponse = serde_json::from_value(data.unwrap()).unwrap();
        Ok(response)
    }

    pub async fn listen(
        &mut self,
        rpc_node: String,
        confirmation: isize,
        destination: String,
    ) -> Result<broadcast::Receiver<Value>> {
        let listen_buffer_capacity = 128; // FIXME: constant or config
        let (sender, receiver) = broadcast::channel(listen_buffer_capacity);
        let id = self.addr.send(SubscribeToListen { sender }).await;

        let content = RequestContent::listen {
            rpc_node,
            confirmation,
            destination,
        };
        submit_request(&mut self.stdin, id.unwrap(), content).await;
        Ok(receiver)
    }
}

#[derive(Debug)]
struct BridgeActor {
    last_id: isize, // Perhaps this whole ID could be usize. (tezos bridge might not even work with negative).
    sync_requests: HashMap<isize, oneshot::Sender<Value>>, //id -> channels to respond to
    listen_requests: HashMap<isize, broadcast::Sender<Value>>,
}

impl BridgeActor {
    pub fn new() -> BridgeActor {
        let sync_requests = HashMap::new();
        let listen_requests = HashMap::new();
        BridgeActor {
            last_id: 0,
            sync_requests,
            listen_requests,
        }
    }
}

impl Actor for BridgeActor {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Context<Self>) {
        println!("started bridge"); // TODO: remove
        ()
    }

    fn stopped(&mut self, _ctx: &mut Context<Self>) {
        println!("stopped bridge"); // TODO: remove
        ()
    }
}

impl Handler<SubscribeToResponse> for BridgeActor {
    type Result = isize;

    fn handle(&mut self, msg: SubscribeToResponse, _ctx: &mut Context<Self>) -> Self::Result {
        let id = self.last_id + 1;
        self.last_id = id;
        self.sync_requests.insert(id, msg.sender);
        id
    }
}

impl Handler<SubscribeToListen> for BridgeActor {
    type Result = isize;

    fn handle(&mut self, msg: SubscribeToListen, _ctx: &mut Context<Self>) -> Self::Result {
        let id = self.last_id + 1;
        self.last_id = id;
        self.listen_requests.insert(id, msg.sender);
        id
    }
}

impl Handler<GetChannelForId> for BridgeActor {
    type Result = ChannelForId;

    fn handle(&mut self, msg: GetChannelForId, _ctx: &mut Context<Self>) -> Self::Result {
        match self.sync_requests.remove(&msg.id) {
            Some(sender) => ChannelForId::Oneshot { sender },
            None => match self.listen_requests.remove(&msg.id) {
                Some(sender) => {
                    let response = ChannelForId::Broadcast {
                        sender: sender.clone(),
                    };
                    self.listen_requests.insert(msg.id, sender);
                    response
                }
                _ => ChannelForId::None,
            },
        }
    }
}

async fn submit_request(stdin: &mut ChildStdin, id: isize, content: RequestContent) // -> ChildStdin
{
    let request = BridgeRequest { id, content };
    let json = serde_json::to_string(&request).expect("Failed to encode request to JSON");
    let payload = format!("{}\n", json);
    println!(">>> payload sent: {}", payload);
    stdin
        .write_all(&payload.as_bytes())
        .await
        .expect("stdin - Write failed");
    stdin.flush().await.expect("stdin - Read failed");
}

async fn process_response(bridge_manager: Addr<BridgeActor>, line: String) {
    println!("line: {:?}", line);
    let response: BridgeResponse = serde_json::from_str(&line).expect("unable to decode json");
    let res = bridge_manager
        .send(GetChannelForId { id: response.id })
        .await
        .expect("requesting id from bridge_manager failed");

    match res {
        ChannelForId::Oneshot { sender } => {
            sender.send(response.content).unwrap();
            ()
        }
        ChannelForId::Broadcast { sender } => {
            sender.send(response.content).unwrap();
            ()
        }
        ChannelForId::None => println!("oopsie"),
    }
}
