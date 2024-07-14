mod api;
use api::*;

fn main() {
    let client = RoyalClient::new();
    client.get_fiction(36049);
}
