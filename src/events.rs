use crossterm::event::{self, KeyCode};
use crate::app::{Action, ActionPlan, AppEvent, AppState, Screen};
use crate::core::{select_commands, topological_sort, CommandGroup};
use crate::workers;

pub fn handle_key_event(state: &mut AppState, key: event::KeyEvent) -> bool {
    // Handle modal screens first
    match &state.screen.clone() {
        Screen::RebootRequired { .. } => {
            if matches!(key.code, KeyCode::Enter | KeyCode::Esc) {
                state.screen = Screen::Main;
            }
            return false;
        }
        _ => {}
    }

    if state.busy && !matches!(key.code, KeyCode::Char('q') | KeyCode::Esc) {
        // Allow cancel with Esc while busy
        if key.code == KeyCode::Esc {
            state.cancel.store(true, std::sync::atomic::Ordering::Relaxed);
        }
        return false;
    }

    match key.code {
        KeyCode::Char('q') => return true, // quit signal
        KeyCode::Up | KeyCode::Char('k') => state.move_cursor_up(),
        KeyCode::Down | KeyCode::Char('j') => state.move_cursor_down(),
        KeyCode::Char(' ') => state.toggle_selection(),
        KeyCode::Esc => state.clear_selection(),
        KeyCode::Tab => cycle_group(state),
        KeyCode::Char('i') => queue_action(state, Action::Install),
        KeyCode::Char('u') => queue_action(state, Action::Update),
        KeyCode::Char('d') => queue_action(state, Action::Uninstall),
        KeyCode::Char('a') => queue_action_smart(state),
        KeyCode::Char('A') => queue_action_smart_all(state),
        KeyCode::Char('r') => start_refresh(state),
        KeyCode::Char('e') => export_snapshot(state),
        KeyCode::PageUp => {
            for _ in 0..10 { state.move_cursor_up(); }
        }
        KeyCode::PageDown => {
            for _ in 0..10 { state.move_cursor_down(); }
        }
        _ => {}
    }

    false
}

fn cycle_group(state: &mut AppState) {
    let groups = state.config.groups.clone();
    if groups.is_empty() {
        return;
    }

    let current_idx = if state.active_group.is_empty() {
        None
    } else {
        groups.iter().position(|g| g.id == state.active_group)
    };

    let next = match current_idx {
        None => Some(0),
        Some(i) if i + 1 < groups.len() => Some(i + 1),
        _ => None,
    };

    let new_group = next.map(|i| groups[i].id.clone()).unwrap_or_default();
    state.apply_group_filter(&new_group.clone());
}

fn queue_action(state: &mut AppState, action: Action) {
    let targets = state.target_indices();
    if targets.is_empty() {
        return;
    }

    let distro = state.distro.clone();
    let plans: Vec<ActionPlan> = targets
        .into_iter()
        .filter(|&idx| {
            let Some(s) = state.statuses.get(idx) else { return false };
            if !s.supported { return false; }
            let map = match action {
                Action::Install => &s.app.install,
                Action::Update => &s.app.update,
                Action::Uninstall => &s.app.uninstall,
            };
            select_commands(map, &distro).is_some()
        })
        .map(|app_index| ActionPlan { app_index, action: action.clone() })
        .collect();

    if plans.is_empty() {
        state.push_log(format!("# No app with {} available", action.as_str()));
        return;
    }

    launch_plans(state, plans);
}

fn queue_action_smart(state: &mut AppState) {
    let targets = state.target_indices();
    let plans: Vec<ActionPlan> = targets
        .into_iter()
        .filter_map(|idx| {
            let action = state.smart_action(idx)?;
            Some(ActionPlan { app_index: idx, action })
        })
        .collect();

    if plans.is_empty() {
        state.push_log("# No smart action available".to_string());
        return;
    }

    launch_plans(state, plans);
}

fn queue_action_smart_all(state: &mut AppState) {
    let sorted = topological_sort(&state.statuses);
    let visible_set: std::collections::HashSet<usize> = state.visible_indices.iter().copied().collect();

    let plans: Vec<ActionPlan> = sorted
        .into_iter()
        .filter(|idx| visible_set.contains(idx))
        .filter_map(|idx| {
            let action = state.smart_action(idx)?;
            Some(ActionPlan { app_index: idx, action })
        })
        .collect();

    if plans.is_empty() {
        state.push_log("# No smart-all action available".to_string());
        return;
    }

    launch_plans(state, plans);
}



pub fn launch_plans(state: &mut AppState, plans: Vec<ActionPlan>) {
    // Build CommandGroups — one per plan, group_id = app_index for result correlation.
    let groups: Vec<CommandGroup> = plans
        .iter()
        .filter_map(|plan| {
            let status = state.statuses.get(plan.app_index)?;
            let map = match plan.action {
                Action::Install => &status.app.install,
                Action::Update => &status.app.update,
                Action::Uninstall => &status.app.uninstall,
            };
            let commands = select_commands(map, &state.distro)?.clone();
            Some(CommandGroup {
                group_id: plan.app_index,
                label: format!("{} ({})", status.app.name, plan.action.as_str()),
                commands,
            })
        })
        .collect();

    if groups.is_empty() {
        state.push_log("# No commands to execute.".to_string());
        return;
    }

    state.busy = true;
    state.cancel.store(false, std::sync::atomic::Ordering::Relaxed);
    let cancel = state.cancel.clone();

    workers::spawn_execute_plans(
        groups,
        plans,
        state.statuses.clone(),
        state.distro.clone(),
        state.dry_run,
        cancel,
        state.event_tx.clone(),
    );
}

pub fn start_refresh(state: &mut AppState) {
    state.busy = true;
    state.statuses.clear();
    state.push_log("# Updating status...".to_string());
    workers::spawn_refresh_all(
        state.config.clone(),
        state.distro.clone(),
        state.event_tx.clone(),
    );
}

fn export_snapshot(state: &mut AppState) {
    let snapshot = build_snapshot(state);
    let json = serde_json::to_string_pretty(&snapshot).unwrap_or_default();
    let ts = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("settupper");
    let _ = std::fs::create_dir_all(&config_dir);
    let path = config_dir.join(format!("snapshot_{}.json", ts));
    match std::fs::write(&path, &json) {
        Ok(_) => state.push_log(format!("# Exported: {}", path.display())),
        Err(e) => state.push_log(format!("# Error exporting: {}", e)),
    }
}

#[derive(serde::Serialize)]
struct SnapshotApp {
    id: String,
    name: String,
    group: String,
    supported: bool,
    installed: bool,
}

#[derive(serde::Serialize)]
struct Snapshot {
    distro: String,
    config: String,
    timestamp: String,
    apps: Vec<SnapshotApp>,
}

fn build_snapshot(state: &AppState) -> Snapshot {
    let apps = state
        .statuses
        .iter()
        .map(|s| SnapshotApp {
            id: s.app.id.clone(),
            name: s.app.name.clone(),
            group: s.app.group.clone(),
            supported: s.supported,
            installed: s.installed,
        })
        .collect();

    Snapshot {
        distro: state.distro.as_str().to_string(),
        config: state.config_path.clone(),
        timestamp: chrono::Local::now().to_rfc3339(),
        apps,
    }
}

pub fn handle_app_event(state: &mut AppState, event: AppEvent) {
    match event {
        AppEvent::StatusesLoaded(statuses) => {
            state.statuses = statuses;
            state.busy = false;
            state.push_log("# Status updated".to_string());
        }
        AppEvent::CommandOutput(line) => {
            state.push_log(line);
        }
        AppEvent::ActionFinished { index, success, reboot } => {
            // Refresh status for this app
            if let Some(app) = state.config.apps.get(index) {
                let new_status = crate::core::resolve_status(app, &state.distro);
                if let Some(s) = state.statuses.get_mut(index) {
                    *s = new_status;
                }
            }

            if reboot && success {
                let app_name = state.config.apps.get(index)
                    .map(|a| a.name.clone())
                    .unwrap_or_default();
                let pending = state.pending_plans.iter()
                    .filter_map(|p| state.config.apps.get(p.app_index))
                    .map(|a| a.name.clone())
                    .collect();
                state.screen = Screen::RebootRequired { app_name, pending };
            }
        }
        AppEvent::AllPlansFinished => {
            state.busy = false;
            state.push_log("# Completed".to_string());
            state.pending_plans.clear();
        }
        AppEvent::Error(msg) => {
            state.push_log(format!("# ERROR: {}", msg));
            state.busy = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{AppConfig, PackageConfig};
    use crate::core::{AppStatus, Distro};
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use std::collections::HashMap;

    fn app(id: &str) -> AppConfig {
        AppConfig {
            id: id.to_string(),
            name: id.to_string(),
            ..AppConfig::default()
        }
    }

    fn state() -> AppState {
        AppState::new(
            PackageConfig {
                version: 1,
                apps: vec![app("one"), app("two")],
                groups: vec![],
            },
            "packages.yaml".to_string(),
            Distro::Ubuntu,
            false,
        )
    }

    fn status(app: AppConfig, supported: bool, installed: bool) -> AppStatus {
        AppStatus {
            app,
            supported,
            installed,
            message: String::new(),
        }
    }

    #[test]
    fn handle_key_event_moves_selection_and_quits() {
        let mut state = state();

        assert!(!handle_key_event(
            &mut state,
            KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)
        ));
        assert_eq!(state.table_cursor, 1);

        assert!(!handle_key_event(
            &mut state,
            KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE)
        ));
        assert!(state.selected_indices.contains(&1));

        assert!(!handle_key_event(
            &mut state,
            KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)
        ));
        assert!(state.selected_indices.is_empty());

        assert!(handle_key_event(
            &mut state,
            KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE)
        ));
    }

    #[test]
    fn handle_key_event_closes_reboot_screen_on_enter() {
        let mut state = state();
        state.screen = Screen::RebootRequired {
            app_name: "one".to_string(),
            pending: vec!["two".to_string()],
        };

        let quit = handle_key_event(
            &mut state,
            KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
        );

        assert!(!quit);
        assert_eq!(state.screen, Screen::Main);
    }

    #[test]
    fn handle_app_event_loads_statuses_and_marks_idle() {
        let mut state = state();
        state.busy = true;
        let statuses = vec![status(app("one"), true, false)];

        handle_app_event(&mut state, AppEvent::StatusesLoaded(statuses));

        assert!(!state.busy);
        assert_eq!(state.statuses.len(), 1);
        assert_eq!(state.log_lines.last(), Some(&"# Status updated".to_string()));
    }

    #[test]
    fn handle_app_event_reboot_success_opens_modal_with_pending_apps() {
        let mut state = state();
        let mut install = HashMap::new();
        install.insert("ubuntu".to_string(), vec!["install".to_string()]);
        state.config.apps[0].install = install.clone();
        state.config.apps[0].reboot_on.insert("install".to_string(), true);
        state.config.apps[1].install = install;
        state.statuses = state
            .config
            .apps
            .iter()
            .cloned()
            .map(|app| status(app, true, false))
            .collect();
        state.pending_plans = vec![
            ActionPlan { app_index: 0, action: Action::Install },
            ActionPlan { app_index: 1, action: Action::Install },
        ];

        handle_app_event(
            &mut state,
            AppEvent::ActionFinished { index: 0, success: true, reboot: true },
        );

        assert_eq!(
            state.screen,
            Screen::RebootRequired {
                app_name: "one".to_string(),
                pending: vec!["one".to_string(), "two".to_string()],
            }
        );
    }

    #[test]
    fn handle_app_event_all_plans_finished_clears_busy_and_pending() {
        let mut state = state();
        state.busy = true;
        state.pending_plans = vec![ActionPlan { app_index: 0, action: Action::Install }];

        handle_app_event(&mut state, AppEvent::AllPlansFinished);

        assert!(!state.busy);
        assert!(state.pending_plans.is_empty());
        assert_eq!(state.log_lines.last(), Some(&"# Completed".to_string()));
    }
}
