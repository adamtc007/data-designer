// This line prevents a console window from appearing on Windows in release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // Use the app library's run function
    app_lib::run();
}