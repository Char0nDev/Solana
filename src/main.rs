use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig, pubkey::Pubkey, signature::Keypair, signer::Signer, system_instruction
};
use std::{env, fs, path::Path};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Config {
    json_rpc_url: String,
    websocket_url: String,
    keypair_path: String,
    address_labels: std::collections::HashMap<String, String>,
    commitment: String,
}

fn main() {
    let config = get_config();

    let payer_keypair_path = config.keypair_path;
    let payer_secret_key_string = fs::read_to_string(payer_keypair_path).unwrap();
    let payer_secret_key: Vec<u8> = serde_json::from_str(&payer_secret_key_string).unwrap();

    let rpc_url = String::from(config.json_rpc_url);
    let rpc_client = RpcClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed());


    let payer = Keypair::from_bytes(&payer_secret_key).unwrap();
    let base = Keypair::new();

    println!("{}" , payer.pubkey().to_string());

    let seed = "charondev";
    let program_id = solana_program::system_program::id();
    let derived_pubkey = Pubkey::create_with_seed(&base.pubkey(), seed, &program_id).unwrap();


    let space = 10;
    let rent_exemption_amount = rpc_client.get_minimum_balance_for_rent_exemption(space).unwrap();
    let create_account_with_seed_ix = system_instruction::create_account_with_seed(
        &payer.pubkey(),
        &derived_pubkey,
        &base.pubkey(),
        seed,
        rent_exemption_amount,
        space as u64,
        &program_id
    );

    let recent_blockhash = rpc_client.get_latest_blockhash().unwrap();
    let create_account_with_seed_tx  = solana_sdk::transaction::Transaction::new_signed_with_payer(
        &[create_account_with_seed_ix],
        Some(&payer.pubkey()),
        &[&payer, &base],
        recent_blockhash
    );

    let create_account_with_seed_tx_signature  = rpc_client
        .send_and_confirm_transaction(&create_account_with_seed_tx)
        .unwrap();

    println!("Transaction signature: {create_account_with_seed_tx_signature}");
    println!("New account {} created successfully", derived_pubkey);
}


fn get_config() -> Config {
    let home_dir = env::var("HOME").unwrap();
    let config_path = Path::new(&home_dir)
        .join(".config/solana/cli/config.yml");
    
    let config_file = fs::File::open(config_path).unwrap();
    let config: Config = serde_yaml::from_reader(config_file).unwrap();

    config
}