mod adapters;
mod app;
mod core;
mod diagnostics;
mod profiles;
mod ui;

fn main() -> anyhow::Result<()> {
    app::run()
}
