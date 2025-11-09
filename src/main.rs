#![doc = include_str!("../README.md")]

mod app;

use app::App;
use clap::Parser;
use color_eyre::Result;
use ratatui;

use crate::app::Inputs;

fn main() -> Result<()> {
    let inputs = Inputs::parse();
    color_eyre::install()?;
    let terminal = ratatui::init();
    let mut app = App::new(inputs.file);
    let result = app.run(terminal);
    ratatui::restore();
    result
}
