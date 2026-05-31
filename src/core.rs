use crate::config::AppConfig;
use anyhow::{Context, Result};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

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

    if let Ok(content) = std::fs::read_to_string("/etc/os-release") {
        let fields = parse_os_release(&content);
        let id = fields.get("ID").map(String::as_str).unwrap_or("");
        let id_like = fields.get("ID_LIKE").map(String::as_str).unwrap_or("");

        let is_ubuntu = |s: &str| s.split_whitespace().any(|w| matches!(w, "ubuntu" | "debian"));
        if is_ubuntu(id) || is_ubuntu(id_like) {
            return Distro::Ubuntu;
        }

        let is_fedora = |s: &str| s.split_whitespace().any(|w| matches!(w, "fedora" | "rhel"));
        if is_fedora(id) || is_fedora(id_like) {
            return Distro::Fedora;
        }
    }

    Distro::Unknown
}

fn parse_os_release(content: &str) -> HashMap<String, String> {
    content
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                return None;
            }
            let (k, v) = line.split_once('=')?;
            Some((k.to_string(), v.trim_matches('"').to_string()))
        })
        .collect()
}

// Mirrors Python: distro → "default" → "all" → "*"
pub fn select_commands<'a>(
    map: &'a HashMap<String, Vec<String>>,
    distro: &Distro,
) -> Option<&'a Vec<String>> {
    map.get(distro.as_str())
        .or_else(|| map.get("default"))
        .or_else(|| map.get("all"))
        .or_else(|| map.get("*"))
}

#[derive(Debug, Clone)]
pub struct CommandResult {
    pub command: String,
    pub exit_code: i32,
    pub output: String,
}

pub fn run_command(
    command: &str,
    _cancel: &Arc<AtomicBool>,
    output_tx: Option<&tokio::sync::mpsc::UnboundedSender<String>>,
) -> Result<CommandResult> {
    #[cfg(windows)]
    return run_command_windows(command, output_tx);
    #[cfg(unix)]
    return run_command_unix(command,  output_tx);
}

#[cfg(unix)]
fn run_command_unix(
    command: &str,
    output_tx: Option<&tokio::sync::mpsc::UnboundedSender<String>>,
) -> Result<CommandResult> {
    let child = std::process::Command::new("bash")
        .arg("-c")
        .arg(&command)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .context("Failed to spawn command")?;


    let output = child.wait_with_output()?;
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let clean = strip_ansi(&combined);
    if let Some(tx) = output_tx {
        for line in clean.lines() {
            if !line.trim().is_empty() {
                let _ = tx.send(line.to_string());
            }
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
    output_tx: Option<&tokio::sync::mpsc::UnboundedSender<String>>,
) -> Result<CommandResult> {
    let output = std::process::Command::new("cmd")
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
            if !line.trim().is_empty() {
                let _ = tx.send(line.to_string());
            }
        }
    }

    Ok(CommandResult {
        command: command.to_string(),
        exit_code: output.status.code().unwrap_or(-1),
        output: clean,
    })
}

pub fn strip_ansi(s: &str) -> String {
    let stripped = strip_ansi_escapes::strip(s.as_bytes());
    String::from_utf8_lossy(&stripped).into_owned()
}

#[derive(Debug, Clone)]
pub struct AppStatus {
    pub app: AppConfig,
    pub supported: bool,
    pub installed: bool,
    pub message: String,
}

// An app is "supported" when it has install or update commands for the current distro.
pub fn resolve_status(app: &AppConfig, distro: &Distro) -> AppStatus {
    let supported = select_commands(&app.install, distro).is_some()
        || select_commands(&app.update, distro).is_some();

    if !supported {
        return AppStatus {
            app: app.clone(),
            supported: false,
            installed: false,
            message: format!("Sem comandos para {}.", distro.as_str()),
        };
    }

    let (installed, message) = check_installed(app, distro);
    AppStatus { app: app.clone(), supported: true, installed, message }
}

fn check_installed(app: &AppConfig, distro: &Distro) -> (bool, String) {
    let Some(commands) = select_commands(&app.check, distro) else {
        // No check command: assume not-installed when install exists, else installed.
        if select_commands(&app.install, distro).is_some() {
            return (false, String::new());
        }
        return (true, String::new());
    };

    let cancel = Arc::new(AtomicBool::new(false));
    let mut output_lines = Vec::new();

    for cmd in commands {
        match run_command(cmd,  &cancel, None) {
            Ok(r) => {
                let trimmed = r.output.trim().to_string();
                if !trimmed.is_empty() {
                    output_lines.push(format!("$ {}\n{}", cmd, trimmed));
                }
                if r.exit_code != 0 {
                    return (false, output_lines.join("\n"));
                }
            }
            Err(e) => return (false, e.to_string()),
        }
    }

    (true, output_lines.join("\n"))
}

pub fn action_requires_reboot(app: &AppConfig, action: &str) -> bool {
    app.reboot_on.get(action).copied().unwrap_or(false)
}

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
        if visited.contains(&idx) || stack.contains(&idx) {
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
    let mut lines = vec![
        "#!/usr/bin/env bash".to_string(),
        "set -u".to_string(),
        format!("RESULT={}", shell_quote(result_path)),
        format!("DONE={}", shell_quote(done_path)),
        r#": > "$RESULT""#.to_string(),
        String::new(),
    ];

    for group in groups {
        lines.push(format!(
            "printf '\\n\\033[1;36m==> %s\\033[0m\\n' {}",
            shell_quote(&group.label)
        ));

        let chained = group
            .commands
            .iter()
            .enumerate()
            .map(|(i, cmd)| if i == 0 { cmd.clone() } else { format!(" && \\\n  {}", cmd) })
            .collect::<String>();

        lines.push(format!("if {}; then rc=0; else rc=$?; fi", chained));
        lines.push(format!(
            "printf '%s\\t%s\\n' {} \"$rc\" >> \"$RESULT\"",
            group.group_id
        ));
        lines.push(r#"if [ "$rc" -ne 0 ]; then"#.to_string());
        lines.push(r#"  printf '\033[1;31m[falhou] rc=%s\033[0m\n' "$rc""#.to_string());
        lines.push("else".to_string());
        lines.push(r#"  printf '\033[1;32m[ok]\033[0m\n'"#.to_string());
        lines.push("fi".to_string());
        lines.push(String::new());
    }

    lines.push(r#": > "$DONE""#.to_string());
    lines.push(r#"printf '\n\033[1mConcluido. Pressione ENTER para fechar...\033[0m'"#.to_string());
    lines.push("read -r _".to_string());

    lines.join("\n")
}

fn shell_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}

// --- External terminal execution ---

#[derive(Debug, Default)]
pub struct TerminalRunResult {
    /// Exit code per group_id. Missing entry = group not reached.
    pub exit_codes: HashMap<usize, i32>,
    pub launched: bool,
    pub terminal: String,
    pub error: String,
}

const KNOWN_TERMINALS: &[&str] = &[
    "gnome-terminal",
    "ptyxis",
    "konsole",
    "xfce4-terminal",
    "tilix",
    "kitty",
    "alacritty",
    "wezterm",
    "foot",
    "kgx",
    "xterm",
    "x-terminal-emulator",
    "cmd",
    "powershell",
    "git-bash"
];

fn build_launcher_argv(exe: &str, script: &str, name: &str) -> Vec<String> {
    match name {
        "gnome-terminal" | "ptyxis" | "kgx" => {
            vec![exe.to_string(), "--".to_string(), "bash".to_string(), script.to_string()]
        }
        "konsole" => vec![exe.to_string(), "-e".to_string(), "bash".to_string(), script.to_string()],
        "xfce4-terminal" => {
            vec![exe.to_string(), "--command".to_string(), format!("bash {}", script)]
        }
        "tilix" => vec![exe.to_string(), "-e".to_string(), format!("bash {}", script)],
        "kitty" | "foot" => vec![exe.to_string(), "bash".to_string(), script.to_string()],
        "wezterm" => vec![
            exe.to_string(),
            "start".to_string(),
            "--".to_string(),
            "bash".to_string(),
            script.to_string(),
        ],
        "cmd" | "powershell" => vec![exe.to_string(), "-e".to_string(), script.to_string()],
        _ => vec![exe.to_string(), "-e".to_string(), "bash".to_string(), script.to_string()],
    }
}

fn find_in_path(name: &str) -> Option<std::path::PathBuf> {
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path).find_map(|dir| {
        let full = dir.join(name);
        if full.is_file() { Some(full) } else { None }
    })
}

pub fn find_terminal_command(script_path: &str) -> Option<Vec<String>> {
    for env_var in &["SETTUPPER_TERMINAL", "TERMINAL"] {
        if let Ok(terminal) = std::env::var(env_var) {
            if !terminal.is_empty() && find_in_path(&terminal).is_some() {
                return Some(vec![
                    terminal,
                    "-e".to_string(),
                    "bash".to_string(),
                    script_path.to_string(),
                ]);
            }
        }
    }

    for &name in KNOWN_TERMINALS {
        if let Some(exe) = find_in_path(name) {
            let exe_str = exe.to_string_lossy().into_owned();
            return Some(build_launcher_argv(&exe_str, script_path, name));
        }
    }

    None
}

pub fn run_groups_in_terminal(
    groups: &[CommandGroup],
    cancel: &Arc<AtomicBool>,
    on_status: impl Fn(String),
) -> TerminalRunResult {
    let mut result = TerminalRunResult::default();

    if groups.is_empty() {
        result.launched = true;
        return result;
    }

    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let tmp_dir = std::env::temp_dir().join(format!("settupper-{}", ts));

    if let Err(e) = std::fs::create_dir_all(&tmp_dir) {
        result.error = format!("Falha ao criar diretório temporário: {}", e);
        return result;
    }

    let script_path = tmp_dir.join("run.sh");
    let result_path = tmp_dir.join("result.tsv");
    let done_path = tmp_dir.join("done");

    let script = build_terminal_script(
        groups,
        &result_path.to_string_lossy(),
        &done_path.to_string_lossy(),
    );

    if let Err(e) = std::fs::write(&script_path, script) {
        result.error = format!("Falha ao escrever script: {}", e);
        let _ = std::fs::remove_dir_all(&tmp_dir);
        return result;
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&script_path, std::fs::Permissions::from_mode(0o755));
    }

    let script_str = script_path.to_string_lossy().into_owned();
    let argv = match find_terminal_command(&script_str) {
        Some(a) => a,
        None => {
            result.error =
                "Nenhum emulador de terminal encontrado."
                    .to_string();
            let _ = std::fs::remove_dir_all(&tmp_dir);
            return result;
        }
    };

    result.terminal = argv[0].clone();
    let term_name = std::path::Path::new(&argv[0])
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();
    on_status(format!("Abrindo terminal externo ({})...", term_name));

    let mut process = match std::process::Command::new(&argv[0]).args(&argv[1..]).spawn() {
        Ok(p) => p,
        Err(e) => {
            result.error = format!("Falha ao abrir terminal: {}", e);
            let _ = std::fs::remove_dir_all(&tmp_dir);
            return result;
        }
    };

    result.launched = true;

    loop {
        if done_path.exists() {
            break;
        }
        if cancel.load(Ordering::Relaxed) {
            on_status("Cancelando: feche o terminal externo para encerrar.".to_string());
            break;
        }
        if let Ok(Some(_)) = process.try_wait() {
            if !done_path.exists() {
                on_status("Terminal fechado antes de concluir.".to_string());
            }
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(300));
    }

    result.exit_codes = parse_result_file(&result_path.to_string_lossy());
    let _ = std::fs::remove_dir_all(&tmp_dir);
    result
}

fn parse_result_file(path: &str) -> HashMap<usize, i32> {
    let mut codes = HashMap::new();
    let Ok(content) = std::fs::read_to_string(path) else {
        return codes;
    };
    for line in content.lines() {
        let mut parts = line.splitn(2, '\t');
        let (Some(id_str), Some(rc_str)) = (parts.next(), parts.next()) else {
            continue;
        };
        if let (Ok(id), Ok(rc)) = (id_str.trim().parse::<usize>(), rc_str.trim().parse::<i32>()) {
            codes.insert(id, rc);
        }
    }
    codes
}
