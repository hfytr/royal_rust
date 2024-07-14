mod api;
mod app;

fn main() -> std::io::Result<()> {
    let mut app = app::App::new()?;
    app.run()?;
    Ok(())
}
