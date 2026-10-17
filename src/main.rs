// Entry point. Module wiring only; all logic lives in the modules below.

#![cfg_attr(windows, windows_subsystem = "windows")]

mod app;
mod backend;
mod assets;
mod message;
mod state;
mod theme;
mod update;
mod views;

fn main() -> iced::Result {
    app::run()
}