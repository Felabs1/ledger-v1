use chrono::prelude::*;
use sha2::{Sha256, Digest};
use serde::{Serialize, Deserialize};
use std::error::Error; // We need this for the return type

// 1. DEFINE BLOCK
#[derive(Debug, Serialize, Deserialize, Clone)]
struct Block {
    timestamp: u64,
    data: String,
    prev_hash: String,
    hash: String,
}

impl Block {
    fn new(data: String, prev_hash: String) -> Self {
        let timestamp = Utc::now().timestamp_millis() as u64;
        let mut block = Block {
            timestamp,
            data,
            prev_hash,
            hash: String::new(),
        };
        block.hash = block.calculate_hash();
        block
    }

    fn calculate_hash(&self) -> String {
        let input = (self.timestamp, &self.data, &self.prev_hash);
        let input_json = serde_json::to_string(&input).unwrap();
        let mut hasher = Sha256::new();
        hasher.update(input_json);
        hex::encode(hasher.finalize())
    }
}

// 2. DEFINE BLOCKCHAIN
struct Blockchain {
    db: sled::Db, 
    current_hash: String,
}

impl Blockchain {
    // FIX 1: Return Result<Blockchain, ...> instead of Self
    // This allows us to use the '?' operator inside.
    fn new() -> Result<Blockchain, Box<dyn Error>> {
        let db = sled::open("my_db")?;

        let last_hash_bytes = db.get("LAST")?;

        let current_hash = match last_hash_bytes {
            // FIX 2: Handle the "Found" case correctly
            Some(bytes) => {
                // Convert bytes to String
                String::from_utf8(bytes.to_vec())?
            },
            // Handle the "Not Found" (First run) case
            None => {
                let genesis = Block::new("Genesis Block".to_string(), "0".to_string());
                let genesis_hash = genesis.hash.clone();
                let genesis_json = serde_json::to_string(&genesis)?;

                db.insert(genesis.hash.as_bytes(), genesis_json.as_bytes())?;
                db.insert("LAST", genesis.hash.as_bytes())?;

                genesis_hash
            }
        };
        
        // FIX 3: Wrap the return struct in Ok()
        Ok(Blockchain { db, current_hash })
    }

    // FIX 4: Return Result<(), ...> so we can use '?'
    fn add_block(&mut self, data: String) -> Result<(), Box<dyn Error>> {
        let new_block = Block::new(data, self.current_hash.clone());
        let new_hash = new_block.hash.clone();
        let new_block_json = serde_json::to_string(&new_block)?;

        self.db.insert(new_block.hash.as_bytes(), new_block_json.as_bytes())?;
        self.db.insert("LAST", new_block.hash.as_bytes())?;

        self.current_hash = new_hash;
        self.db.flush()?; // Ensure save to disk

        Ok(())
    }

    fn print_chain(&self) {
        let mut search_hash = self.current_hash.clone();
        println!("--- CHAIN ON DISK ---");

        loop {
            match self.db.get(search_hash.as_bytes()) {
                Ok(Some(bytes)) => {
                    let block_json = String::from_utf8(bytes.to_vec()).unwrap();
                    let block: Block = serde_json::from_str(&block_json).unwrap();

                    println!("Hash: {}", block.hash);
                    println!("Data: {}", block.data);
                    println!("Prev: {}\n", block.prev_hash);

                    if block.prev_hash == "0" {
                        break;
                    }
                    search_hash = block.prev_hash;
                },
                _ => break,
            }
        }
    }


    // Returns Ok(true) if valid, Ok(false) if corrupted
    fn is_chain_valid(&self) -> Result<bool, Box<dyn Error>> {
        let mut search_hash = self.current_hash.clone();
        
        loop {
            // 1. Get the block bytes from the DB
            match self.db.get(search_hash.as_bytes())? {
                Some(bytes) => {
                    let block_json = String::from_utf8(bytes.to_vec())?;
                    let block: Block = serde_json::from_str(&block_json)?;

                    // CHECK 1: Data Integrity
                    // We recalculate the hash using the data inside the block.
                    // If the data was edited, this calculated hash won't match the stored hash.
                    if block.hash != block.calculate_hash() {
                        println!("ERROR: Hash mismatch for block {}", block.hash);
                        return Ok(false);
                    }

                    // CHECK 2: Link Integrity
                    // (Implicit) We are using 'prev_hash' to find the next block. 
                    // If this pointer is wrong, the next DB lookup will fail or return the wrong block.
                    
                    // Stop at Genesis
                    if block.prev_hash == "0" {
                        println!("Chain valid. Genesis reached.");
                        break;
                    }
                    
                    // Move backwards
                    search_hash = block.prev_hash;
                },
                None => {
                    // We were looking for a block that should exist (because a prev_hash pointed to it)
                    // but we couldn't find it. The chain is broken.
                    println!("ERROR: Broken link! Could not find block: {}", search_hash);
                    return Ok(false);
                }
            }
        }
        
        Ok(true)
    }
}

fn main() {
    // We unwrap here because if the DB fails to load, we want to crash and see why.
    let mut chain = Blockchain::new().unwrap();
    println!("Blockchain loaded. Current tip: {}", chain.current_hash);

    // 1. Check validity on load
    match chain.is_chain_valid() {
        Ok(true) => println!("Integrity check passed: ✅"),
        Ok(false) => {
            println!("Integrity check failed: ❌");
            return; // Stop the program if the DB is corrupted
        }, 
        Err(e) => println!("Error during validation: {}", e),
    }

    // 2. Add a new block
    chain.add_block("Transaction: User A -> User B".to_string()).unwrap();
    println!("Added new block.");
    chain.print_chain();
}