use rocket::serde::json::Json;
use rocket::http::Status;
use crate::solana_client::{fetch_usdc_transfers, Transfer};

#[get("/usdc-transfers/<wallet_address>")]
pub async fn get_usdc_transfers(wallet_address: &str) -> Result<Json<Vec<Transfer>>, Status> {
    match fetch_usdc_transfers(wallet_address.to_string()).await {
        Ok(transfers) => Ok(Json(transfers)),
        Err(e) => {
            log::error!("Error fetching USDC transfers: {:?}", e);
            Err(Status::InternalServerError)
        }
    }
}
