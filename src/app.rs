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
