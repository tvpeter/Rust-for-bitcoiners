use std::{env, time};

use bitcoincore_rpc::{json, jsonrpc::{self}, Auth, Client, RpcApi};
use chrono::Duration;
#[macro_use]
extern crate lazy_static;

lazy_static! {
    static ref RPC_CLIENT: Client = {
        dotenv::dotenv().ok();
        let rpc_url: String = env::var("BITCOIN_RPC_URL").expect("BITCOIN_RPC_URL must be set");
        let rpc_user: String = env::var("BITCOIN_RPC_USER").expect("BITCOIN_RPC_USER must be set");
        let rpc_password: String =
            env::var("BITCOIN_RPC_PASSWORD").expect("BITCOIN_RPC_PASSWORD must be set");
        Client::new(&rpc_url, Auth::UserPass(rpc_user, rpc_password)).unwrap()
    };
}

// static client: Client = Client::new("url", Auth::UserPass("user".to_owned(), "password".to_owned())).unwrap();

// TODO: Task 1
fn time_to_mine(block_height: u64) -> Duration {
    // * is a deref operator which invokes the Deref trait of the type RPC_CLIENT which was created
    // when the lazy macro is expanded
    // if a value has a static lifetime then it means that value lives as long as the program lives
    let rpc_client: &Client = &*RPC_CLIENT;

    let given_block_hash = rpc_client.get_block_hash(block_height).expect("Error obtaining blockhash for given block");
    let given_block_header = rpc_client.get_block_header(&given_block_hash).expect("error getting block header for given block");

    let prev_block_hash = given_block_header.prev_blockhash;
    let prev_block_header = rpc_client.get_block_header(&prev_block_hash).expect("error getting previous block header");

    let time_diff = given_block_header.time - prev_block_header.time;

    Duration::seconds(time_diff as i64)
}

// TODO: Task 2
fn number_of_transactions(block_height: u64) -> u16 {
    let rpc_client = &RPC_CLIENT;
    // let some_value = Box::new(4 as u32);
    let block_hash = rpc_client.get_block_hash(block_height).expect("error getting given block height");
    let block = rpc_client.get_block(&block_hash).expect("error getting block data");

    let transactions_num = block.txdata.len();

    transactions_num as u16
}

fn main() {
    // you can use rpc_client here as if it was a global variable
    // println!("{:?}", res);
    const TIMEOUT_UTXO_SET_SCANS: time::Duration = time::Duration::from_secs(60 * 8); // 8 minutes
    dotenv::dotenv().ok();
        let rpc_url: String = env::var("BITCOIN_RPC_URL").expect("BITCOIN_RPC_URL must be set");
        let rpc_user: String = env::var("BITCOIN_RPC_USER").expect("BITCOIN_RPC_USER must be set");
        let rpc_password: String =
            env::var("BITCOIN_RPC_PASSWORD").expect("BITCOIN_RPC_PASSWORD must be set");

    let custom_timeout_transport = jsonrpc::simple_http::Builder::new()
        .url(&rpc_url)
        .expect("invalid rpc url")
        .auth(rpc_user, Some(rpc_password))
        .timeout(TIMEOUT_UTXO_SET_SCANS)
        .build();
    let custom_timeout_rpc_client =
        jsonrpc::client::Client::with_transport(custom_timeout_transport);

    let rpc_client = Client::from_jsonrpc(custom_timeout_rpc_client);
    let res: json::GetTxOutSetInfoResult =
        rpc_client.get_tx_out_set_info(None, None, None).unwrap();
    println!("{:?}", res);
}
