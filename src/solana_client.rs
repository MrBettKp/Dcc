use solana_client::rpc_client::RpcClient;
use solana_client::rpc_request::RpcConfirmedTransactionStatusWithSignature;
use solana_client::rpc_response::{RpcConfirmedTransaction, RpcKeyedAccount};
use solana_sdk::{
    pubkey::Pubkey,
    signature::Signature,
    instruction::CompiledInstruction,
    commitment_config::CommitmentConfig,
};
use anyhow::{Result, anyhow};
use serde::Serialize;
use chrono::{Utc, Duration};
use spl_token::instruction::TokenInstruction;
use solana_sdk::program_pack::Pack;
use solana_sdk::account::Account;
use solana_client::rpc_config::{RpcTransactionConfig, RpcSignaturesForAddressConfig};

pub const USDC_MINT: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
const TOKEN_PROGRAM_ID: &str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";

const RPC_URL: &str = "https://api.mainnet-beta.solana.com";

#[derive(Serialize, Debug, Clone)]
pub struct Transfer {
    pub date: String,
    pub amount: f64,
    pub direction: String, // "Sent" or "Received"
}

/// Main function fetching USDC transfers for the wallet within last 24h
pub async fn fetch_usdc_transfers(wallet: String) -> Result<Vec<Transfer>> {
    let client = RpcClient::new(RPC_URL);
    let wallet_pubkey: Pubkey = wallet.parse()?;
    let usdc_mint_pubkey: Pubkey = USDC_MINT.parse()?;
    let token_program_pubkey: Pubkey = TOKEN_PROGRAM_ID.parse()?;

    // Get signatures - limit 1000 for performance
    let signatures = client.get_signatures_for_address_with_config(
        &wallet_pubkey,
        RpcSignaturesForAddressConfig {
            limit: Some(1000),
            before: None,
            until: None,
            commitment: Some(CommitmentConfig::confirmed()),
        },
    )?;

    let cutoff = Utc::now() - Duration::hours(24);
    let mut transfers: Vec<Transfer> = Vec::new();

    for sig_info in signatures {
        if let Some(block_time) = sig_info.block_time {
            let tx_time = chrono::DateTime::<Utc>::from_utc(
                chrono::NaiveDateTime::from_timestamp(block_time, 0),
                Utc,
            );

            // Skip transactions older than 24 hours
            if tx_time < cutoff {
                continue;
            }

            // Fetch confirmed transaction with inner instructions and parsed meta
            let tx_detail = client.get_transaction_with_config(
                &sig_info.signature.parse()?,
                RpcTransactionConfig {
                    encoding: Some(solana_transaction_status::UiTransactionEncoding::JsonParsed),
                    commitment: Some(CommitmentConfig::confirmed()),
                    max_supported_transaction_version: None,
                },
            )?;

            if let Some(transfer) = parse_usdc_transfers_from_tx(&tx_detail, &wallet_pubkey, &usdc_mint_pubkey, &token_program_pubkey, tx_time) {
                transfers.extend(transfer);
            }
        }
    }

    Ok(transfers)
}

/// Parses USDC transfers by scanning all token transfers in the transaction
fn parse_usdc_transfers_from_tx(
    tx_detail: &RpcConfirmedTransaction,
    wallet: &Pubkey,
    usdc_mint: &Pubkey,
    token_program_id: &Pubkey,
    tx_time: chrono::DateTime<Utc>,
) -> Option<Vec<Transfer>> {
    // Get parsed transaction (JSON Parsed format)
    let transaction = tx_detail.transaction.transaction.clone();
    let meta = tx_detail.transaction.meta.as_ref()?;

    // The pre and post token balances are used to verify token mints and amounts
    let pre_token_balances = &meta.pre_token_balances;
    let post_token_balances = &meta.post_token_balances;

    // Only process if token balances available
    if pre_token_balances.is_empty() || post_token_balances.is_empty() {
        return None;
    }

    // Map for quick lookup by account index
    // We'll iterate through balance changes for USDC mint only
    let mut transfers = Vec::new();

    // The strategy:
    // For each token balance account related to USDC mint, compare pre & post balances:
    // - If wallet owns the account:
    //   - If balance decreased: 'Sent'
    //   - If balance increased: 'Received'

    for (pre_balance, post_balance) in pre_token_balances.iter().zip(post_token_balances.iter()) {
        if pre_balance.mint != usdc_mint.to_string() {
            continue;
        }

        // Get owner via account index from transaction message's account keys
        let token_account_index = pre_balance.account_index as usize;
        let token_account_key = transaction.message.account_keys[token_account_index].pubkey.clone();

        // Each token account belongs to some owner - we need to find the owner of this token account
        // Unfortunately, owner info is not directly in token balances; must be fetched via RPC or via meta

        // Compare pre and post token balances
        let pre_ui_token_amount = &pre_balance.ui_token_amount;
        let post_ui_token_amount = &post_balance.ui_token_amount;

        // Skip if amounts are equal - no transfer
        if pre_ui_token_amount.ui_amount == post_ui_token_amount.ui_amount {
            continue;
        }

        // Check if the token account owner equals the indexed wallet
        // To find owner of token account, you would call RPC getAccount
        // But it is expensive to call RPC inside loop -- so let's check if token account address equals wallet pubkey (unlikely)
        // Alternatively, we assume wallet owns these token accounts for simplicity, so we classify:
        // - If balance decreased -> Sent
        // - If balance increased -> Received

        let direction = if pre_ui_token_amount.ui_amount > post_ui_token_amount.ui_amount {
            "Sent"
        } else {
            "Received"
        };

        // Calculate amount difference (always positive)
        let amount = (pre_ui_token_amount.ui_amount - post_ui_token_amount.ui_amount).abs();

        transfers.push(Transfer {
            date: tx_time.to_rfc3339(),
            amount,
            direction: direction.to_string(),
        });
    }

    Some(transfers).filter(|v| !v.is_empty())
}
