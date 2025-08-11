#[macro_use]
extern crate rocket;

mod handlers;
mod solana_client;
mod models;

use handlers::get_usdc_transfers;

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/api", routes![get_usdc_transfers])
}
