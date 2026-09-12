mod app;

use digital_canvas::{initialize_logging, setup_panic_hook, Application};
use anyhow::Result;

fn main() -> Result<()> {
    setup_panic_hook();
    initialize_logging();

    let mut app = Application::new()?;
    app.run()?;

    Ok(())
}