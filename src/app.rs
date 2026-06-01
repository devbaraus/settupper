use crate::config::PackageConfig;
use crate::core::{AppStatus, Distro};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::mpsc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Install,
    Update,
    Uninstall,
}

impl Action {
    pub fn as_str(&self) -> &'static str {
        match self {
            Action::Install => "install",
            Action::Update => "update",
            Action::Uninstall => "uninstall",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ActionPlan {
    pub app_index: usize,
    pub action: Action,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    Main,
    RebootRequired { app_name: String, pending: Vec<String> },
}

#[derive(Debug)]
pub enum AppEvent {
    StatusesLoaded(Vec<AppStatus>),
    CommandOutput(String),
    ActionFinished { index: usize, success: bool, reboot: bool },
    AllPlansFinished,
    Error(String),
}

pub struct AppState {
    pub config: PackageConfig,
    pub config_path: String,
    pub distro: Distro,
    pub dry_run: bool,

    pub statuses: Vec<AppStatus>,
    pub visible_indices: Vec<usize>,
    pub selected_indices: HashSet<usize>,
    pub active_group: String,

    pub table_cursor: usize,
    pub split_ratio: f64,

    pub screen: Screen,
    pub busy: bool,

    pub log_lines: Vec<String>,
    pub detail_scroll: u16,

    pub pending_plans: Vec<ActionPlan>,

    pub event_tx: mpsc::UnboundedSender<AppEvent>,
    pub event_rx: mpsc::UnboundedReceiver<AppEvent>,

    pub cancel: Arc<std::sync::atomic::AtomicBool>,
}

impl AppState {
    pub fn new(config: PackageConfig, config_path: String, distro: Distro, dry_run: bool) -> Self {
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        let visible_indices: Vec<usize> = (0..config.apps.len()).collect();
        Self {
            config,
            config_path,
            distro,
            dry_run,
            statuses: Vec::new(),
            visible_indices,
            selected_indices: HashSet::new(),
            active_group: String::new(),
            table_cursor: 0,
            split_ratio: 0.55,
            screen: Screen::Main,
            busy: false,
            log_lines: Vec::new(),
            detail_scroll: 0,
            pending_plans: Vec::new(),
            event_tx,
            event_rx,
            cancel: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }

    pub fn current_real_index(&self) -> Option<usize> {
        self.visible_indices.get(self.table_cursor).copied()
    }

    pub fn target_indices(&self) -> Vec<usize> {
        if !self.selected_indices.is_empty() {
            let mut sel: Vec<usize> = self.selected_indices.iter().copied().collect();
            sel.sort();
            sel
        } else if let Some(idx) = self.current_real_index() {
            vec![idx]
        } else {
            vec![]
        }
    }

    pub fn toggle_selection(&mut self) {
        if let Some(idx) = self.current_real_index() {
            if self.selected_indices.contains(&idx) {
                self.selected_indices.remove(&idx);
            } else {
                self.selected_indices.insert(idx);
            }
        }
    }

    pub fn clear_selection(&mut self) {
        self.selected_indices.clear();
    }

    pub fn move_cursor_up(&mut self) {
        if self.table_cursor > 0 {
            self.table_cursor -= 1;
        }
    }

    pub fn move_cursor_down(&mut self) {
        if !self.visible_indices.is_empty() && self.table_cursor < self.visible_indices.len() - 1 {
            self.table_cursor += 1;
        }
    }

    pub fn apply_group_filter(&mut self, group: &str) {
        self.active_group = group.to_string();
        self.visible_indices = if group.is_empty() {
            (0..self.config.apps.len()).collect()
        } else {
            (0..self.config.apps.len())
                .filter(|&i| self.config.apps[i].group == group)
                .collect()
        };
        self.table_cursor = 0;
        self.selected_indices.clear();
    }

    pub fn smart_action(&self, idx: usize) -> Option<Action> {
        let status = self.statuses.get(idx)?;
        if !status.supported {
            return None;
        }
        if status.installed {
            if crate::core::select_commands(&status.app.update, &self.distro).is_some() {
                Some(Action::Update)
            } else {
                None
            }
        } else {
            if crate::core::select_commands(&status.app.install, &self.distro).is_some() {
                Some(Action::Install)
            } else {
                None
            }
        }
    }

    pub fn groups(&self) -> Vec<&crate::config::GroupConfig> {
        self.config.groups.iter().collect()
    }

    pub fn push_log(&mut self, line: String) {
        self.log_lines.push(line);
        // Keep last 500 lines
        if self.log_lines.len() > 500 {
            self.log_lines.remove(0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{AppConfig, GroupConfig};
    use std::collections::HashMap;

    fn app(id: &str, group: &str) -> AppConfig {
        AppConfig {
            id: id.to_string(),
            name: id.to_string(),
            group: group.to_string(),
            ..AppConfig::default()
        }
    }

    fn state_with_apps(apps: Vec<AppConfig>) -> AppState {
        AppState::new(
            PackageConfig {
                version: 1,
                apps,
                groups: vec![
                    GroupConfig { id: "dev".to_string(), name: "Dev".to_string() },
                    GroupConfig { id: "ops".to_string(), name: "Ops".to_string() },
                ],
            },
            "packages.yaml".to_string(),
            Distro::Ubuntu,
            false,
        )
    }

    #[test]
    fn target_indices_uses_sorted_selection_before_cursor() {
        let mut state = state_with_apps(vec![app("a", "dev"), app("b", "ops"), app("c", "dev")]);
        state.table_cursor = 1;

        assert_eq!(state.target_indices(), vec![1]);

        state.selected_indices.insert(2);
        state.selected_indices.insert(0);

        assert_eq!(state.target_indices(), vec![0, 2]);
    }

    #[test]
    fn apply_group_filter_updates_visible_rows_and_clears_selection() {
        let mut state = state_with_apps(vec![app("a", "dev"), app("b", "ops"), app("c", "dev")]);
        state.table_cursor = 2;
        state.selected_indices.insert(1);

        state.apply_group_filter("dev");

        assert_eq!(state.active_group, "dev");
        assert_eq!(state.visible_indices, vec![0, 2]);
        assert_eq!(state.table_cursor, 0);
        assert!(state.selected_indices.is_empty());

        state.apply_group_filter("");

        assert_eq!(state.visible_indices, vec![0, 1, 2]);
    }

    #[test]
    fn smart_action_installs_missing_apps_and_updates_installed_apps() {
        let mut install = HashMap::new();
        install.insert("ubuntu".to_string(), vec!["install".to_string()]);
        let mut update = HashMap::new();
        update.insert("ubuntu".to_string(), vec!["update".to_string()]);

        let mut state = state_with_apps(vec![app("missing", "dev"), app("installed", "dev")]);
        let mut missing = state.config.apps[0].clone();
        missing.install = install.clone();
        let mut installed = state.config.apps[1].clone();
        installed.install = install;
        installed.update = update;

        state.statuses = vec![
            crate::core::AppStatus {
                app: missing,
                supported: true,
                installed: false,
                message: String::new(),
            },
            crate::core::AppStatus {
                app: installed,
                supported: true,
                installed: true,
                message: String::new(),
            },
        ];

        assert_eq!(state.smart_action(0), Some(Action::Install));
        assert_eq!(state.smart_action(1), Some(Action::Update));
    }

    #[test]
    fn push_log_keeps_last_500_lines() {
        let mut state = state_with_apps(vec![]);

        for i in 0..505 {
            state.push_log(format!("line {i}"));
        }

        assert_eq!(state.log_lines.len(), 500);
        assert_eq!(state.log_lines.first(), Some(&"line 5".to_string()));
        assert_eq!(state.log_lines.last(), Some(&"line 504".to_string()));
    }
}
