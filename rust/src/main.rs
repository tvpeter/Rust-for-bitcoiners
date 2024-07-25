use std::{error::Error, fmt, fs};
use serde::{Deserialize, Serialize};
use std::path::Path;
use walkdir::WalkDir;

#[derive(Debug)]
enum TransactionError {
    InvalidFileExtension,
    DirectoryNotFound,
    ReadFileError(std::io::Error),
    ParseJsonError(serde_json::Error),
}

impl fmt::Display for TransactionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TransactionError::InvalidFileExtension => write!(f, "Invalid file extension"),
            TransactionError::DirectoryNotFound => write!(f, "Directory not found"),
            TransactionError::ReadFileError(err) => write!(f, "Error reading file: {}", err),
            TransactionError::ParseJsonError(err) => write!(f, "Error parsing JSON: {}", err),
        }
    }
}

impl Error for TransactionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            TransactionError::ReadFileError(err) => Some(err),
            TransactionError::ParseJsonError(err) => Some(err),
            _ => None,
        }
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
struct Transaction {
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

fn get_transactions(path: &str) -> Result<Vec<Transaction>, Box<dyn Error>> {
    let mut transactions: Vec<Transaction> = Vec::new();

    if !Path::new(path).is_dir() {
        return Err(Box::new(TransactionError::DirectoryNotFound));
    }

    let directory = WalkDir::new(path);

    for file in directory.into_iter().filter_map(|e| e.ok()) {
        if file.file_type().is_file() {
            let file_path = file.path();

            // Check if the file is a json file
            if file_path.extension().and_then(|s| s.to_str()) != Some("json") {
                return Err(Box::new(TransactionError::InvalidFileExtension));
            }

            let content = fs::read_to_string(file_path).map_err(TransactionError::ReadFileError)?;
            let transaction: Transaction = serde_json::from_str(&content).map_err(TransactionError::ParseJsonError)?;
            transactions.push(transaction);
        }
    }
    Ok(transactions)
}

    


fn main() {
let path = "mempool";
    
    let transactions = get_transactions(path).unwrap();

    
    
}
