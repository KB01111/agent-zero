#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::api::process::Command;
use tauri::Manager;
use dotenvy::dotenv;
use std::env;

#[tauri::command]
async fn proxy(path: String, method: String, body: Option<String>) -> Result<String, String> {
    let port = env::var("WEB_UI_PORT").unwrap_or_else(|_| "5000".into());
    let url = format!("http://127.0.0.1:{}/{}", port, path);
    let client = reqwest::Client::new();
    let req = match method.as_str() {
        "POST" => client.post(url).body(body.unwrap_or_default()),
        _ => client.get(url),
    };
    let resp = req.send().await.map_err(|e| e.to_string())?;
    let text = resp.text().await.map_err(|e| e.to_string())?;
    Ok(text)
}

fn main() {
    // load environment variables from the project root
    dotenv().ok();
    tauri::Builder::default()
        .setup(|app| {
            let envs: Vec<(String, String)> = env::vars().collect();
            tauri::async_runtime::spawn(async move {
                let mut cmd = Command::new("python");
                cmd.args(["run_ui.py"]).current_dir("..");
                for (k, v) in envs {
                    cmd.env(&k, &v);
                }
                if let Ok(mut child) = cmd.spawn() {
                    while let Some(_event) = child.rx.recv().await {};
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![proxy])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
