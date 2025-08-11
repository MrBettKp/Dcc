#[macro_use] extern crate rocket;

use solana_usdc_indexer::handlers::get_usdc_transfers;

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/api", routes![get_usdc_transfers])
}
