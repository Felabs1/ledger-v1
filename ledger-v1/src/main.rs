use chrono::prelude::*;
use sha2::{Sha256, Digest};
use serde::{Serialize, Deserialize};

// 1. DEFining the block struct
// we derive serialize so we can convert this struct to a json string strictly for hashing

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Block {
    timestamp: u64,
    data: String,
    prev_hash: String,
    hash: String,
}

impl Block {
    // constructor for a new block
    fn new(data: String, prev_hash: String) -> Self {
        let timestamp = Utc::now().timestamp_millis() as u64;
        let mut block = Block {
            timestamp,
            data,
            prev_hash,
            hash: String::new(), // calculated below
        };
        block.hash = block.calculate_hash();
        block
    }

    // concept: SERIALIZATION and hashing
    fn calculate_hash(&self) -> String {
        // we do not include the block's own hash in the input, obviously
        // we create a tuple of the data we want to hash

        let input = (self.timestamp, &self.data, &self.prev_hash);

        // serialize to JSON string
        let input_json = serde_json::to_string(&input).unwrap();

        // hash the string
        let mut hasher = Sha256::new();
        hasher.update(input_json);

        let result = hasher.finalize();

        hex::encode(result)
    }
}


// 2. DEFINING THE BLOCKCHAIN STRUCT
struct Blockchain {
    chain: Vec<Block>,
}

impl Blockchain {
    fn new() -> Self {
        // the first block(Genesis block) has no previous hash
        let genesis_block = Block::new("Genesis block".to_string(), "0".to_string());
        Blockchain {
            chain: vec![genesis_block],
        }
    }


    // concept: Linking
    // we take the hash of the *last* block and use it as the prev_hash for the *new* block
    fn add_block(&mut self, data: String) {
        let prev_block = self.chain.last().unwrap();
        let new_block = Block::new(data, prev_block.hash.clone());
        self.chain.push(new_block);
    }

    // VALIDATION
    // this loops through the chain and ensures two things
    // the data hasn't been tampered with (recalculating hash matches stored hash)
    // the blocks are correctly linked current.prev_hash matches previous hash
    fn is_valid(&self) -> bool{
        for i in 1..self.chain.len() {
            let current_block = &self.chain[i];
            let previous_block = &self.chain[i - 1];

            // check 1: Did the data inside the block chainge
            if current_block.hash != current_block.calculate_hash() {
                println!("block {} data tampered", i);
                return false;
            }

            // check 2: Is the link broken
            if current_block.prev_hash != previous_block.hash {
                println!("block {} link broken", i);
                return false;
            }
        }

        true
    }
}


fn main() {
    let mut ledger = Blockchain::new();

    // add legitimate blocks
    println!("Mining block 1...");
    ledger.add_block("Transaction: Alice pays bob 5 Btc".to_string());

    println!("Mining block 2...");
    ledger.add_block("Transaction: Bob pays charlie 2 btc".to_string());


    // print the ledger
    println!("\n current ledger");
    for block in &ledger.chain {
        println!("hash {} prev {} data {}", block.hash, block.prev_hash, block.data);
    }

    println!("\nIs blockchain valid? {}", ledger.is_valid());

    println!("---------------------------------------");
    println!("TAMPERING ATTACK IN PROGRESS...");
    println!("---------------------------------------");


    // attack: change the data in the second block
    // we use mutable access to the chain to simulate a database hack
    ledger.chain[1].data = "Transaction: Alice pays bob 1000 BTC".to_string();

    println!("\nis blockchain valid? {}\n", ledger.is_valid());

    // even if the attacker is smart and recalculates the hash for that block
    ledger.chain[1].hash = ledger.chain[1].calculate_hash();
    println!("attacker recalculated hash ...");

    println!("is blockchain valid? {}", ledger.is_valid());
}