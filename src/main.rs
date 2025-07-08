mod tui;

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    tui::run().await
}
