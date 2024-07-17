mod api;
mod app;

fn main() -> std::io::Result<()> {
    let client = api::RoyalClient::new();
    client.get_fiction(40920);
    let mut app = app::App::new()?;
    app.run()?;
    Ok(())
}
