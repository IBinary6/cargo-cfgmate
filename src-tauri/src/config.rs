use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

const RUSTUP_DIST_SERVER: &str = "RUSTUP_DIST_SERVER";
const RUSTUP_UPDATE_ROOT: &str = "RUSTUP_UPDATE_ROOT";
#[cfg(not(target_os = "windows"))]
const RUSTUP_MANAGED_START: &str = "# >>> quickchange rustup env >>>";
#[cfg(not(target_os = "windows"))]
const RUSTUP_MANAGED_END: &str = "# <<< quickchange rustup env <<<";

#[derive(Debug, Serialize, Clone)]
pub struct BackupEntry {
    pub name: String,
    pub path: String,
    pub modified: u64,
    pub size: u64,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct CargoConfig {
    pub source: Option<HashMap<String, SourceEntry>>,
    pub registry: Option<RegistryConfig>,
    pub net: Option<NetConfig>,
    pub http: Option<HttpConfig>,
    pub https: Option<HttpsConfig>,
    pub build: Option<BuildConfig>,
    pub target: Option<HashMap<String, TargetConfig>>,
    pub env: Option<HashMap<String, EnvValue>>,
    pub profile: Option<HashMap<String, ProfileConfig>>,
    pub alias: Option<HashMap<String, String>>,
    pub doc: Option<DocConfig>,
    pub registries: Option<HashMap<String, RegistryEntry>>, // Allow custom registries
    #[serde(flatten)]
    pub other: HashMap<String, toml::Value>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct RustupEnvScope {
    pub value: Option<String>,
    pub source: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct RustupEnvVarStatus {
    pub user: RustupEnvScope,
    pub system: RustupEnvScope,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct RustupEnvStatus {
    pub dist: RustupEnvVarStatus,
    pub root: RustupEnvVarStatus,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct RustupEnvWriteOutcome {
    pub ok: bool,
    pub error: Option<String>,
    #[serde(default)]
    pub skipped: bool,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct RustupEnvWriteResult {
    pub user: RustupEnvWriteOutcome,
    pub system: RustupEnvWriteOutcome,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct AdminStatus {
    pub is_admin: bool,
    pub hint: String,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct DocConfig {
    pub browser: Option<String>,
    #[serde(rename = "open-result")]
    pub open_result: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct RegistryEntry {
    pub index: Option<String>,
    pub token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum EnvValue {
    Simple(String),
    Object(EnvObject),
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct EnvObject {
    pub value: String,
    pub force: Option<bool>,
    pub relative: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct SourceEntry {
    pub registry: Option<String>,
    #[serde(rename = "replace-with")]
    pub replace_with: Option<String>,
    #[serde(flatten)]
    pub other: HashMap<String, toml::Value>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct RegistryConfig {
    pub default: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct NetConfig {
    pub offline: Option<bool>,
    #[serde(rename = "git-fetch-with-cli")]
    pub git_fetch_with_cli: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct HttpConfig {
    pub proxy: Option<String>,
    #[serde(rename = "check-revoke")]
    pub check_revoke: Option<bool>,
    pub multiplexing: Option<bool>,
    pub cainfo: Option<String>,
    pub timeout: Option<u32>,
    #[serde(rename = "low-speed-limit")]
    pub low_speed_limit: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct HttpsConfig {
    pub proxy: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct BuildConfig {
    pub jobs: Option<u32>,
    pub target: Option<String>,
    #[serde(rename = "rustc-wrapper")]
    pub rustc_wrapper: Option<String>,
    pub rustflags: Option<Vec<String>>,
    pub rustdocflags: Option<Vec<String>>,
    #[serde(flatten)]
    pub other: HashMap<String, toml::Value>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct TargetConfig {
    pub linker: Option<String>,
    pub ar: Option<String>,
    pub rustflags: Option<Vec<String>>,
    pub runner: Option<String>,
    #[serde(flatten)]
    pub other: HashMap<String, toml::Value>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct ProfileConfig {
    #[serde(rename = "opt-level")]
    pub opt_level: Option<toml::Value>,
    pub lto: Option<toml::Value>,
    #[serde(rename = "codegen-units")]
    pub codegen_units: Option<u32>,
    pub debug: Option<toml::Value>,
    pub strip: Option<toml::Value>,
    pub panic: Option<String>,
    #[serde(rename = "trim-paths")]
    pub trim_paths: Option<String>,
    #[serde(flatten)]
    pub other: HashMap<String, toml::Value>,
}

pub fn get_home_dir() -> String {
    if cfg!(windows) {
        std::env::var("USERPROFILE").unwrap_or_else(|_| ".".to_string())
    } else {
        std::env::var("HOME").unwrap_or_else(|_| ".".to_string())
    }
}

pub fn get_cargo_config_path() -> PathBuf {
    Path::new(&get_home_dir())
        .join(".cargo")
        .join("config.toml")
}

pub fn get_backup_dir(config_path: Option<&str>) -> PathBuf {
    let config_path = resolve_config_path(config_path);
    let base_dir = config_path.parent().unwrap_or_else(|| Path::new("."));
    base_dir.join("quickchange-backups")
}

fn expand_tilde(input: &str) -> String {
    let trimmed = input.trim();
    if trimmed == "~" {
        return get_home_dir();
    }
    if let Some(rest) = trimmed
        .strip_prefix("~/")
        .or_else(|| trimmed.strip_prefix("~\\"))
    {
        return Path::new(&get_home_dir())
            .join(rest)
            .to_string_lossy()
            .to_string();
    }
    trimmed.to_string()
}

#[cfg(windows)]
fn expand_env_vars(input: &str) -> String {
    let mut result = String::new();
    let mut chars = input.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '%' {
            let mut name = String::new();
            let mut closed = false;
            while let Some(next) = chars.next() {
                if next == '%' {
                    closed = true;
                    break;
                }
                name.push(next);
            }
            if closed && !name.is_empty() {
                match std::env::var(&name) {
                    Ok(val) => result.push_str(&val),
                    Err(_) => {
                        result.push('%');
                        result.push_str(&name);
                        result.push('%');
                    }
                }
            } else {
                result.push('%');
                result.push_str(&name);
            }
        } else {
            result.push(ch);
        }
    }
    result
}

#[cfg(not(windows))]
fn expand_env_vars(input: &str) -> String {
    let mut result = String::new();
    let mut chars = input.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '$' {
            if let Some('{') = chars.peek().copied() {
                chars.next();
                let mut name = String::new();
                while let Some(next) = chars.next() {
                    if next == '}' {
                        break;
                    }
                    name.push(next);
                }
                if !name.is_empty() {
                    match std::env::var(&name) {
                        Ok(val) => result.push_str(&val),
                        Err(_) => {
                            result.push_str("${");
                            result.push_str(&name);
                            result.push('}');
                        }
                    }
                } else {
                    result.push('$');
                }
            } else {
                let mut name = String::new();
                while let Some(next) = chars.peek().copied() {
                    if next.is_ascii_alphanumeric() || next == '_' {
                        chars.next();
                        name.push(next);
                    } else {
                        break;
                    }
                }
                if !name.is_empty() {
                    match std::env::var(&name) {
                        Ok(val) => result.push_str(&val),
                        Err(_) => {
                            result.push('$');
                            result.push_str(&name);
                        }
                    }
                } else {
                    result.push('$');
                }
            }
        } else {
            result.push(ch);
        }
    }
    result
}

pub fn expand_path(input: &str) -> PathBuf {
    let tilde_expanded = expand_tilde(input);
    let env_expanded = expand_env_vars(&tilde_expanded);
    PathBuf::from(env_expanded)
}

pub fn resolve_config_path(path_override: Option<&str>) -> PathBuf {
    match path_override {
        Some(path) if !path.trim().is_empty() => expand_path(path),
        _ => get_cargo_config_path(),
    }
}

fn ensure_backup_dir(config_path: Option<&str>) -> Result<PathBuf, String> {
    let dir = get_backup_dir(config_path);
    if !dir.exists() {
        fs::create_dir_all(&dir).map_err(|e| format!("Failed to create backup dir: {}", e))?;
    }
    Ok(dir)
}

fn sanitize_backup_label(label: &str) -> String {
    let mut out = String::new();
    let invalid_chars = ['<', '>', ':', '"', '/', '\\', '|', '?', '*'];
    for ch in label.chars() {
        if ch.is_control() {
            continue;
        }
        if invalid_chars.contains(&ch) {
            out.push('-');
            continue;
        }
        if ch.is_whitespace() {
            out.push('-');
            continue;
        }
        out.push(ch);
    }
    let cleaned = out.trim_matches('-').to_string();
    if cleaned.is_empty() {
        "custom".to_string()
    } else {
        cleaned
    }
}

fn build_backup_name(label: Option<&str>) -> String {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let prefix = match label {
        Some(value) => format!("manual-{}", sanitize_backup_label(value)),
        None => "auto".to_string(),
    };
    format!("{}-{}.toml", prefix, stamp)
}

fn modified_secs(metadata: &fs::Metadata) -> u64 {
    metadata
        .modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn atomic_write(path: &Path, content: &str) -> Result<(), String> {
    let dir = path.parent().unwrap_or_else(|| Path::new("."));
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let tmp_path = dir.join(format!(".config.tmp-{}", stamp));
    let backup_path = dir.join(format!(".config.bak-{}", stamp));
    fs::write(&tmp_path, content).map_err(|e| format!("Failed to write temp file: {}", e))?;
    match fs::rename(&tmp_path, path) {
        Ok(_) => Ok(()),
        Err(err) => {
            if path.exists() {
                fs::rename(path, &backup_path)
                    .map_err(|e| {
                        let _ = fs::remove_file(&tmp_path);
                        format!("Failed to move original config aside: {} (original: {})", e, err)
                    })?;
            }
            match fs::rename(&tmp_path, path) {
                Ok(_) => {
                    if backup_path.exists() {
                        let _ = fs::remove_file(&backup_path);
                    }
                    Ok(())
                }
                Err(e) => {
                    if backup_path.exists() {
                        let _ = fs::rename(&backup_path, path);
                    }
                    let _ = fs::remove_file(&tmp_path);
                    Err(format!("Failed to replace config: {} (original: {})", e, err))
                }
            }
        }
    }
}

pub fn load_config(path_override: Option<&str>) -> Result<CargoConfig, String> {
    let path = resolve_config_path(path_override);
    if !path.exists() {
        return Ok(CargoConfig::default());
    }
    if !path.is_file() {
        return Err("配置路径不是文件".to_string());
    }
    let content = fs::read_to_string(&path).map_err(|e| format!("Failed to read config: {}", e))?;
    if content.trim().is_empty() {
        return Ok(CargoConfig::default());
    }
    toml::from_str(&content).map_err(|e| format!("Failed to parse config: {}", e))
}

pub fn save_config(config: &CargoConfig, path_override: Option<&str>) -> Result<(), String> {
    let path = resolve_config_path(path_override);
    let content =
        toml::to_string_pretty(config).map_err(|e| format!("Failed to serialize config: {}", e))?;

    if let Some(parent) = path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).map_err(|e| format!("Failed to create directory: {}", e))?;
        }
    }

    atomic_write(&path, &content)
}

pub fn serialize_config(config: &CargoConfig) -> Result<String, String> {
    toml::to_string_pretty(config).map_err(|e| format!("Failed to serialize config: {}", e))
}

pub fn list_backups(config_path: Option<&str>) -> Result<Vec<BackupEntry>, String> {
    let dir = get_backup_dir(config_path);
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut items = Vec::new();
    let entries = fs::read_dir(&dir).map_err(|e| format!("Failed to read backup dir: {}", e))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read backup entry: {}", e))?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if path.extension().and_then(|ext| ext.to_str()) != Some("toml") {
            continue;
        }
        let metadata = entry
            .metadata()
            .map_err(|e| format!("Failed to read backup metadata: {}", e))?;
        let name = entry.file_name().to_string_lossy().to_string();
        items.push(BackupEntry {
            name,
            path: path.to_string_lossy().to_string(),
            modified: modified_secs(&metadata),
            size: metadata.len(),
        });
    }

    items.sort_by(|a, b| b.modified.cmp(&a.modified));
    Ok(items)
}

pub fn create_backup(
    config_path: Option<&str>,
    label: Option<String>,
) -> Result<BackupEntry, String> {
    let config_path = resolve_config_path(config_path);
    if !config_path.exists() {
        return Err("配置文件不存在，无法备份".to_string());
    }

    let dir = ensure_backup_dir(config_path.to_str())?;
    let name = build_backup_name(label.as_deref());
    let dest_path = dir.join(&name);
    fs::copy(&config_path, &dest_path).map_err(|e| format!("Failed to copy backup: {}", e))?;
    let metadata = fs::metadata(&dest_path).map_err(|e| format!("Failed to read backup: {}", e))?;

    Ok(BackupEntry {
        name,
        path: dest_path.to_string_lossy().to_string(),
        modified: modified_secs(&metadata),
        size: metadata.len(),
    })
}

pub fn restore_backup(config_path: Option<&str>, name: String) -> Result<(), String> {
    let backup_dir = get_backup_dir(config_path);
    let file_name = Path::new(&name)
        .file_name()
        .ok_or_else(|| "Invalid backup name".to_string())?;
    let backup_path = backup_dir.join(file_name);
    if !backup_path.exists() {
        return Err("备份文件不存在".to_string());
    }

    let content =
        fs::read_to_string(&backup_path).map_err(|e| format!("Failed to read backup: {}", e))?;
    toml::from_str::<CargoConfig>(&content).map_err(|e| format!("备份文件解析失败: {}", e))?;

    let config_path = resolve_config_path(config_path);
    if let Some(parent) = config_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).map_err(|e| format!("Failed to create directory: {}", e))?;
        }
    }
    atomic_write(&config_path, &content)
}

pub fn import_config_from_path(path: &str) -> Result<CargoConfig, String> {
    let resolved = expand_path(path);
    let content =
        fs::read_to_string(&resolved).map_err(|e| format!("Failed to read file: {}", e))?;
    toml::from_str(&content).map_err(|e| format!("Failed to parse config: {}", e))
}

pub fn export_config_to_path(config: &CargoConfig, path: &str) -> Result<(), String> {
    let resolved = expand_path(path);
    let content =
        toml::to_string_pretty(config).map_err(|e| format!("Failed to serialize config: {}", e))?;
    if let Some(parent) = resolved.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).map_err(|e| format!("Failed to create directory: {}", e))?;
        }
    }
    atomic_write(&resolved, &content)
}

pub fn get_current_target() -> String {
    #[cfg(all(target_os = "windows", target_arch = "x86_64", target_env = "msvc"))]
    return "x86_64-pc-windows-msvc".to_string();

    #[cfg(all(target_os = "windows", target_arch = "x86_64", target_env = "gnu"))]
    return "x86_64-pc-windows-gnu".to_string();

    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    return "x86_64-unknown-linux-gnu".to_string();

    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    return "x86_64-apple-darwin".to_string();

    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    return "aarch64-apple-darwin".to_string();

    #[cfg(not(any(
        all(target_os = "windows", target_arch = "x86_64"),
        all(target_os = "linux", target_arch = "x86_64"),
        all(target_os = "macos", target_arch = "x86_64"),
        all(target_os = "macos", target_arch = "aarch64")
    )))]
    return "unknown".to_string();
}

fn normalize_env_value(value: Option<String>) -> Option<String> {
    value.and_then(|v| {
        let trimmed = v.trim().to_string();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    })
}

#[cfg(not(target_os = "windows"))]
fn strip_wrapping_quotes(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.len() >= 2 {
        let first = trimmed.as_bytes()[0];
        let last = trimmed.as_bytes()[trimmed.len() - 1];
        if (first == b'"' && last == b'"') || (first == b'\'' && last == b'\'') {
            return trimmed[1..trimmed.len() - 1].to_string();
        }
    }
    trimmed.to_string()
}

#[cfg(not(target_os = "windows"))]
fn parse_env_value_from_lines(lines: &str, key: &str) -> Option<String> {
    let prefix = format!("{}=", key);
    for line in lines.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let without_export = trimmed.strip_prefix("export ").unwrap_or(trimmed);
        if let Some(rest) = without_export.strip_prefix(&prefix) {
            return Some(strip_wrapping_quotes(rest));
        }
    }
    None
}

#[cfg(not(target_os = "windows"))]
fn find_managed_block(content: &str) -> Option<(usize, usize)> {
    let start = content.find(RUSTUP_MANAGED_START)?;
    let end = content.find(RUSTUP_MANAGED_END)?;
    let start_line = content[..start].rfind('\n').map(|i| i + 1).unwrap_or(0);
    let end_line = content[end..]
        .find('\n')
        .map(|i| end + i + 1)
        .unwrap_or_else(|| content.len());
    Some((start_line, end_line))
}

#[cfg(not(target_os = "windows"))]
fn extract_managed_block_body(content: &str) -> Option<String> {
    let start = content.find(RUSTUP_MANAGED_START)?;
    let end = content.find(RUSTUP_MANAGED_END)?;
    let body_start = content[start..]
        .find('\n')
        .map(|i| start + i + 1)
        .unwrap_or(end);
    let body = content[body_start..end].trim();
    if body.is_empty() {
        None
    } else {
        Some(body.to_string())
    }
}

#[cfg(not(target_os = "windows"))]
fn apply_managed_block(content: &str, block: Option<&str>) -> String {
    if let Some((start, end)) = find_managed_block(content) {
        let mut out = String::new();
        out.push_str(&content[..start]);
        if let Some(block) = block {
            if !out.ends_with('\n') && !out.is_empty() {
                out.push('\n');
            }
            out.push_str(block);
            if !out.ends_with('\n') {
                out.push('\n');
            }
        }
        out.push_str(&content[end..]);
        return out;
    }
    if let Some(block) = block {
        let mut out = content.to_string();
        if !out.is_empty() && !out.ends_with('\n') {
            out.push('\n');
        }
        out.push_str(block);
        if !out.ends_with('\n') {
            out.push('\n');
        }
        out
    } else {
        content.to_string()
    }
}

#[cfg(not(target_os = "windows"))]
fn build_managed_block(body: &str) -> String {
    format!(
        "{start}\n{body}\n{end}",
        start = RUSTUP_MANAGED_START,
        body = body.trim_end(),
        end = RUSTUP_MANAGED_END
    )
}

#[cfg(not(target_os = "windows"))]
fn escape_shell_value(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(not(target_os = "windows"))]
fn build_export_lines(dist: Option<&str>, root: Option<&str>) -> String {
    let mut lines = Vec::new();
    if let Some(dist) = dist {
        lines.push(format!(
            "export {}=\"{}\"",
            RUSTUP_DIST_SERVER,
            escape_shell_value(dist)
        ));
    }
    if let Some(root) = root {
        lines.push(format!(
            "export {}=\"{}\"",
            RUSTUP_UPDATE_ROOT,
            escape_shell_value(root)
        ));
    }
    lines.join("\n")
}

#[cfg(not(target_os = "windows"))]
fn read_env_value_from_file(path: &Path, key: &str) -> Result<Option<String>, String> {
    if !path.exists() {
        return Ok(None);
    }
    let content = fs::read_to_string(path).map_err(|e| format!("Failed to read env file: {}", e))?;
    Ok(parse_env_value_from_lines(&content, key))
}

#[cfg(not(target_os = "windows"))]
fn write_env_file(path: &Path, dist: Option<&str>, root: Option<&str>) -> Result<(), String> {
    if dist.is_none() && root.is_none() {
        if path.exists() {
            fs::remove_file(path).map_err(|e| format!("Failed to remove env file: {}", e))?;
        }
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).map_err(|e| format!("Failed to create env dir: {}", e))?;
        }
    }
    let mut content = String::new();
    content.push_str("# Managed by Cargo CfgMate\n");
    let exports = build_export_lines(dist, root);
    if !exports.is_empty() {
        content.push_str(&exports);
        content.push('\n');
    }
    fs::write(path, content).map_err(|e| format!("Failed to write env file: {}", e))
}

#[cfg(not(target_os = "windows"))]
fn read_env_value_from_managed_block(path: &Path, key: &str) -> Result<Option<String>, String> {
    if !path.exists() {
        return Ok(None);
    }
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read profile file: {}", e))?;
    let block = extract_managed_block_body(&content);
    Ok(block.and_then(|b| parse_env_value_from_lines(&b, key)))
}

#[cfg(not(target_os = "windows"))]
fn write_managed_block(path: &Path, block: Option<&str>) -> Result<(), String> {
    let content = if path.exists() {
        fs::read_to_string(path).map_err(|e| format!("Failed to read profile file: {}", e))?
    } else {
        String::new()
    };
    let updated = apply_managed_block(&content, block);
    if updated != content {
        fs::write(path, updated).map_err(|e| format!("Failed to write profile file: {}", e))?;
    }
    Ok(())
}

#[cfg(target_os = "windows")]
fn create_hidden_command(program: &str) -> Command {
    const CREATE_NO_WINDOW: u32 = 0x08000000;
    let mut cmd = Command::new(program);
    cmd.creation_flags(CREATE_NO_WINDOW);
    cmd
}

#[cfg(not(target_os = "windows"))]
fn escape_shell_path(value: &str) -> String {
    value.replace('"', "\\\"")
}

#[cfg(target_os = "windows")]
fn build_admin_hint() -> String {
    "请右键应用图标并选择“以管理员身份运行”".to_string()
}

#[cfg(not(target_os = "windows"))]
fn build_admin_hint() -> String {
    let exe_path = std::env::current_exe()
        .ok()
        .and_then(|path| path.to_str().map(|value| value.to_string()));
    match exe_path {
        Some(path) => format!("请在终端执行：sudo \"{}\"", escape_shell_path(&path)),
        None => "请在终端执行：sudo <应用路径>".to_string(),
    }
}

#[cfg(target_os = "windows")]
fn is_admin() -> bool {
    create_hidden_command("net")
        .args(["session"])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

#[cfg(not(target_os = "windows"))]
fn is_admin() -> bool {
    Command::new("id")
        .arg("-u")
        .output()
        .map(|output| String::from_utf8_lossy(&output.stdout).trim() == "0")
        .unwrap_or(false)
}

pub fn get_admin_status() -> AdminStatus {
    let is_admin = is_admin();
    let hint = if is_admin { String::new() } else { build_admin_hint() };
    AdminStatus { is_admin, hint }
}

#[cfg(target_os = "windows")]
fn escape_powershell_single_quotes(value: &str) -> String {
    value.replace('\'', "''")
}

#[cfg(target_os = "windows")]
fn looks_like_utf16_le(bytes: &[u8]) -> bool {
    if bytes.len() < 4 {
        return false;
    }
    let mut zero_count = 0;
    let mut sample_count = 0;
    for (idx, byte) in bytes.iter().take(64).enumerate() {
        if idx % 2 == 1 {
            sample_count += 1;
            if *byte == 0 {
                zero_count += 1;
            }
        }
    }
    sample_count > 0 && zero_count * 2 >= sample_count
}

#[cfg(target_os = "windows")]
fn decode_powershell_output(bytes: &[u8]) -> String {
    if looks_like_utf16_le(bytes) {
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

#[cfg(target_os = "windows")]
fn run_powershell(script: &str) -> Result<String, String> {
    let output = create_hidden_command("powershell")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            script,
        ])
        .output()
        .map_err(|e| format!("Failed to run PowerShell: {}", e))?;
    if !output.status.success() {
        return Err(decode_powershell_output(&output.stderr));
    }
    Ok(decode_powershell_output(&output.stdout))
}

#[cfg(target_os = "windows")]
#[derive(Deserialize)]
struct WindowsRustupEnvSnapshot {
    user_dist: Option<String>,
    user_root: Option<String>,
    system_dist: Option<String>,
    system_root: Option<String>,
}

#[cfg(target_os = "windows")]
#[derive(Deserialize)]
struct WindowsRustupEnvWriteResult {
    user_ok: bool,
    user_error: Option<String>,
    system_ok: bool,
    system_error: Option<String>,
}

#[cfg(target_os = "windows")]
fn read_windows_env_snapshot() -> Result<WindowsRustupEnvSnapshot, String> {
    let script = format!(
        "$result = [ordered]@{{ user_dist = [Environment]::GetEnvironmentVariable('{dist}','User'); user_root = [Environment]::GetEnvironmentVariable('{root}','User'); system_dist = [Environment]::GetEnvironmentVariable('{dist}','Machine'); system_root = [Environment]::GetEnvironmentVariable('{root}','Machine') }}; $result | ConvertTo-Json -Compress",
        dist = RUSTUP_DIST_SERVER,
        root = RUSTUP_UPDATE_ROOT
    );
    let output = run_powershell(&script)?;
    serde_json::from_str(&output).map_err(|e| format!("Failed to parse PowerShell output: {}", e))
}

#[cfg(target_os = "windows")]
fn write_windows_env_all(dist: Option<&str>, root: Option<&str>) -> Result<WindowsRustupEnvWriteResult, String> {
    let dist_expr = dist
        .map(|v| format!("'{}'", escape_powershell_single_quotes(v)))
        .unwrap_or_else(|| "$null".to_string());
    let root_expr = root
        .map(|v| format!("'{}'", escape_powershell_single_quotes(v)))
        .unwrap_or_else(|| "$null".to_string());
    let script = format!(
        "$dist = {dist}; $root = {root}; $result = [ordered]@{{ user_ok = $true; user_error = $null; system_ok = $true; system_error = $null }}; try {{ [Environment]::SetEnvironmentVariable('{dist_key}', $dist, 'User'); [Environment]::SetEnvironmentVariable('{root_key}', $root, 'User') }} catch {{ $result.user_ok = $false; $result.user_error = $_.Exception.Message }}; try {{ [Environment]::SetEnvironmentVariable('{dist_key}', $dist, 'Machine'); [Environment]::SetEnvironmentVariable('{root_key}', $root, 'Machine') }} catch {{ $result.system_ok = $false; $result.system_error = $_.Exception.Message }}; $result | ConvertTo-Json -Compress",
        dist = dist_expr,
        root = root_expr,
        dist_key = RUSTUP_DIST_SERVER,
        root_key = RUSTUP_UPDATE_ROOT
    );
    let output = run_powershell(&script)?;
    serde_json::from_str(&output).map_err(|e| format!("Failed to parse PowerShell output: {}", e))
}

#[cfg(target_os = "windows")]
fn write_windows_env_user(dist: Option<&str>, root: Option<&str>) -> Result<(), String> {
    let dist_expr = dist
        .map(|v| format!("'{}'", escape_powershell_single_quotes(v)))
        .unwrap_or_else(|| "$null".to_string());
    let root_expr = root
        .map(|v| format!("'{}'", escape_powershell_single_quotes(v)))
        .unwrap_or_else(|| "$null".to_string());
    let script = format!(
        "$dist = {dist}; $root = {root}; [Environment]::SetEnvironmentVariable('{dist_key}', $dist, 'User'); [Environment]::SetEnvironmentVariable('{root_key}', $root, 'User')",
        dist = dist_expr,
        root = root_expr,
        dist_key = RUSTUP_DIST_SERVER,
        root_key = RUSTUP_UPDATE_ROOT
    );
    let _ = run_powershell(&script)?;
    Ok(())
}

#[cfg(target_os = "windows")]
fn broadcast_windows_env_change() {
    let script = r#"
$signature = '[DllImport("user32.dll")] public static extern IntPtr SendMessageTimeout(IntPtr hWnd, uint Msg, IntPtr wParam, string lParam, uint fuFlags, uint uTimeout, out IntPtr lpdwResult);'
Add-Type -MemberDefinition $signature -Name 'Win32SendMessageTimeout' -Namespace Win32Functions -PassThru | Out-Null
$HWND_BROADCAST = [intptr]0xffff
$WM_SETTINGCHANGE = 0x1A
$result = [intptr]0
[Win32Functions.Win32SendMessageTimeout]::SendMessageTimeout($HWND_BROADCAST, $WM_SETTINGCHANGE, [intptr]0, "Environment", 0x2, 5000, [ref]$result) | Out-Null
"#;
    let _ = run_powershell(script);
}

#[cfg(target_os = "macos")]
fn get_user_profile_path() -> PathBuf {
    let home = get_home_dir();
    let zprofile = Path::new(&home).join(".zprofile");
    if zprofile.exists() {
        zprofile
    } else {
        Path::new(&home).join(".profile")
    }
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn get_user_profile_path() -> PathBuf {
    Path::new(&get_home_dir()).join(".profile")
}

#[cfg(not(target_os = "windows"))]
fn get_linux_env_file() -> PathBuf {
    Path::new(&get_home_dir())
        .join(".config")
        .join("quickchange")
        .join("rustup.env")
}

#[cfg(target_os = "macos")]
fn read_macos_user_env(key: &str) -> Result<Option<String>, String> {
    let profile = get_user_profile_path();
    read_env_value_from_managed_block(&profile, key)
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn read_unix_user_env(key: &str) -> Result<Option<String>, String> {
    let env_file = get_linux_env_file();
    read_env_value_from_file(&env_file, key)
}

#[cfg(target_os = "macos")]
fn read_macos_system_env(key: &str) -> Result<Option<String>, String> {
    read_env_value_from_managed_block(Path::new("/etc/zprofile"), key)
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn read_unix_system_env(key: &str) -> Result<Option<String>, String> {
    read_env_value_from_file(Path::new("/etc/profile.d/quickchange-rustup.sh"), key)
}

#[cfg(target_os = "macos")]
fn write_macos_user_env(dist: Option<&str>, root: Option<&str>) -> Result<(), String> {
    let profile = get_user_profile_path();
    if dist.is_none() && root.is_none() {
        return write_managed_block(&profile, None);
    }
    let exports = build_export_lines(dist, root);
    let block = build_managed_block(&exports);
    write_managed_block(&profile, Some(&block))
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn write_unix_user_env(dist: Option<&str>, root: Option<&str>) -> Result<(), String> {
    let env_file = get_linux_env_file();
    write_env_file(&env_file, dist, root)?;
    let profile = get_user_profile_path();
    if dist.is_none() && root.is_none() {
        return write_managed_block(&profile, None);
    }
    let block = build_managed_block(
        "if [ -f \"$HOME/.config/quickchange/rustup.env\" ]; then\n  . \"$HOME/.config/quickchange/rustup.env\"\nfi"
    );
    write_managed_block(&profile, Some(&block))
}

#[cfg(target_os = "macos")]
fn write_macos_system_env(dist: Option<&str>, root: Option<&str>) -> Result<(), String> {
    let profile = Path::new("/etc/zprofile");
    if dist.is_none() && root.is_none() {
        return write_managed_block(profile, None);
    }
    let exports = build_export_lines(dist, root);
    let block = build_managed_block(&exports);
    write_managed_block(profile, Some(&block))
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn write_unix_system_env(dist: Option<&str>, root: Option<&str>) -> Result<(), String> {
    write_env_file(Path::new("/etc/profile.d/quickchange-rustup.sh"), dist, root)
}

pub fn get_rustup_env_status() -> RustupEnvStatus {
    let mut status = RustupEnvStatus::default();
    let session_dist = normalize_env_value(std::env::var(RUSTUP_DIST_SERVER).ok());
    let session_root = normalize_env_value(std::env::var(RUSTUP_UPDATE_ROOT).ok());

    #[cfg(target_os = "windows")]
    {
        match read_windows_env_snapshot() {
            Ok(snapshot) => {
                status.dist.user.value = normalize_env_value(snapshot.user_dist);
                status.root.user.value = normalize_env_value(snapshot.user_root);
                status.dist.system.value = normalize_env_value(snapshot.system_dist);
                status.root.system.value = normalize_env_value(snapshot.system_root);

                if status.dist.user.value.is_some() {
                    status.dist.user.source = Some("registry".to_string());
                }
                if status.root.user.value.is_some() {
                    status.root.user.source = Some("registry".to_string());
                }
                if status.dist.system.value.is_some() {
                    status.dist.system.source = Some("registry".to_string());
                }
                if status.root.system.value.is_some() {
                    status.root.system.source = Some("registry".to_string());
                }
            }
            Err(err) => {
                status.dist.user.error = Some(err.clone());
                status.root.user.error = Some(err.clone());
                status.dist.system.error = Some(err.clone());
                status.root.system.error = Some(err);
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        match read_macos_user_env(RUSTUP_DIST_SERVER) {
            Ok(value) => {
                status.dist.user.value = value;
                if status.dist.user.value.is_some() {
                    status.dist.user.source = Some("managed".to_string());
                }
            }
            Err(err) => status.dist.user.error = Some(err),
        }
        match read_macos_system_env(RUSTUP_DIST_SERVER) {
            Ok(value) => {
                status.dist.system.value = value;
                if status.dist.system.value.is_some() {
                    status.dist.system.source = Some("managed".to_string());
                }
            }
            Err(err) => status.dist.system.error = Some(err),
        }
        match read_macos_user_env(RUSTUP_UPDATE_ROOT) {
            Ok(value) => {
                status.root.user.value = value;
                if status.root.user.value.is_some() {
                    status.root.user.source = Some("managed".to_string());
                }
            }
            Err(err) => status.root.user.error = Some(err),
        }
        match read_macos_system_env(RUSTUP_UPDATE_ROOT) {
            Ok(value) => {
                status.root.system.value = value;
                if status.root.system.value.is_some() {
                    status.root.system.source = Some("managed".to_string());
                }
            }
            Err(err) => status.root.system.error = Some(err),
        }
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        match read_unix_user_env(RUSTUP_DIST_SERVER) {
            Ok(value) => {
                status.dist.user.value = value;
                if status.dist.user.value.is_some() {
                    status.dist.user.source = Some("managed".to_string());
                }
            }
            Err(err) => status.dist.user.error = Some(err),
        }
        match read_unix_system_env(RUSTUP_DIST_SERVER) {
            Ok(value) => {
                status.dist.system.value = value;
                if status.dist.system.value.is_some() {
                    status.dist.system.source = Some("managed".to_string());
                }
            }
            Err(err) => status.dist.system.error = Some(err),
        }
        match read_unix_user_env(RUSTUP_UPDATE_ROOT) {
            Ok(value) => {
                status.root.user.value = value;
                if status.root.user.value.is_some() {
                    status.root.user.source = Some("managed".to_string());
                }
            }
            Err(err) => status.root.user.error = Some(err),
        }
        match read_unix_system_env(RUSTUP_UPDATE_ROOT) {
            Ok(value) => {
                status.root.system.value = value;
                if status.root.system.value.is_some() {
                    status.root.system.source = Some("managed".to_string());
                }
            }
            Err(err) => status.root.system.error = Some(err),
        }
    }

    if status.dist.user.value.is_none() {
        status.dist.user.value = session_dist;
        if status.dist.user.value.is_some() {
            status.dist.user.source = Some("session".to_string());
        }
    }
    if status.root.user.value.is_none() {
        status.root.user.value = session_root;
        if status.root.user.value.is_some() {
            status.root.user.source = Some("session".to_string());
        }
    }

    status
}

fn set_process_rustup_env(dist: Option<&str>, root: Option<&str>) {
    if let Some(value) = dist {
        std::env::set_var(RUSTUP_DIST_SERVER, value);
    } else {
        std::env::remove_var(RUSTUP_DIST_SERVER);
    }
    if let Some(value) = root {
        std::env::set_var(RUSTUP_UPDATE_ROOT, value);
    } else {
        std::env::remove_var(RUSTUP_UPDATE_ROOT);
    }
}

pub fn set_rustup_env(
    dist: Option<String>,
    root: Option<String>,
    scope: Option<String>,
) -> RustupEnvWriteResult {
    let dist = normalize_env_value(dist);
    let root = normalize_env_value(root);
    let user: RustupEnvWriteOutcome;
    let system: RustupEnvWriteOutcome;
    let is_admin = is_admin();
    let should_write_system = match scope.as_deref() {
        Some("user") => false,
        Some("user_and_system") => is_admin,
        _ => is_admin,
    };

    #[cfg(target_os = "windows")]
    {
        if should_write_system {
            match write_windows_env_all(dist.as_deref(), root.as_deref()) {
                Ok(result) => {
                    user = RustupEnvWriteOutcome {
                        ok: result.user_ok,
                        error: result.user_error,
                        skipped: false,
                    };
                    system = RustupEnvWriteOutcome {
                        ok: result.system_ok,
                        error: result.system_error,
                        skipped: false,
                    };
                }
                Err(err) => {
                    user = RustupEnvWriteOutcome { ok: false, error: Some(err.clone()), skipped: false };
                    system = RustupEnvWriteOutcome { ok: false, error: Some(err), skipped: false };
                }
            }
        } else {
            user = match write_windows_env_user(dist.as_deref(), root.as_deref()) {
                Ok(_) => RustupEnvWriteOutcome { ok: true, error: None, skipped: false },
                Err(err) => RustupEnvWriteOutcome { ok: false, error: Some(err), skipped: false },
            };
            system = RustupEnvWriteOutcome {
                ok: true,
                error: None,
                skipped: true,
            };
        }
        let system_applied = system.ok && !system.skipped;
        if user.ok || system_applied {
            broadcast_windows_env_change();
        }
    }

    #[cfg(target_os = "macos")]
    {
        user = match write_macos_user_env(dist.as_deref(), root.as_deref()) {
            Ok(_) => RustupEnvWriteOutcome { ok: true, error: None, skipped: false },
            Err(err) => RustupEnvWriteOutcome { ok: false, error: Some(err), skipped: false },
        };
        system = if should_write_system {
            match write_macos_system_env(dist.as_deref(), root.as_deref()) {
                Ok(_) => RustupEnvWriteOutcome { ok: true, error: None, skipped: false },
                Err(err) => RustupEnvWriteOutcome { ok: false, error: Some(err), skipped: false },
            }
        } else {
            RustupEnvWriteOutcome {
                ok: true,
                error: None,
                skipped: true,
            }
        };
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        user = match write_unix_user_env(dist.as_deref(), root.as_deref()) {
            Ok(_) => RustupEnvWriteOutcome { ok: true, error: None, skipped: false },
            Err(err) => RustupEnvWriteOutcome { ok: false, error: Some(err), skipped: false },
        };
        system = if should_write_system {
            match write_unix_system_env(dist.as_deref(), root.as_deref()) {
                Ok(_) => RustupEnvWriteOutcome { ok: true, error: None, skipped: false },
                Err(err) => RustupEnvWriteOutcome { ok: false, error: Some(err), skipped: false },
            }
        } else {
            RustupEnvWriteOutcome {
                ok: true,
                error: None,
                skipped: true,
            }
        };
    }

    let system_applied = system.ok && !system.skipped;
    if user.ok || system_applied {
        set_process_rustup_env(dist.as_deref(), root.as_deref());
    }

    RustupEnvWriteResult { user, system }
}

pub fn get_rustup_env_effective() -> (Option<String>, Option<String>) {
    let status = get_rustup_env_status();
    let dist = status.dist.user.value.or(status.dist.system.value);
    let root = status.root.user.value.or(status.root.system.value);
    (dist, root)
}

pub fn clear_backups(config_path: Option<&str>) -> Result<usize, String> {
    let backup_dir = get_backup_dir(config_path);
    if !backup_dir.exists() {
        return Ok(0);
    }

    let mut count = 0;
    let entries =
        fs::read_dir(&backup_dir).map_err(|e| format!("Failed to read backup dir: {}", e))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|ext| ext.to_str()) == Some("toml") {
            fs::remove_file(&path)
                .map_err(|e| format!("Failed to remove {}: {}", path.display(), e))?;
            count += 1;
        }
    }

    Ok(count)
}

pub fn delete_backup(config_path: Option<&str>, name: String) -> Result<(), String> {
    let backup_dir = get_backup_dir(config_path);
    let file_name = Path::new(&name)
        .file_name()
        .ok_or_else(|| "Invalid backup name".to_string())?;
    let backup_path = backup_dir.join(file_name);

    if !backup_path.exists() {
        return Err("备份文件不存在".to_string());
    }

    fs::remove_file(&backup_path).map_err(|e| format!("Failed to delete backup: {}", e))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 验证首次加载不存在的配置时只返回默认配置，不创建文件。
    ///
    /// ```ignore
    /// let config = load_config(Some("missing-config.toml")).unwrap();
    /// assert!(config.source.is_none());
    /// ```
    #[test]
    fn load_missing_config_does_not_create_file() {
        let path = std::env::temp_dir().join(format!(
            "cargo-cfgmate-missing-{}.toml",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));

        if path.exists() {
            fs::remove_file(&path).unwrap();
        }

        let config = load_config(path.to_str()).unwrap();

        assert!(config.source.is_none());
        assert!(!path.exists());
    }
}

pub fn rename_backup(
    config_path: Option<&str>,
    old_name: String,
    new_name: String,
) -> Result<(), String> {
    let backup_dir = get_backup_dir(config_path);

    let old_file_name = Path::new(&old_name)
        .file_name()
        .ok_or_else(|| "Invalid old backup name".to_string())?;
    let old_path = backup_dir.join(old_file_name);

    if !old_path.exists() {
        return Err("备份文件不存在".to_string());
    }

    // 确保新文件名以 .toml 结尾
    let new_name_with_ext = if new_name.ends_with(".toml") {
        new_name
    } else {
        format!("{}.toml", new_name)
    };

    let new_path = backup_dir.join(&new_name_with_ext);

    if new_path.exists() {
        return Err("目标文件名已存在".to_string());
    }

    fs::rename(&old_path, &new_path).map_err(|e| format!("Failed to rename backup: {}", e))?;
    Ok(())
}
