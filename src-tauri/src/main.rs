// This line prevents a console window from appearing on Windows in release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// We use our library by its package name, `data_designer`
use data_designer::transpile_dsl_to_rules;

// This is the command that your JavaScript frontend can call.
#[tauri::command]
fn save_rules(dsl_text: String) -> Result<(), String> {
    // 1. Save the human-readable .rules file
    // Note: In a real app, you'd let the user choose a file path.
    std::fs::write("my_rules.rules", &dsl_text).map_err(|e| e.to_string())?;

    // 2. Transpile the DSL to JSON using the function from our library
    let json_output = transpile_dsl_to_rules(&dsl_text)?;

    // 3. Save the machine-readable rules.json
    std::fs::write(
        "rules.json",
        serde_json::to_string_pretty(&json_output).unwrap(),
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

fn main() {
    // This is the entry point of your application.
    // It builds and runs the Tauri window.
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![save_rules])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
