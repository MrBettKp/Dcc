use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use serde::Serialize;
use chrono::{Utc, Duration};
use anyhow::Result;

pub const USDC_MINT: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";

#[derive(Serialize)]
pub struct Transfer {
    pub date: String,
    pub amount: f64,
    pub direction: String, // "Sent" or "Received"
}

// Async wrapper for RpcClient calls
pub async fn fetch_usdc_transfers(wallet: String) -> Result<Vec<Transfer>> {
    let rpc_url = std::env::var("SOLANA_RPC_URL").unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string());
    let client = RpcClient::new(rpc_url);

    let wallet_pubkey = wallet.parse::<Pubkey>()?;

    // Fetch signatures for the wallet
    let signatures = client.get_signatures_for_address_with_config(
        &wallet_pubkey,
        solana_client::rpc_client::GetConfirmedSignaturesForAddress2Config {
            limit: Some(1000),
            ..Default::default()
        },
    )?;

    let mut transfers = Vec::new();

    let cutoff = Utc::now() - Duration::hours(24);

    for sig in signatures {
        if let Some(block_time) = sig.block_time {
            let tx_time = chrono::DateTime::<Utc>::from_utc(
                chrono::NaiveDateTime::from_timestamp(block_time, 0),
                Utc,
            );

            if tx_time < cutoff {
                continue;
            }

            if let Ok(tx) = client.get_confirmed_transaction(&sig.signature) {
                // Implement your instruction parsing here
                if let Some(transfer) = parse_usdc_transfer(&tx.transaction.message.instructions, &wallet_pubkey) {
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

// Parsing placeholder - you need business logic here
fn parse_usdc_transfer(
    _instructions: &Vec<solana_sdk::instruction::CompiledInstruction>,
    _wallet: &Pubkey,
) -> Option<Transfer> {
    // Insert parsing logic here

    None
}
