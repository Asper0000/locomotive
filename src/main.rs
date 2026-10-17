// Entry point. Module wiring only; all logic lives in the modules below.

#![windows_subsystem = "windows"]

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