mod app;
use anyhow::Result;
use app::App;
// use app::Done;
use app::config::info::APP_ID;
use relm4::RelmApp;
fn main() -> Result<()> {
    let app = RelmApp::new(APP_ID);
    // setup::init()?;
    app.run::<App>(());
    Ok(())
}
