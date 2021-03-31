//! The docstrings on types and their fields with `derive(Clap)` are displayed
//! in the CLI `--help`.
use anoma::cli::{anoma_client_cli, CliBuilder};
use anoma::protobuf::services::rpc_service_client::RpcServiceClient;
use anoma::protobuf::types;
use anoma::rpc_types::{self, Message};
use color_eyre::eyre::Result;
use tendermint_rpc::{Client, HttpClient};

pub async fn main() -> Result<()> {
    let matches = CliBuilder::new().anoma_client_cli();

    match matches.subcommand() {
        Some(("tx", args)) => {
            // here unwrap is safe as the arguments are required
            let path = args.value_of("path").unwrap().to_string();
            let data = args.value_of("data").unwrap().to_string();
            Ok(exec_tx(path, data).await)
        }
        Some(("intent", args)) => {
            // here unwrap is safe as the arguments are required
            let orderbook = args.value_of("orderbook").unwrap().to_string();
            let data = args.value_of("data").unwrap().to_string();
            Ok(gossip(orderbook, data).await.unwrap())
        }
        _ => Ok(()),
    }
}

async fn exec_tx(code_path: String, data_hex: String) {
    // TODO tendermint cache blocks the same transaction sent more than once,
    // add a counter or timestamp?

    let code = std::fs::read(code_path).unwrap();
    let data = Some(hex::decode(data_hex).unwrap());
    let tx = rpc_types::Tx { code, data };
    let mut tx_bytes = vec![];
    tx.encode(&mut tx_bytes).unwrap();

    // NOTE: use this to print the request JSON body:
    // let request =
    // tendermint_rpc::endpoint::broadcast::tx_commit::Request::new(
    //     tx_bytes.clone().into(),
    // );
    // use tendermint_rpc::Request;
    // let request_body = request.into_json();
    // println!("HTTP request body: {}", request_body);

    let client =
        HttpClient::new("tcp://127.0.0.1:26657".parse().unwrap()).unwrap();
    // TODO broadcast_tx_commit shouldn't be used live
    let response = client.broadcast_tx_commit(tx_bytes.into()).await.unwrap();
    println!("{:#?}", response);
}

async fn gossip(
    _orderbook_addr: String,
    data: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut client = RpcServiceClient::connect("http://[::1]:39111").await?;
    let intent = Some(types::Intent { data });
    let intent_message = types::IntentMessage { intent };
    let message = types::Message {
        message: Some(types::message::Message::IntentMessage(intent_message)),
    };
    let _response = client.send_message(message).await?;
    Ok(())
}
