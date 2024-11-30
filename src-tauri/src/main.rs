// src-tauri/src/main.rs
use tauri::Manager;
use serde::{Serialize, Deserialize};
use tokio::process::Command;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
struct WPSite {
    id: String,
    name: String,
    path: PathBuf,
    wp_cli_path: PathBuf,
    environment: Environment,
}

#[derive(Debug, Serialize, Deserialize)]
enum Environment {
    Development,
    Staging,
    Production,
}

#[derive(Debug, Serialize, Deserialize)]
struct FileOperation {
    operation_type: FileOperationType,
    source: PathBuf,
    destination: PathBuf,
}

#[derive(Debug, Serialize, Deserialize)]
enum FileOperationType {
    Copy,
    Move,
    Delete,
    Chmod,
}

// WP-CLI Bridge Implementation
#[tauri::command]
async fn execute_wp_cli(site: WPSite, command: Vec<String>) -> Result<String, String> {
    let mut cmd = Command::new(&site.wp_cli_path);
    cmd.args(command);
    cmd.current_dir(&site.path);
    
    match cmd.output().await {
        Ok(output) => {
            if output.status.success() {
                Ok(String::from_utf8_lossy(&output.stdout).to_string())
            } else {
                Err(String::from_utf8_lossy(&output.stderr).to_string())
            }
        }
        Err(e) => Err(format!("Failed to execute WP-CLI: {}", e)),
    }
}

// File Operations Layer
#[tauri::command]
async fn handle_file_operation(operation: FileOperation) -> Result<(), String> {
    match operation.operation_type {
        FileOperationType::Copy => {
            tokio::fs::copy(operation.source, operation.destination)
                .await
                .map_err(|e| format!("Copy failed: {}", e))?;
        }
        FileOperationType::Move => {
            tokio::fs::rename(operation.source, operation.destination)
                .await
                .map_err(|e| format!("Move failed: {}", e))?;
        }
        FileOperationType::Delete => {
            tokio::fs::remove_file(operation.source)
                .await
                .map_err(|e| format!("Delete failed: {}", e))?;
        }
        FileOperationType::Chmod => {
            // Implementation for chmod using platform-specific operations
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let metadata = tokio::fs::metadata(&operation.source)
                    .await
                    .map_err(|e| format!("Failed to get metadata: {}", e))?;
                let mut perms = metadata.permissions();
                perms.set_mode(0o755); // Example permission
                tokio::fs::set_permissions(&operation.source, perms)
                    .await
                    .map_err(|e| format!("Chmod failed: {}", e))?;
            }
        }
    }
    Ok(())
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            execute_wp_cli,
            handle_file_operation
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}