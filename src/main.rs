use std::{env, str::FromStr, time};

use bitcoincore_rpc::{bitcoin::{Address, Amount, SignedAmount, Txid}, json::{self, GetTransactionResult, WalletTxInfo}, jsonrpc::{self}, Auth, Client, Error, RpcApi};
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
fn time_to_mine(block_height: u64) -> Result<Duration, Error> {
    // * is a deref operator which invokes the Deref trait of the type RPC_CLIENT which was created
    // when the lazy macro is expanded
    // if a value has a static lifetime then it means that value lives as long as the program lives
    let rpc_client: &Client = &*RPC_CLIENT;

    let given_block_hash = rpc_client.get_block_hash(block_height)?;
    let given_block_header = rpc_client.get_block_header(&given_block_hash)?;

    let prev_block_hash = given_block_header.prev_blockhash;
    let prev_block_header = rpc_client.get_block_header(&prev_block_hash)?;

    let time_diff = given_block_header.time - prev_block_header.time;

    Ok(Duration::seconds(time_diff as i64))
}

// TODO: Task 2
fn number_of_transactions(block_height: u64) -> Result<u16, Error> {
    let rpc_client = &*RPC_CLIENT;

    // let some_value = Box::new(4 as u32);
    let block_hash = rpc_client.get_block_hash(block_height)?;
    let block = rpc_client.get_block(&block_hash)?;

    let transactions_num = block.txdata.len();

    Ok(transactions_num as u16)
}


fn is_segwit_address(address: &str) -> Result<bool, Error> {
    let rpc_client = &*RPC_CLIENT;

    let address = Address::from_str(address);

    let derived_address =  match address {
        Ok(address) => address,
        Err(error) => return Err(Error::ReturnedError(error.to_string())),
    };

    // validate given address
    let address_info = rpc_client.get_address_info(&derived_address.assume_checked())?;
    
   let status = address_info.is_witness.unwrap_or(false);

   Ok(status)
}

fn get_wallet_balance() -> Result<Amount, Error>{
    let rpc_client = &*RPC_CLIENT;

    //loaded wallet
    let wallet_info = rpc_client.get_wallet_info()?;

    Ok(wallet_info.balance)
}

fn get_transaction_info(txid: &str) -> Result<(i32, Option<u32>, SignedAmount), Error>{
    let rpc_client = &*RPC_CLIENT;

    let txid = Txid::from_str(txid);

    let txid = match txid {
        Ok(txnid) => txnid,
        Err(error) => return Err(Error::ReturnedError(error.to_string())),
    };

    let trxn_info = rpc_client.get_transaction(&txid, Some(false))?;

    let GetTransactionResult { info: WalletTxInfo { confirmations, blockheight, .. }, amount, .. } = trxn_info;

    let result = (confirmations, blockheight, amount);

    Ok(result)
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


//these tests will fail without connecting to a node
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_connection(){
        let client = &*RPC_CLIENT;

        let best_block_hash = client.get_best_block_hash().unwrap();

        assert!(!best_block_hash.to_string().is_empty());
        // blockhash is 32-bytes .i.e 64 characters (sha256)
        assert_eq!(best_block_hash.to_string().len(), 64); 
    }

    #[test]
    fn test_time_to_mine(){
        let time_to_mine = time_to_mine(900).unwrap();
        assert!(time_to_mine.num_seconds()>= 0);
    }

    #[test]
    #[should_panic]
    fn test_time_to_mine_failure(){
        //set this to be above the block count
        let time_to_mine = time_to_mine(1000).unwrap();
        time_to_mine.num_seconds();
    }

    #[test]
    fn test_number_of_transactions(){

        let num_of_txns = number_of_transactions(800).unwrap();

        assert!(num_of_txns >= 1);
    }

    #[test]
    fn test_address_balance(){
        let address = "bcrt1qdpdk6yxavaeerwuumxyy8vc9k9zr2ysvghsxug";

        let is_segwit = is_segwit_address(address).unwrap();

        assert!(is_segwit);
    }

    #[test]
    fn test_get_wallet_balance(){
        let balance = get_wallet_balance().unwrap();
        assert!(balance.to_sat() > 0);
    }

    #[test]
    fn test_get_transaction_info(){
        let txid = "f965f67e86b658aae279ac01714a0aa8a78501d8d2b0463b8f298addd47ff0ba";

        let txn_info = get_transaction_info(txid).unwrap();

        assert!(txn_info.0 > 1);
        assert!(txn_info.1 > Some(1));
        assert!(txn_info.2.to_sat() > 100);
    }
}
