use solana_client::rpc_client::RpcClient;
use solana_sdk::{pubkey::Pubkey};
use serde::{Serialize};
use chrono::{Utc, Duration};
use anyhow::Result;

pub const USDC_MINT: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";

#[derive(Serialize)]
pub struct Transfer {
    pub date: String,
    pub amount: f64,
    pub direction: String, // "Sent" or "Received"
}

// Fetch recent confirmed signatures for the wallet
pub async fn fetch_usdc_transfers(wallet: String) -> Result<Vec<Transfer>> {
    let rpc_url = std::env::var("SOLANA_RPC_URL").unwrap_or("https://api.mainnet-beta.solana.com".to_string());
    let client = RpcClient::new(rpc_url);

    let wallet_pubkey = wallet.parse::<Pubkey>()?;

    // Fetch signatures for this wallet - limit to last 1000 (adjust as needed)
    let signatures = client.get_signatures_for_address_with_config(
        &wallet_pubkey,
        solana_client::rpc_client::GetConfirmedSignaturesForAddress2Config {
            limit: Some(1000),
            ..Default::default()
        },
    )?;

    let mut transfers = Vec::new();

    // Define time cutoff 24h ago
    let cutoff = Utc::now() - Duration::hours(24);

    for sig in signatures {
        if let Some(block_time) = sig.block_time {
            let tx_time = chrono::DateTime::<Utc>::from_utc(
                chrono::NaiveDateTime::from_timestamp(block_time, 0),
                Utc
            );

            if tx_time < cutoff {
                continue;
            }

            // Fetch confirmed transaction details
            if let Ok(tx) = client.get_confirmed_transaction(&sig.signature) {
                // Parse transaction instructions to find USDC token transfers involving wallet
                if let Some(transfer) = parse_usdc_transfer(tx.transaction.transaction.message.instructions, &wallet_pubkey) {
                    transfers.push(Transfer {
                        date: tx_time.to_rfc3339(),
                        amount: transfer.amount,
                        direction: transfer.direction,
                    });
                }
            }
        }
    }

    Ok(transfers)
}

// Dummy function outline: You develop the logic to parse instructions for USDC transfer amount and direction.
fn parse_usdc_transfer(
    _instructions: Vec<solana_sdk::instruction::CompiledInstruction>,
    _wallet: &Pubkey,
) -> Option<Transfer> {
    // Implement parsing logic using Solana Token Program specifics
    // This includes filtering for USDC Mint address, parsing inner instructions,
    // checking source and destination token accounts to determine direction
    // Return Transfer struct if relevant transfer found.

    None // Placeholder
              }
