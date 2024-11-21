use rusqlite::Connection;
use std::process::{Command};
use serde::Serialize;

#[derive(Debug ,Serialize)]
struct CommandStruct {
    id: i16,
    title: String,
    description: String,
    command: String,
}

fn run_hyprland_autostart_rust() -> Result<Vec<String>, std::io::Error> {
    let output = Command::new("./hyprIdalRust")
        .arg("--getAutoStart")
        .output();

    let mut all_commands = Vec::new();

    match output {
        Ok(output) => {
             if !output.stdout.is_empty() {
                let commands = String::from_utf8_lossy(&output.stdout);

                commands.split('\n').for_each(|command| {
                    if !command.is_empty() {
                        all_commands.push(command.to_string());
                    }
                });
            }

            Ok(all_commands)
        }
        Err(err) => {
            eprintln!("Failed to run hyprIdalRust: {}", err);
            Err(err)
        }
    }
}

#[tauri::command]
fn insert_new_records(title_para: String, description_para: String, command_para: String ) -> Result<String ,String> {
    let conn = Connection::open("hyprautogui.db").expect("Failed to open hyprautogui.db");

    match conn.execute(
        "INSERT INTO command ('title' ,'description' ,'command') VALUES (?1, ?2, ?3)",
        &[&title_para, &description_para, &command_para],
    ) {
        Ok(_) => {
            println!("Successfully inserted record!");
            conn.close().expect("Failed to close connection");
            Ok("Successfully inserted record!".to_string())
        },
        Err(err) => {
            println!("Failed to insert record! Error: {}", err);
            conn.close().expect("Failed to close connection");
            Err(String::from("Failed to insert new record!"))
        },
    }
}

#[tauri::command]
fn get_all_records() -> Result<Vec<CommandStruct> ,String> {
    let conn = Connection::open("hyprautogui.db").expect("Failed to open hyprautogui.db");

    let mut command_json = Vec::new();

    {
        let commands = conn.prepare("SELECT * FROM command");

        let mut binding = commands.unwrap();

        let command_records = binding.query_map([], |row| {
            Ok(CommandStruct {
                id: row.get(0).unwrap(),
                title: row.get(1).unwrap(),
                description: row.get(2).unwrap(),
                command: row.get(3).unwrap(),
            })
        }).expect("fails");

        for command_record in command_records {
            command_json.push(command_record.unwrap());
        }
    }

    conn.close().expect("Failed to close connection");

    Ok(command_json)
}

#[tauri::command]
fn update_information_command(title: String ,description: String ,id: String) -> Result<String ,String> {
    let conn = Connection::open("hyprautogui.db").expect("Failed to open hyprautogui.db");

    match conn.execute(
        "update command set title=?1 ,description=?2 where id=?3",
        &[&title ,&description ,&id]
    ) {
        Ok(value) => {
            println!("{}" ,{value});
            Ok("success".to_string())
        }
        Err(err) => {
            Err(err.to_string())
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let commands_from_executable = match run_hyprland_autostart_rust() {
        Ok(commands) => commands,
        Err(err) => {
            eprintln!("Failed to run hyprIdalRust: {}", err);
            return;
        }
    };

    if commands_from_executable.is_empty() {
        eprintln!("No commands retrieved from hyprIdalRust");
        return;
    }

    // Get existing commands from the database
    let existing_records = match get_all_records() {
        Ok(records) => records,
        Err(err) => {
            eprintln!("Failed to fetch records from the database: {}", err);
            return;
        }
    };

    // Extract just the commands from the existing records for comparison
    let existing_commands: Vec<String> = existing_records.iter().map(|record| record.command.clone()).collect();

    // Compare and insert missing commands
    for command in commands_from_executable {
        if !existing_commands.contains(&command) {
            if let Err(err) = insert_new_records(
                "No Title Has Been Provided".to_string(),
                "No Description Has Been Provided".to_string(),
                command.clone(),
            ) {
                eprintln!("Failed to insert command into database: {}", err);
            } else {
                println!("Inserted missing command: {}", command);
            }
        }
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_sql::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![insert_new_records ,get_all_records ,update_information_command])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
