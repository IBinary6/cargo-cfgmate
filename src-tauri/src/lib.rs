mod config;

use config::{AdminStatus, BackupEntry, CargoConfig, RustupEnvStatus, RustupEnvWriteResult};
use serde::Serialize;
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;
use std::path::Path;
use std::process::Command;

const CREATE_NO_WINDOW: u32 = 0x08000000;

fn create_hidden_command(program: &str) -> Command {
    let mut cmd = Command::new(program);
    #[cfg(target_os = "windows")]
    cmd.creation_flags(CREATE_NO_WINDOW);
    cmd
}

#[cfg(target_os = "windows")]
fn decode_command_output(bytes: &[u8]) -> String {
    let sample_len = bytes.len().min(200);
    let zero_count = bytes.iter().take(sample_len).filter(|b| **b == 0).count();
    if zero_count > 0 {
        let mut utf16 = Vec::with_capacity((bytes.len() + 1) / 2);
        for chunk in bytes.chunks(2) {
            let lo = chunk[0];
            let hi = *chunk.get(1).unwrap_or(&0);
            utf16.push(u16::from_le_bytes([lo, hi]));
        }
        return String::from_utf16_lossy(&utf16).trim().to_string();
    }
    String::from_utf8_lossy(bytes).trim().to_string()
}

#[cfg(not(target_os = "windows"))]
fn decode_command_output(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).trim().to_string()
}

#[tauri::command]
fn get_config(path: Option<String>) -> Result<CargoConfig, String> {
    config::load_config(path.as_deref())
}

#[tauri::command]
fn save_config(config: CargoConfig, path: Option<String>) -> Result<(), String> {
    config::save_config(&config, path.as_deref())
}

#[tauri::command]
fn get_config_path() -> String {
    config::get_cargo_config_path()
        .to_string_lossy()
        .to_string()
}

#[tauri::command]
fn get_current_target() -> String {
    config::get_current_target()
}

#[tauri::command]
fn open_config_folder(path: Option<String>) -> Result<(), String> {
    let path = config::resolve_config_path(path.as_deref());
    let target = if path.is_dir() {
        path
    } else {
        path.parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf()
    };
    #[cfg(target_os = "windows")]
    {
        Command::new("explorer")
            .arg(&target)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }
    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(&target)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }
    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    {
        Command::new("xdg-open")
            .arg(&target)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }
    Ok(())
}

#[tauri::command]
fn open_folder(path: String) -> Result<(), String> {
    let target = std::path::PathBuf::from(&path);
    if !target.exists() {
        return Err("目录不存在".to_string());
    }

    #[cfg(target_os = "windows")]
    {
        Command::new("explorer")
            .arg(&target)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }
    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(&target)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }
    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    {
        Command::new("xdg-open")
            .arg(&target)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }
    Ok(())
}

#[tauri::command]
fn open_config_file(path: Option<String>) -> Result<(), String> {
    let path = config::resolve_config_path(path.as_deref());
    if !path.exists() {
        return Err("配置文件不存在".to_string());
    }
    #[cfg(target_os = "windows")]
    {
        let path_str = path.to_string_lossy().to_string();
        create_hidden_command("cmd")
            .args(["/C", "start", "", &path_str])
            .spawn()
            .map_err(|e| format!("Failed to open file: {}", e))?;
    }
    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(path)
            .spawn()
            .map_err(|e| format!("Failed to open file: {}", e))?;
    }
    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    {
        Command::new("xdg-open")
            .arg(path)
            .spawn()
            .map_err(|e| format!("Failed to open file: {}", e))?;
    }
    Ok(())
}

#[tauri::command]
fn check_command_exists(cmd: String) -> bool {
    if cfg!(target_os = "windows") {
        create_hidden_command("where")
            .arg(&cmd)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    } else {
        Command::new("which")
            .arg(&cmd)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}

#[tauri::command]
fn check_file_exists(path: String) -> bool {
    config::expand_path(&path).is_file()
}

#[tauri::command]
async fn get_admin_status() -> Result<AdminStatus, String> {
    tauri::async_runtime::spawn_blocking(config::get_admin_status)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_rustup_env_status() -> Result<RustupEnvStatus, String> {
    tauri::async_runtime::spawn_blocking(config::get_rustup_env_status)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn set_rustup_env(
    dist: Option<String>,
    root: Option<String>,
    scope: Option<String>,
) -> Result<RustupEnvWriteResult, String> {
    tauri::async_runtime::spawn_blocking(move || config::set_rustup_env(dist, root, scope))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn get_backup_dir(path: Option<String>) -> String {
    config::get_backup_dir(path.as_deref())
        .to_string_lossy()
        .to_string()
}

#[tauri::command]
async fn list_backups(path: Option<String>) -> Result<Vec<BackupEntry>, String> {
    let result = tauri::async_runtime::spawn_blocking(move || config::list_backups(path.as_deref()))
        .await
        .map_err(|e| e.to_string())??;
    Ok(result)
}

#[tauri::command]
async fn create_backup(path: Option<String>, label: Option<String>) -> Result<BackupEntry, String> {
    let result = tauri::async_runtime::spawn_blocking(move || config::create_backup(path.as_deref(), label))
        .await
        .map_err(|e| e.to_string())??;
    Ok(result)
}

#[tauri::command]
async fn restore_backup(path: Option<String>, name: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || config::restore_backup(path.as_deref(), name))
        .await
        .map_err(|e| e.to_string())??;
    Ok(())
}

#[tauri::command]
async fn clear_backups(path: Option<String>) -> Result<usize, String> {
    let result = tauri::async_runtime::spawn_blocking(move || config::clear_backups(path.as_deref()))
        .await
        .map_err(|e| e.to_string())??;
    Ok(result)
}

#[tauri::command]
async fn delete_backup(path: Option<String>, name: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || config::delete_backup(path.as_deref(), name))
        .await
        .map_err(|e| e.to_string())??;
    Ok(())
}

#[tauri::command]
async fn rename_backup(path: Option<String>, old_name: String, new_name: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || config::rename_backup(path.as_deref(), old_name, new_name))
        .await
        .map_err(|e| e.to_string())??;
    Ok(())
}

#[tauri::command]
async fn import_config(path: String) -> Result<CargoConfig, String> {
    let result = tauri::async_runtime::spawn_blocking(move || config::import_config_from_path(&path))
        .await
        .map_err(|e| e.to_string())??;
    Ok(result)
}

#[tauri::command]
async fn export_config(config: CargoConfig, path: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || config::export_config_to_path(&config, &path))
        .await
        .map_err(|e| e.to_string())??;
    Ok(())
}

#[tauri::command]
async fn preview_config(config: CargoConfig) -> Result<String, String> {
    let result = tauri::async_runtime::spawn_blocking(move || config::serialize_config(&config))
        .await
        .map_err(|e| e.to_string())??;
    Ok(result)
}

#[tauri::command]
async fn preview_config_path(path: String) -> Result<String, String> {
    let result = tauri::async_runtime::spawn_blocking(move || {
        let config = config::import_config_from_path(&path)?;
        config::serialize_config(&config)
    })
    .await
    .map_err(|e| e.to_string())??;
    Ok(result)
}

#[tauri::command]
async fn install_sccache(_window: tauri::Window) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(|| {
        #[cfg(target_os = "windows")]
        let mut cmd = create_hidden_command("powershell");
        #[cfg(target_os = "windows")]
        cmd.args(["-Command", "cargo install sccache --locked"]);

        #[cfg(not(target_os = "windows"))]
        let mut cmd = Command::new("cargo");
        #[cfg(not(target_os = "windows"))]
        cmd.args(["install", "sccache", "--locked"]);

        let output = cmd.output().map_err(|e| e.to_string())?;
        if output.status.success() {
            Ok(())
        } else {
            Err(decode_command_output(&output.stderr))
        }
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn get_installed_targets() -> Result<Vec<String>, String> {
    let result = tauri::async_runtime::spawn_blocking(|| -> Result<Vec<String>, String> {
        let mut cmd = if cfg!(target_os = "windows") {
            create_hidden_command("rustup")
        } else {
            Command::new("rustup")
        };

        apply_rustup_env(&mut cmd);

        let output = cmd
            .args(["target", "list", "--installed"])
            .output()
            .map_err(|e| e.to_string())?;

        if output.status.success() {
            let stdout = decode_command_output(&output.stdout);
            let targets = stdout.lines().map(|s| s.trim().to_string()).collect();
            Ok(targets)
        } else {
            Err(decode_command_output(&output.stderr))
        }
    })
    .await
    .map_err(|e| e.to_string())??;
    Ok(result)
}

#[tauri::command]
async fn install_target(target: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        let mut cmd = if cfg!(target_os = "windows") {
            create_hidden_command("rustup")
        } else {
            Command::new("rustup")
        };

        apply_rustup_env(&mut cmd);

        let output = cmd
            .args(["target", "add", &target])
            .output()
            .map_err(|e| e.to_string())?;

        if output.status.success() {
            Ok(())
        } else {
            Err(decode_command_output(&output.stderr))
        }
    })
    .await
    .map_err(|e| e.to_string())?
}

#[derive(Debug, Serialize)]
pub struct CacheStats {
    registry_size: u64,
    registry_path: String,
    git_size: u64,
    git_path: String,
}

fn get_dir_size(path: &Path) -> u64 {
    let mut total_size = 0;
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries {
            if let Ok(entry) = entry {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_dir() {
                        total_size += get_dir_size(&entry.path());
                    } else {
                        total_size += metadata.len();
                    }
                }
            }
        }
    }
    total_size
}

fn apply_rustup_env(cmd: &mut Command) {
    let (dist, root) = config::get_rustup_env_effective();
    if let Some(dist) = dist {
        cmd.env("RUSTUP_DIST_SERVER", dist);
    } else {
        cmd.env_remove("RUSTUP_DIST_SERVER");
    }
    if let Some(root) = root {
        cmd.env("RUSTUP_UPDATE_ROOT", root);
    } else {
        cmd.env_remove("RUSTUP_UPDATE_ROOT");
    }
}

#[tauri::command]
async fn get_cargo_cache_stats() -> Result<CacheStats, String> {
    let result = tauri::async_runtime::spawn_blocking(|| -> Result<CacheStats, String> {
        let home = config::get_home_dir();
        let cargo_home = Path::new(&home).join(".cargo");

        let registry_path = cargo_home.join("registry");
        let git_path = cargo_home.join("git");

        let registry_size = if registry_path.exists() {
            get_dir_size(&registry_path)
        } else {
            0
        };
        let git_size = if git_path.exists() {
            get_dir_size(&git_path)
        } else {
            0
        };

        Ok(CacheStats {
            registry_size,
            registry_path: registry_path.to_string_lossy().to_string(),
            git_size,
            git_path: git_path.to_string_lossy().to_string(),
        })
    })
    .await
    .map_err(|e| e.to_string())??;
    Ok(result)
}

#[tauri::command]
async fn clean_cargo_cache(target: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        let home = config::get_home_dir();
        let cargo_home = Path::new(&home).join(".cargo");
        let path_to_clean = match target.as_str() {
            "registry" => cargo_home.join("registry"),
            "git" => cargo_home.join("git"),
            _ => return Err("Invalid target".to_string()),
        };

        if !path_to_clean.exists() {
            return Ok(());
        }

        // Safety check: ensure we are deleting inside .cargo
        if !path_to_clean.starts_with(&cargo_home) {
            return Err("Safety check failed: path not in .cargo".to_string());
        }

        std::fs::remove_dir_all(&path_to_clean).map_err(|e| e.to_string())?;
        std::fs::create_dir_all(&path_to_clean).map_err(|e| e.to_string())?;

        Ok(())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
fn exit_app(app: tauri::AppHandle) {
    app.exit(0);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            get_config,
            save_config,
            get_config_path,
            get_current_target,
            open_config_folder,
            open_folder,
            open_config_file,
            check_command_exists,
            check_file_exists,
            get_admin_status,
            get_rustup_env_status,
            set_rustup_env,
            get_backup_dir,
            list_backups,
            create_backup,
            restore_backup,
            clear_backups,
            delete_backup,
            rename_backup,
            import_config,
            export_config,
            preview_config,
            preview_config_path,
            install_sccache,
            get_installed_targets,
            install_target,
            get_cargo_cache_stats,
            clean_cargo_cache,
            exit_app
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
