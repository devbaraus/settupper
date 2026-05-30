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
        state.push_log(format!("# Nenhum app com {} disponível", action.as_str()));
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
        state.push_log("# Nenhuma ação smart disponível".to_string());
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
        state.push_log("# Nenhuma ação smart-all disponível".to_string());
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
        state.push_log("# Nenhum comando para executar.".to_string());
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
    state.push_log("# Atualizando status...".to_string());
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
        Ok(_) => state.push_log(format!("# Exportado: {}", path.display())),
        Err(e) => state.push_log(format!("# Erro ao exportar: {}", e)),
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
            state.push_log("# Status atualizado".to_string());
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
            state.push_log("# Concluído".to_string());
            state.pending_plans.clear();
        }
        AppEvent::Error(msg) => {
            state.push_log(format!("# ERRO: {}", msg));
            state.busy = false;
        }
    }
}
