mod app_registry;
mod bootstrap_application;

use bootstrap_application::Application;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    Application::build().await?.run().await
}