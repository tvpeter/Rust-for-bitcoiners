use std::{error::Error, fmt, fs::{self, File}, io::Write, str::FromStr, time::{ SystemTime, UNIX_EPOCH}};
use bitcoin::{ absolute::{Height, LockTime}, address::ParseError, block::Header, consensus::encode, error::UnprefixedHexError, hashes::Hash, hex::DisplayHex, witness, Address, Amount, Block, BlockHash, OutPoint, ScriptBuf, Sequence, Target, Transaction, TxIn, TxMerkleNode, TxOut, Txid, Witness};
use serde::{Deserialize, Serialize};
use std::path::Path;
use walkdir::WalkDir;


const MAX_BLOCK_WEIGHT: u64 = 4_000_000; // 4 MB in weight units

#[derive(Debug)]
enum BlockMiningError {
    InvalidFileExtension,
    DirectoryNotFound,
    ReadFileError(std::io::Error),
    ParseJsonError(serde_json::Error),
    SystemTimeError(std::time::SystemTimeError),
    UnprefixedHexError,
    ParseError(ParseError),
    InvalidVersion,
    InvalidLockTime,
    NoInputs,
    NoOutputs,
    InvalidInput,
    InvalidMerkleRoot,
}

impl fmt::Display for BlockMiningError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BlockMiningError::InvalidFileExtension => write!(f, "Invalid file extension"),
            BlockMiningError::DirectoryNotFound => write!(f, "Directory not found"),
            BlockMiningError::ReadFileError(err) => write!(f, "Error reading file: {}", err),
            BlockMiningError::ParseJsonError(err) => write!(f, "Error parsing JSON: {}", err),
            BlockMiningError::SystemTimeError(e) => write!(f, "SystemTimeError: {}", e),
            BlockMiningError::UnprefixedHexError => write!(f, "UnprefixedHexError"),
            BlockMiningError::ParseError(e) => write!(f, "ParseError: {}", e),
            BlockMiningError::InvalidVersion => write!(f, "InvalidVersion"),
            BlockMiningError::InvalidLockTime => write!(f, "InvalidLockTime"),
            BlockMiningError::NoInputs => write!(f, "NoInputs"),
            BlockMiningError::NoOutputs => write!(f, "NoOutputs"),
            BlockMiningError::InvalidInput => write!(f, "InvalidInput"),
            BlockMiningError::InvalidMerkleRoot => write!(f, "InvalidMerkleRoot"),
        }
    }
}

impl Error for BlockMiningError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            BlockMiningError::ReadFileError(err) => Some(err),
            BlockMiningError::ParseJsonError(err) => Some(err),
            _ => None,
        }
    }
}

impl From<std::time::SystemTimeError> for BlockMiningError {
    fn from(error: std::time::SystemTimeError) -> Self {
        BlockMiningError::SystemTimeError(error)
    }
}

impl From<UnprefixedHexError> for BlockMiningError {
    fn from(_error: UnprefixedHexError) -> Self {
        BlockMiningError::UnprefixedHexError
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Prevout {
    scriptpubkey: String,
    scriptpubkey_asm: String,
    scriptpubkey_type: String,
    scriptpubkey_address: Option<String>,
    value: u64,
}
#[derive(Debug, Deserialize, Serialize, Clone)]
struct Vin {
    txid: String,
    vout: u32,
    prevout: Prevout,
    scriptsig: Option<String>,
    scriptsig_asm: Option<String>,
    witness: Option<Vec<String>>,
    is_coinbase: bool,
    sequence: u32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Vout {
    scriptpubkey: String,
    scriptpubkey_asm: String,
    scriptpubkey_type: String,
    scriptpubkey_address: Option<String>,
    value: u64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Status {
    confirmed: bool,
    block_height: u32,
    block_hash: String,
    block_time: u32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct BTransaction {
    txid: String,
    version: u32,
    locktime: u32,
    vin: Vec<Vin>,
    vout: Vec<Vout>,
    size: u32,
    weight: u32,
    fee: u64,
    status: Status,
    hex: String,
}

fn get_transactions(path: &str) -> Result<Vec<BTransaction>, BlockMiningError> {
    let mut transactions: Vec<BTransaction> = Vec::new();

    // Check if the path is a directory
    if !Path::new(path).is_dir() {
        return Err(BlockMiningError::DirectoryNotFound);
    }

    let directory = WalkDir::new(path);

    // Iterate over the files in the directory
    for file in directory.into_iter().filter_map(|e| e.ok()) {
        if file.file_type().is_file() {
            let file_path = file.path();

            // Check if the file is a json file
            if file_path.extension().and_then(|s| s.to_str()) != Some("json") {
                return Err(BlockMiningError::InvalidFileExtension);
            }
            if file.file_name() == "mempool.json" {
                continue;
            }

            // Read the file contents
            let content = fs::read_to_string(file_path).map_err(BlockMiningError::ReadFileError)?;

            // Parse the JSON content into a Transaction struct
            let transaction: BTransaction = serde_json::from_str(&content).map_err(BlockMiningError::ParseJsonError)?;
            transactions.push(transaction);
        }
    }
    Ok(transactions)
}

// Get the target from the target string
fn get_target(target: &str) -> Result<Target, BlockMiningError> {
    let target = Target::from_unprefixed_hex(target)?;
    Ok(target)
}

fn get_transaction_fee(tx: &BTransaction) -> Option<u64> {
    let mut total_input = 0;
    let mut total_output = 0;   
    for input in tx.vin.iter() {
        total_input += input.prevout.value;
    }
    for output in tx.vout.iter() {
        total_output += output.value;
    }
    Some(total_input - total_output)
}


fn time_stamp() -> Result<u32, BlockMiningError> {
    let time = SystemTime::now().duration_since(UNIX_EPOCH)?;
    Ok(time.as_secs() as u32)
}

fn convert_txin(vin: &[Vin]) -> Vec<TxIn> {
    vin.iter().map(|vin| {
        let witness = if vin.witness.is_some() {
            Some(witness::Witness::from_slice(vin.witness.clone().unwrap().as_slice()))
        } else {
            None
        };

        TxIn {
            previous_output: OutPoint {
                txid: Txid::from_str(&vin.txid).unwrap(),
                vout: vin.vout,
            },
            script_sig: ScriptBuf::from_hex(&vin.scriptsig.clone().unwrap()).unwrap(),
            sequence: Sequence(vin.sequence),
            witness: witness.unwrap_or_default(),
        }
    }).collect()
}

fn convert_vout(vout: &[Vout]) -> Vec<TxOut> {
    vout.iter().map(|vout| {
        TxOut {
            script_pubkey: ScriptBuf::from_hex(vout.scriptpubkey.as_str()).unwrap(),
            value: Amount::from_sat(vout.value),
        }
    }).collect()
}

fn output_scriptpk(address: &str) -> ScriptBuf {
    let address = Address::from_str(address).map_err(BlockMiningError::ParseError).unwrap().assume_checked();
    address.script_pubkey()
}

fn coinbase_transaction(script_pubkey: ScriptBuf, total_fee: u64) -> Result<Transaction, BlockMiningError> {

    let block_height = "994120".as_bytes(); //block height
    let miner = "tvpeter".as_bytes();

    let mut script_sig_bytes = Vec::new();
    script_sig_bytes.push(block_height.len() as u8);
    script_sig_bytes.extend(block_height);
    script_sig_bytes.push(miner.len() as u8);
    script_sig_bytes.extend(miner);

    let script_sig = ScriptBuf::from_bytes(script_sig_bytes);

    let mut witness_reserved = Witness::new();
    let reserved = &[0; 32];
    witness_reserved.push(reserved);

    let coinbase_input = TxIn {
        previous_output: OutPoint::null(), 
        script_sig,
        sequence: bitcoin::Sequence(0xFFFFFFFF),
        witness: witness_reserved,
    };

      // add block rewards and fees to the coinbase transaction
    let output = vec![TxOut {script_pubkey: script_pubkey.clone(),value:Amount::from_sat(3_125_000) }, 
        TxOut { script_pubkey, value: Amount::from_sat(total_fee) }];
    
    Ok(Transaction {
        version: bitcoin::transaction::Version::TWO,
        lock_time: LockTime::ZERO, 
        input: vec![coinbase_input],
        output,
    })
}

fn mine_block(mut block_header: Header, target:Target) -> Header {

    loop {
        let hash = block_header.block_hash();

        if hash.to_byte_array() <= target.to_be_bytes() {
            return block_header;
        }
    
        block_header.nonce += 1;
    }
}

fn construct_candidate_block(txdata: Vec<Transaction>, target: Target) -> Result<Block, BlockMiningError> {

    let prev_blockhash = BlockHash::all_zeros();


    let time = time_stamp()?;

    let merkle_root = TxMerkleNode::all_zeros();

    let block_header = Header {
        version: bitcoin::block::Version::from_consensus(4),
        prev_blockhash,
        merkle_root,
        time,
        bits: target.to_compact_lossy(),
        nonce: 0,
    };

   let mut candidate_block = Block {
        header: block_header,
        txdata,
    };

    let merkle_root = if candidate_block.compute_merkle_root().is_none() {
        return Err(BlockMiningError::InvalidMerkleRoot);
    } else {
        candidate_block.compute_merkle_root().unwrap()
    };
    candidate_block.header.merkle_root = merkle_root;

    Ok(candidate_block)

}

fn validate_transaction(tx: &Transaction) -> Result<Transaction, BlockMiningError> {
    let txin = tx.input.clone();
    let txout = tx.output.clone();

    // Check if the transaction is valid
    if tx.version < bitcoin::transaction::Version::ONE {
        return Err(BlockMiningError::InvalidVersion);
    }

    // only transactions with a lock time of 0 are valid are valid
    if tx.lock_time != LockTime::ZERO {
        return Err(BlockMiningError::InvalidLockTime);
    }

    // Check if the transaction has at least one input
    if txin.is_empty() {
        return Err(BlockMiningError::NoInputs);
    }

    // Check if the transaction has at least one output
    if txout.is_empty() {
        return Err(BlockMiningError::NoOutputs);
    }

    // Check if the transaction has a valid input
    for input in txin.iter() {
        if input.previous_output.txid == Txid::all_zeros() {
            return Err(BlockMiningError::InvalidInput);
        }
    }

    Ok(tx.clone())
}


fn select_transactions( transactions: Vec<BTransaction>) -> Vec<(Transaction, u64, f64)> {
    let mut txdata: Vec<(Transaction, u64, f64)> = Vec::new();

    for tx in transactions.iter() {
    let height = Height::from_consensus(tx.locktime);

    if height.is_err() {
        continue;
    }

    let fee = get_transaction_fee(tx);

    if fee.is_none() {
        continue;
    }

        let txn = Transaction {
            version: bitcoin::transaction::Version(tx.version as i32),
            lock_time: LockTime::from_height(tx.locktime).unwrap(),
            input: convert_txin(&tx.vin),
            output: convert_vout(&tx.vout),
        };

        let tx_weight = txn.weight().to_vbytes_ceil();
        let fee = fee.unwrap();

        if tx_weight > fee {
            continue;
        }
        let fee_rate = fee as f64 / tx_weight as f64;
        //validate the transaction, if it is valid, add it to the block
       if validate_transaction(&txn).is_ok() {
            // add the transaction to the txdata 
            txdata.push((txn.clone(), fee, fee_rate));
        };
    }
    txdata
}

fn sort_transactions_by_fee_rate(transactions: &mut [(Transaction, u64, f64)]) {
    transactions.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap()); // Sort by fee_rate in descending order
}

fn add_transactions(transactions: Vec<(Transaction, u64, f64)>) -> (Vec<Transaction>, u64) {
    let mut txdata: Vec<Transaction> = Vec::new();
    let mut total_weight = 0;
    let mut total_fee = 0;
    for (tx, fee, _fee_rate) in transactions.iter() {
        total_weight += tx.weight().to_vbytes_ceil();
        total_fee += fee;

        if total_weight >= MAX_BLOCK_WEIGHT {
            break;
        }

        txdata.push(tx.clone());
    }

    (txdata, total_fee)
}


fn write_txdata_to_file(txdata: &[Transaction], output_file: &mut File) {
    for tx in txdata.iter() {
        let txid = tx.compute_ntxid();
        writeln!(output_file, "{}", txid).unwrap();
    }
}

fn main() {
    let path = "mempool";
    
    // deserialize the transactions from the mempool folder
    let transactions = get_transactions(path).unwrap();

    let miner_address = "bcrt1qgm4necqxnlz05a8he3cspjh63gt4vwlvsguhhg";
    let script_pubkey = output_scriptpk(miner_address);

    let mut txns_and_fees = select_transactions(transactions);

    sort_transactions_by_fee_rate(&mut txns_and_fees);
    // create a coinbase transaction
    
    let (mut txdata, total_fee) = add_transactions(txns_and_fees);

    let coinbase_transaction = coinbase_transaction(script_pubkey, total_fee).unwrap();

    //add the coinbase transaction to the txdata
    txdata.insert(0, coinbase_transaction.clone());

    let serialed_coinbase_tx = encode::serialize(&coinbase_transaction);
    let coinbase_tx_hex = serialed_coinbase_tx.as_hex();
    
    let target = get_target("0000ffff00000000000000000000000000000000000000000000000000000000").unwrap();

    let candidate_block = construct_candidate_block(txdata, target).unwrap();

    mine_block(candidate_block.header, target);

    let mut output_file = File::create("out.txt").unwrap();

    let binding = encode::serialize(&candidate_block.header);
    let block_header_hex = binding.as_hex();

    // Write the block header to the output file
    writeln!(output_file, "{}", block_header_hex).unwrap();
    writeln!(output_file, "{}", coinbase_tx_hex).unwrap();
    write_txdata_to_file(&candidate_block.txdata, &mut output_file);

}
