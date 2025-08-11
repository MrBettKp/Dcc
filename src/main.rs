#[macro_use] extern crate rocket;

mod handlers;
mod solana_client;
mod models;

use handlers::get_usdc_transfers;

#[launch]
fn rocket() -> _ {
    env_logger::init();
    rocket::build()
        .mount("/api", routes![get_usdc_transfers])
}
