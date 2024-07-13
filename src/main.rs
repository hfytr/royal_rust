mod api;
use api::*;

fn main() {
    let mut client = RoyalClient::new();
    client.get_fiction(40920);
}
