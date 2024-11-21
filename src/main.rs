// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use rusqlite::Connection;

fn main() {

    let conn = Connection::open("hyprautogui.db").expect("Failed to open hyprautogui.db");

    match conn.execute(
        "CREATE TABLE IF NOT EXISTS command (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        title TEXT,
        description TEXT,
        command TEXT
    )",
        [],
    ) {
        Ok(create) => println!("Success to create {create}"),
        Err(e) => println!("Error to create: {e}"),
    };

    conn.close().expect("Failed to close hyprautogui.db");

    hypridelgui_lib::run();
}
