use crate::config::AppConfig;
use anyhow::{Context, Result};
use std::collections::{HashMap, HashSet};
use std::process::Command;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Distro {
    Ubuntu,
    Fedora,
    Macos,
    Windows,
    Unknown,
}

impl Distro {
    pub fn as_str(&self) -> &'static str {
        match self {
            Distro::Ubuntu => "ubuntu",
            Distro::Fedora => "fedora",
            Distro::Macos => "macos",
            Distro::Windows => "windows",
            Distro::Unknown => "unknown",
        }
    }
}

pub fn detect_distro() -> Distro {
    if cfg!(target_os = "windows") {
        return Distro::Windows;
    }
    if cfg!(target_os = "macos") {
        return Distro::Macos;
    }

    // Linux: parse /etc/os-release
    if let Ok(content) = std::fs::read_to_string("/etc/os-release") {
        let fields = parse_os_release(&content);

        let id = fields.get("ID").map(String::as_str).unwrap_or("");
        let id_like = fields.get("ID_LIKE").map(String::as_str).unwrap_or("");

        let matches = |s: &str| {
            s.split_whitespace()
                .any(|w| matches!(w, "ubuntu" | "debian"))
        };
        if matches(id) || matches(id_like) {
            return Distro::Ubuntu;
        }

        let matches_fedora = |s: &str| {
            s.split_whitespace()
                .any(|w| matches!(w, "fedora" | "rhel"))
        };
        if matches_fedora(id) || matches_fedora(id_like) {
            return Distro::Fedora;
        }
    }

    Distro::Unknown
}

fn parse_os_release(content: &str) -> HashMap<String, String> {
    content
        .lines()
        .filter_map(|line| {
            let (k, v) = line.split_once('=')?;
            let v = v.trim_matches('"').to_string();
            Some((k.to_string(), v))
        })
        .collect()
}

pub fn select_commands<'a>(
    map: &'a HashMap<String, Vec<String>>,
    distro: &Distro,
) -> Option<&'a Vec<String>> {
    map.get(distro.as_str()).or_else(|| map.get("default"))
}

pub fn command_requires_sudo(commands: &[String]) -> bool {
    commands.iter().any(|c| {
        c.contains("sudo ")
            || c.starts_with("sudo")
            || c.contains(" sudo ")
    })
}

#[derive(Debug, Clone)]
pub struct CommandResult {
    pub command: String,
    pub exit_code: i32,
    pub output: String,
}

pub fn run_command(
    command: &str,
    sudo_password: Option<&str>,
    cancel: &std::sync::Arc<std::sync::atomic::AtomicBool>,
    output_tx: Option<&tokio::sync::mpsc::UnboundedSender<String>>,
) -> Result<CommandResult> {
    #[cfg(windows)]
    return run_command_windows(command, cancel, output_tx);
    #[cfg(unix)]
    return run_command_unix(command, sudo_password, cancel, output_tx);
}

#[cfg(unix)]
fn run_command_unix(
    command: &str,
    sudo_password: Option<&str>,
    cancel: &std::sync::Arc<std::sync::atomic::AtomicBool>,
    output_tx: Option<&tokio::sync::mpsc::UnboundedSender<String>>,
) -> Result<CommandResult> {
    use std::io::{Read, Write};
    use std::os::unix::io::AsRawFd;

    let prepared = prepare_command_unix(command, sudo_password);

    let mut output_buf = String::new();

    // Use bash to run the command
    let mut child = std::process::Command::new("bash")
        .arg("-c")
        .arg(&prepared)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .context("Failed to spawn command")?;

    if let (Some(mut stdin), Some(password)) = (child.stdin.take(), sudo_password) {
        let _ = stdin.write_all(format!("{}\n", password).as_bytes());
    }

    let output = child.wait_with_output()?;
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let clean = strip_ansi(&combined);
    if let Some(tx) = output_tx {
        for line in clean.lines() {
            let _ = tx.send(line.to_string());
        }
    }

    Ok(CommandResult {
        command: command.to_string(),
        exit_code: output.status.code().unwrap_or(-1),
        output: clean,
    })
}

#[cfg(windows)]
fn run_command_windows(
    command: &str,
    cancel: &std::sync::Arc<std::sync::atomic::AtomicBool>,
    output_tx: Option<&tokio::sync::mpsc::UnboundedSender<String>>,
) -> Result<CommandResult> {
    let output = Command::new("cmd")
        .args(["/C", command])
        .output()
        .context("Failed to spawn command")?;

    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let clean = strip_ansi(&combined);
    if let Some(tx) = output_tx {
        for line in clean.lines() {
            let _ = tx.send(line.to_string());
        }
    }

    Ok(CommandResult {
        command: command.to_string(),
        exit_code: output.status.code().unwrap_or(-1),
        output: clean,
    })
}

fn prepare_command_unix(command: &str, sudo_password: Option<&str>) -> String {
    if sudo_password.is_some() && command_requires_sudo(&[command.to_string()]) {
        command
            .replace("sudo ", "sudo -S -p '' ")
            .replace("sudo\t", "sudo -S -p '' ")
    } else {
        command.to_string()
    }
}

pub fn strip_ansi(s: &str) -> String {
    // Simple ANSI escape stripping
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            // Skip escape sequence
            if chars.peek() == Some(&'[') {
                chars.next();
                // Skip until letter
                for ch in chars.by_ref() {
                    if ch.is_ascii_alphabetic() {
                        break;
                    }
                }
            }
        } else {
            result.push(c);
        }
    }
    result
}

#[derive(Debug, Clone)]
pub struct AppStatus {
    pub app: AppConfig,
    pub supported: bool,
    pub installed: bool,
    pub message: String,
}

pub fn resolve_status(app: &AppConfig, distro: &Distro) -> AppStatus {
    let has_any_action = !app.install.is_empty()
        || !app.update.is_empty()
        || !app.uninstall.is_empty()
        || !app.check.is_empty();

    let supported = select_commands(&app.install, distro).is_some()
        || select_commands(&app.check, distro).is_some()
        || !has_any_action;

    if !supported {
        return AppStatus {
            app: app.clone(),
            supported: false,
            installed: false,
            message: String::new(),
        };
    }

    let (installed, message) = check_installed(app, distro);

    AppStatus {
        app: app.clone(),
        supported: true,
        installed,
        message,
    }
}

fn check_installed(app: &AppConfig, distro: &Distro) -> (bool, String) {
    let Some(commands) = select_commands(&app.check, distro) else {
        // No check commands → assume not installed if install exists
        if select_commands(&app.install, distro).is_some() {
            return (false, String::new());
        }
        return (true, String::new());
    };

    let mut output_lines = Vec::new();
    for cmd in commands {
        let result = run_command(
            cmd,
            None,
            &std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
            None,
        );
        match result {
            Ok(r) => {
                output_lines.push(r.output.trim().to_string());
                if r.exit_code != 0 {
                    return (false, output_lines.join("\n"));
                }
            }
            Err(e) => {
                return (false, e.to_string());
            }
        }
    }

    (true, output_lines.join("\n"))
}

pub fn action_requires_reboot(app: &AppConfig, action: &str) -> bool {
    app.reboot_on.get(action).copied().unwrap_or(false)
}

/// Topological sort of apps by depends_on. Returns indices into the apps slice.
pub fn topological_sort(apps: &[AppStatus]) -> Vec<usize> {
    let id_to_idx: HashMap<&str, usize> = apps
        .iter()
        .enumerate()
        .map(|(i, s)| (s.app.id.as_str(), i))
        .collect();

    let mut visited = HashSet::new();
    let mut result = Vec::new();

    fn visit(
        idx: usize,
        apps: &[AppStatus],
        id_to_idx: &HashMap<&str, usize>,
        visited: &mut HashSet<usize>,
        result: &mut Vec<usize>,
        stack: &mut HashSet<usize>,
    ) {
        if visited.contains(&idx) {
            return;
        }
        if stack.contains(&idx) {
            // Cycle detected
            return;
        }
        stack.insert(idx);
        for dep_id in &apps[idx].app.depends_on {
            if let Some(&dep_idx) = id_to_idx.get(dep_id.as_str()) {
                visit(dep_idx, apps, id_to_idx, visited, result, stack);
            }
        }
        stack.remove(&idx);
        visited.insert(idx);
        result.push(idx);
    }

    let mut stack = HashSet::new();
    for i in 0..apps.len() {
        visit(i, apps, &id_to_idx, &mut visited, &mut result, &mut stack);
    }

    result
}

#[derive(Debug, Clone)]
pub struct CommandGroup {
    pub group_id: usize,
    pub label: String,
    pub commands: Vec<String>,
}

pub fn build_terminal_script(groups: &[CommandGroup], result_path: &str, done_path: &str) -> String {
    let mut script = format!(
        "#!/usr/bin/env bash\nset -u\nRESULT={}\nDONE={}\n: > \"$RESULT\"\n\n",
        result_path, done_path
    );

    for group in groups {
        script.push_str(&format!(
            "printf '\\n\\033[1;36m==> {}\\033[0m\\n'\n",
            group.label
        ));

        let chain = group
            .commands
            .iter()
            .enumerate()
            .map(|(i, cmd)| {
                if i == 0 {
                    cmd.clone()
                } else {
                    format!(" && \\\n   {}", cmd)
                }
            })
            .collect::<String>();

        script.push_str(&format!(
            "if {}; then rc=0; else rc=$?; fi\n",
            chain
        ));
        script.push_str(&format!(
            "printf '%s\\t%s\\n' {} \"$rc\" >> \"$RESULT\"\n\n",
            group.group_id
        ));
    }

    script.push_str(": > \"$DONE\"\n");
    script.push_str("printf '\\n\\033[1mConcluido. Pressione ENTER para fechar...\\033[0m'\n");
    script.push_str("read -r _\n");

    script
}

pub fn validate_sudo_password(password: &str) -> bool {
    #[cfg(windows)]
    return true;

    #[cfg(unix)]
    {
        use std::io::Write;
        let mut child = match std::process::Command::new("sudo")
            .args(["-S", "-k", "-p", "", "-v"])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
        {
            Ok(c) => c,
            Err(_) => return false,
        };

        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(format!("{}\n", password).as_bytes());
        }

        child.wait().map(|s| s.success()).unwrap_or(false)
    }
}
