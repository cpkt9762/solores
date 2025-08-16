//! Integration test for transaction parsing with real Solana RPC data
//! 
//! This test fetches actual transaction data from Solana RPC in Base64 format
//! and compares the parsed results with existing JSON snapshot data.

use std::fs;
use serde_json::Value;
use sol_idl_core::{
    mata_parser::parse_encoded_confirmed_transaction,
    instruction::InstructionUpdate,
};
use solana_client::{
    rpc_client::RpcClient,
    rpc_config::RpcTransactionConfig,
};
use solana_sdk::{
    commitment_config::CommitmentConfig,
    signature::Signature,
};
use solana_transaction_status::{
    EncodedConfirmedTransactionWithStatusMeta,
    UiTransactionEncoding,
    option_serializer::OptionSerializer,
};
use tokio;

/// Test signature for comparison
const TEST_SIGNATURE: &str = "3js7grEiXVmNugAsRnSHd87GxeNrmPMsLfVCv3QceUhxQevUikVNZhXHV8McXfZ2AaTRLSAMYyjie3BxBnF8JJsu";

/// JSON snapshot path for comparison
const JSON_SNAPSHOT_PATH: &str = "/Users/pingzi/Developer/work/solana/solana-arbitrage/tools/tx-snapshot-dump/snapshots/3js7grEiXVmNugAsRnSHd87GxeNrmPMsLfVCv3QceUhxQevUikVNZhXHV8McXfZ2AaTRLSAMYyjie3BxBnF8JJsu/jsonparser.json";

/// Comparison result structure
#[derive(Debug)]
pub struct ComparisonResult {
    pub basic_info_matches: bool,
    pub instruction_count_matches: bool,
    pub program_ids_match: bool,
    pub fee_matches: bool,
    pub compute_units_match: bool,
    pub error_status_matches: bool,
    pub differences: Vec<String>,
}

impl ComparisonResult {
    pub fn is_success(&self) -> bool {
        self.basic_info_matches
            && self.instruction_count_matches 
            && self.program_ids_match
            && self.fee_matches
            && self.compute_units_match
            && self.error_status_matches
    }
    
    pub fn print_summary(&self) {
        println!("=== Transaction Parsing Comparison Result ===");
        println!("✅ Basic Info: {}", self.basic_info_matches);
        println!("✅ Instruction Count: {}", self.instruction_count_matches);
        println!("✅ Program IDs: {}", self.program_ids_match);
        println!("✅ Fee: {}", self.fee_matches);
        println!("✅ Compute Units: {}", self.compute_units_match);
        println!("✅ Error Status: {}", self.error_status_matches);
        
        if !self.differences.is_empty() {
            println!("⚠️  Differences found:");
            for diff in &self.differences {
                println!("   - {}", diff);
            }
        }
        
        println!("🎯 Overall Result: {}", if self.is_success() { "SUCCESS" } else { "NEEDS REVIEW" });
    }
}

/// Main integration test function
#[test]
fn test_rpc_vs_json_snapshot_comparison() {
    println!("🚀 Starting transaction parsing integration test...");
    
    // Parse the test signature
    let signature = TEST_SIGNATURE.parse::<Signature>()
        .expect("Failed to parse test signature");

    // Step 1: Fetch transaction from RPC in Base64 format
    println!("📡 Fetching transaction from RPC (Base64 format)...");
    let rpc_transaction = fetch_transaction_from_rpc(&signature)
        .expect("Failed to fetch transaction from RPC");

    // Step 2: Parse using our new parser
    println!("🔧 Parsing transaction with our parser...");
    let parsed_instructions = parse_encoded_confirmed_transaction(&rpc_transaction, rpc_transaction.slot)
        .expect("Failed to parse transaction with our parser");

    // Step 3: Load JSON snapshot for comparison
    println!("📄 Loading JSON snapshot for comparison...");
    let json_snapshot = load_json_snapshot()
        .expect("Failed to load JSON snapshot");

    // Step 4: Compare results
    println!("🔍 Comparing parsed results with JSON snapshot...");
    let comparison = compare_with_json_snapshot(&parsed_instructions, &rpc_transaction, &json_snapshot);

    // Step 5: Print detailed results
    comparison.print_summary();

    // Assert the test passes
    assert!(comparison.is_success(), "Transaction parsing comparison failed! Check differences above.");

    println!("✅ Integration test completed successfully!");
}

/// Fetch transaction from Solana RPC in Base64 format
fn fetch_transaction_from_rpc(
    signature: &Signature,
) -> Result<EncodedConfirmedTransactionWithStatusMeta, Box<dyn std::error::Error>> {
    let client = RpcClient::new("https://api.mainnet-beta.solana.com");
    
    let transaction = client
        .get_transaction_with_config(
            signature,
            RpcTransactionConfig {
                encoding: Some(UiTransactionEncoding::Base64),
                max_supported_transaction_version: Some(0),
                commitment: Some(CommitmentConfig::finalized()),
            },
        )?;

    Ok(transaction)
}

/// Load JSON snapshot from file
fn load_json_snapshot() -> Result<Value, Box<dyn std::error::Error>> {
    let json_content = fs::read_to_string(JSON_SNAPSHOT_PATH)?;
    let json_value: Value = serde_json::from_str(&json_content)?;
    Ok(json_value)
}

/// Compare parsed instructions with JSON snapshot
fn compare_with_json_snapshot(
    parsed_instructions: &[InstructionUpdate],
    rpc_transaction: &EncodedConfirmedTransactionWithStatusMeta,
    json_snapshot: &Value,
) -> ComparisonResult {
    let mut result = ComparisonResult {
        basic_info_matches: true,
        instruction_count_matches: true,
        program_ids_match: true,
        fee_matches: true,
        compute_units_match: true,
        error_status_matches: true,
        differences: Vec::new(),
    };

    // Compare basic transaction info
    if let Some(meta) = &rpc_transaction.transaction.meta {
        // Compare fee
        if let Some(json_fee) = json_snapshot["meta"]["fee"].as_u64() {
            if meta.fee != json_fee {
                result.fee_matches = false;
                result.differences.push(format!(
                    "Fee mismatch: parsed={}, json={}", meta.fee, json_fee
                ));
            }
        }

        // Compare compute units
        if let Some(json_compute) = json_snapshot["meta"]["computeUnitsConsumed"].as_u64() {
            match &meta.compute_units_consumed {
                OptionSerializer::Some(computed) => {
                    if *computed != json_compute {
                        result.compute_units_match = false;
                        result.differences.push(format!(
                            "Compute units mismatch: parsed={}, json={}", computed, json_compute
                        ));
                    }
                }
                _ => {
                    result.compute_units_match = false;
                    result.differences.push("Compute units not found in parsed result".to_string());
                }
            }
        }

        // Compare error status
        let json_err_is_null = json_snapshot["meta"]["err"].is_null();
        let parsed_err_is_none = meta.err.is_none();
        if json_err_is_null != parsed_err_is_none {
            result.error_status_matches = false;
            result.differences.push(format!(
                "Error status mismatch: json_null={}, parsed_none={}", 
                json_err_is_null, parsed_err_is_none
            ));
        }
    }

    // Count total instructions (including inner instructions)
    let parsed_total_count = count_total_instructions(parsed_instructions);
    let json_instruction_count = count_json_instructions(json_snapshot);
    
    // Compare instruction counts
    if parsed_total_count != json_instruction_count {
        result.instruction_count_matches = false;
        result.differences.push(format!(
            "Total instruction count mismatch: parsed={} ({}+inner), json={}", 
            parsed_total_count, parsed_instructions.len(), json_instruction_count
        ));
    }

    // Extract and compare program IDs
    let parsed_program_ids = extract_program_ids(parsed_instructions);
    let json_program_ids = extract_json_program_ids(json_snapshot);
    
    if parsed_program_ids != json_program_ids {
        result.program_ids_match = false;
        result.differences.push(format!(
            "Program IDs mismatch:\n  Parsed: {:?}\n  JSON: {:?}", 
            parsed_program_ids, json_program_ids
        ));
    }

    result
}

/// Count total instructions from parsed results (including inner instructions)
fn count_total_instructions(instructions: &[InstructionUpdate]) -> usize {
    instructions.iter().map(|inst| {
        1 + inst.inner.len()  // Main instruction + inner instructions
    }).sum()
}

/// Count total instructions in JSON (including inner instructions)
fn count_json_instructions(json: &Value) -> usize {
    let mut count = 0;
    
    // Count main transaction instructions
    if let Some(transaction) = json.get("transaction") {
        if let Some(message) = transaction.get("message") {
            if let Some(instructions) = message.get("instructions").and_then(|v| v.as_array()) {
                count += instructions.len();
            }
        }
    }
    
    // Count inner instructions
    if let Some(inner_instructions) = json["meta"]["innerInstructions"].as_array() {
        for inner_group in inner_instructions {
            if let Some(instructions) = inner_group["instructions"].as_array() {
                count += instructions.len();
            }
        }
    }
    
    count
}

/// Extract program IDs from parsed instructions
fn extract_program_ids(instructions: &[InstructionUpdate]) -> Vec<String> {
    let mut program_ids = Vec::new();
    
    for instruction in instructions {
        // Convert program ID bytes to base58 string
        let program_id_str = bs58::encode(&instruction.program).into_string();
        program_ids.push(program_id_str);
        
        // Also check inner instructions
        for inner in &instruction.inner {
            let inner_program_id = bs58::encode(&inner.program).into_string();
            program_ids.push(inner_program_id);
        }
    }
    
    program_ids.sort();
    program_ids.dedup();
    program_ids
}

/// Extract program IDs from JSON snapshot
fn extract_json_program_ids(json: &Value) -> Vec<String> {
    let mut program_ids = Vec::new();
    
    // Extract from main instructions
    if let Some(transaction) = json.get("transaction") {
        if let Some(message) = transaction.get("message") {
            if let Some(instructions) = message.get("instructions").and_then(|v| v.as_array()) {
                for instruction in instructions {
                    if let Some(program_id) = instruction.get("programId").and_then(|v| v.as_str()) {
                        program_ids.push(program_id.to_string());
                    }
                }
            }
        }
    }
    
    // Extract from inner instructions
    if let Some(inner_instructions) = json["meta"]["innerInstructions"].as_array() {
        for inner_group in inner_instructions {
            if let Some(instructions) = inner_group["instructions"].as_array() {
                for instruction in instructions {
                    if let Some(program_id) = instruction.get("programId").and_then(|v| v.as_str()) {
                        program_ids.push(program_id.to_string());
                    }
                }
            }
        }
    }
    
    program_ids.sort();
    program_ids.dedup();
    program_ids
}

/// Test for debugging - print detailed parsed instruction info
#[test]
#[ignore] // Use with `cargo test -- --ignored` to run
fn debug_print_parsed_transaction() {
    let signature = TEST_SIGNATURE.parse::<Signature>().unwrap();
    
    println!("🔍 Fetching and parsing transaction for debugging...");
    let rpc_transaction = fetch_transaction_from_rpc(&signature).unwrap();
    let parsed_instructions = parse_encoded_confirmed_transaction(&rpc_transaction, rpc_transaction.slot).unwrap();

    println!("📊 Parsed Transaction Details:");
    println!("   Slot: {}", rpc_transaction.slot);
    println!("   Block Time: {:?}", rpc_transaction.block_time);
    println!("   Instructions Count: {}", parsed_instructions.len());
    
    for (i, instruction) in parsed_instructions.iter().enumerate() {
        println!("   Instruction {}: Program = {}", 
                 i, bs58::encode(&instruction.program).into_string());
        println!("     Accounts: {}", instruction.accounts.len());
        println!("     Data Length: {}", instruction.data.len());
        println!("     Inner Instructions: {}", instruction.inner.len());
    }

    if let Some(meta) = &rpc_transaction.transaction.meta {
        println!("📈 Transaction Meta:");
        println!("   Fee: {}", meta.fee);
        println!("   Pre Balances: {}", meta.pre_balances.len());
        println!("   Post Balances: {}", meta.post_balances.len());
    }
}