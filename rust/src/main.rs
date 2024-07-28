use std::{error::Error, fmt, fs::{self, File}, io::Write, str::FromStr, time::{ SystemTime, SystemTimeError, UNIX_EPOCH}};
use bitcoin::{ absolute::{Height, LockTime}, address::ParseError, block::{self, Header}, consensus::encode, error::UnprefixedHexError, hashes::Hash, hex::DisplayHex, psbt::serialize, script::Builder, transaction::Version, witness, Address, Amount, Block, BlockHash, CompactTarget, OutPoint, ScriptBuf, Sequence, Target, Transaction, TxIn, TxMerkleNode, TxOut, Txid, Witness};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::path::Path;
use walkdir::WalkDir;

#[derive(Debug)]
enum BlockMiningError {
    InvalidFileExtension,
    DirectoryNotFound,
    ReadFileError(std::io::Error),
    ParseJsonError(serde_json::Error),
    SystemTimeError(std::time::SystemTimeError),
    UnprefixedHexError,
    ConversionError,
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
            BlockMiningError::ConversionError => write!(f, "ConversionError"),
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

fn time_stamp() -> Result<u32, BlockMiningError> {
    let time = SystemTime::now().duration_since(UNIX_EPOCH)?;
    Ok(time.as_secs() as u32)
}

fn get_nonce() -> u32 {
    let mut rng = rand::thread_rng();
    rng.gen_range(0..u32::MAX)
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

fn coinbase_transaction(address: &str) -> Result<Transaction, BlockMiningError> {

    let coinbase_input = TxIn {
        previous_output: OutPoint {
            txid: Txid::all_zeros(),
            vout: u32::MAX,
        },
        script_sig: Builder::new().push_int(0).into_script(),
        sequence: bitcoin::Sequence(0xFFFFFFFF),
        witness: Witness::new(),
    };

    let address = Address::from_str(address).map_err(BlockMiningError::ParseError)?.assume_checked();

    let script_pubkey = address.script_pubkey();

    // coinbase transactions are valid after 100 blocks
    let height = Height::from_consensus(100).map_err(|e| BlockMiningError::ConversionError)?;
    let lock_time = LockTime::Blocks(height);
    
    Ok(Transaction {
        version: Version::ONE,
        lock_time, 
        input: vec![coinbase_input],
        output: vec![TxOut {
            script_pubkey,
            value: Amount::from_sat(3_125_000),
        }],
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
        version: bitcoin::block::Version::ONE,
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
    if tx.version < Version::ONE {
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


fn add_transactions(mut txdata: Vec<Transaction>, transactions: Vec<BTransaction>, mut output_file: &File) -> Vec<Transaction> {

    for tx in transactions.iter() {
        let txn = Transaction {
            version: Version(tx.version as i32),
            lock_time: LockTime::ZERO,
            input: convert_txin(&tx.vin),
            output: convert_vout(&tx.vout),
        };

        //validate the transaction, if it is valid, add it to the block
       if validate_transaction(&txn).is_ok() {
            // add the transaction to the txdata 
            txdata.push(txn.clone());
        };
    }
    txdata
}

fn write_txdata(txdata: &[Transaction], output_file: &mut File) {
    for tx in txdata.iter() {
        let txid = tx.compute_txid();
        writeln!(output_file, "{}", txid).unwrap();
    }
}

fn main() {
    let path = "mempool";
    
    // deserialize the transactions from the mempool folder
    let transactions = get_transactions(path).unwrap();

    let miner_address = "bcrt1qgm4necqxnlz05a8he3cspjh63gt4vwlvsguhhg";

    // create a coinbase transaction
    let coinbase_transaction = coinbase_transaction(miner_address).unwrap();

    let mut output_file = File::create("out.txt").unwrap();

    let serialed_coinbase_tx = encode::serialize(&coinbase_transaction);
    let coinbase_tx_hex = serialed_coinbase_tx.as_hex();

    let mut txdata: Vec<Transaction> = vec![coinbase_transaction.clone()];

    txdata = add_transactions(txdata, transactions, &output_file);

    let target = get_target("0000ffff00000000000000000000000000000000000000000000000000000000").unwrap();

    let candidate_block = construct_candidate_block(txdata, target).unwrap();

    mine_block(candidate_block.header, target);

    // Write the block header to the output file
    writeln!(output_file, "block hash: {}", candidate_block.header.block_hash()).unwrap();
    writeln!(output_file, "nonce: {}", candidate_block.header.nonce).unwrap();
    writeln!(output_file, "target: {}", Target::from_compact(candidate_block.header.bits)).unwrap();
    writeln!(output_file, "block time: {}", candidate_block.header.time).unwrap();
    writeln!(output_file, "serialized coinbase tx: {}", coinbase_tx_hex).unwrap();


   write_txdata(&candidate_block.txdata, &mut output_file);


   // sanity check
    assert!(candidate_block.check_merkle_root());
    assert!(candidate_block.total_size() > 1);
    assert!(coinbase_transaction.is_coinbase());
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nonce_generation() {
        let nonce = get_nonce();
        assert!(nonce > 0);
        assert!(nonce < u32::MAX);
    }

}
