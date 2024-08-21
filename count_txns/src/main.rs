use bitcoincore_rpc::{Auth, Client, RpcApi};
use dotenv::dotenv;
use std::env;
use std::{
    sync::{Arc, Mutex},
    thread,
    time::SystemTime,
};

fn main() {
    dotenv().ok();

    let transactions_count = Arc::new(Mutex::new(0u64));
    let mut handles = vec![];

    // number of threads
    let num_threads = 10;
    let node_url = env::var("NODE_URL").unwrap();
    let node_password = env::var("RPC_PASSWORD").unwrap();
    let node_username = env::var("RPC_USER").unwrap();

    let start_time = SystemTime::now();
    let rpc_client = Client::new(
        &node_url,
        Auth::UserPass(node_password.clone(), node_username.clone()),
    )
    .unwrap();
    let block_count = rpc_client.get_block_count().unwrap();
    println!("Block count: {}", block_count);
    let block_range = block_count / num_threads;
    let remainder = block_count % num_threads;

    for i in 0..num_threads {
        let start = i * block_range;
        let end = if i == num_threads - 1 {
            // last thread gets the remainder
            start + block_range + remainder
        } else {
            start + block_range
        };

        // Clone the Arc to share ownership across threads
        let transactions_count = Arc::clone(&transactions_count);

        // Create a new thread
        let handle = thread::spawn(move || {
            let rpc_client = Client::new(
                &env::var("NODE_URL").unwrap(),
                Auth::UserPass(
                    env::var("RPC_PASSWORD").unwrap(),
                    env::var("RPC_USER").unwrap(),
                ),
            )
            .unwrap();

            for height in start..end {
                let transactions = get_transactions_count(&rpc_client, height);

                // Lock the mutex to get access to the u64 value
                let mut num = transactions_count.lock().unwrap();

                // Increment the u64 value
                *num += transactions;
            }
        });

        // Collect thread handles
        handles.push(handle);
    }

    // Wait for all threads to finish
    for handle in handles {
        handle.join().unwrap();
    }

    let completing_time = start_time.elapsed().unwrap().as_secs();

    println!("transactions count: {}", transactions_count.lock().unwrap());
    println!("time elapsed: {:?}", completing_time);
}

fn get_transactions_count(client: &Client, block_height: u64) -> u64 {
    let block_hash = client.get_block_hash(block_height).unwrap();

    let block = client.get_block(&block_hash).unwrap();

    block.txdata.len() as u64
}
