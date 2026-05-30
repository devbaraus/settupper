use crate::app::{Action, ActionPlan, AppEvent};
use crate::config::PackageConfig;
use crate::core::{
    action_requires_reboot, resolve_status, run_command, run_groups_in_terminal, select_commands,
    AppStatus, CommandGroup, Distro,
};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::mpsc::UnboundedSender;

pub fn spawn_refresh_all(
    config: PackageConfig,
    distro: Distro,
    event_tx: UnboundedSender<AppEvent>,
) {
    tokio::task::spawn_blocking(move || {
        let statuses: Vec<_> = config.apps.iter().map(|app| resolve_status(app, &distro)).collect();
        let _ = event_tx.send(AppEvent::StatusesLoaded(statuses));
    });
}

pub fn spawn_execute_plans(
    groups: Vec<CommandGroup>,
    plans: Vec<ActionPlan>,
    statuses: Vec<AppStatus>,
    distro: Distro,
    dry_run: bool,
    cancel: Arc<AtomicBool>,
    event_tx: UnboundedSender<AppEvent>,
) {
    tokio::task::spawn_blocking(move || {
        if dry_run {
            for plan in &plans {
                let Some(status) = statuses.get(plan.app_index) else { continue };
                let _ = event_tx.send(AppEvent::CommandOutput(format!(
                    "# [dry-run] {} ({})",
                    status.app.name,
                    plan.action.as_str()
                )));
                let _ = event_tx.send(AppEvent::ActionFinished {
                    index: plan.app_index,
                    success: true,
                    reboot: false,
                });
            }
            let _ = event_tx.send(AppEvent::AllPlansFinished);
            return;
        }

        let tx_clone = event_tx.clone();
        let result = run_groups_in_terminal(&groups, &cancel, move |msg| {
            let _ = tx_clone.send(AppEvent::CommandOutput(msg));
        });

        if !result.launched {
            // Fall back to inline execution with a warning
            let _ = event_tx.send(AppEvent::CommandOutput(format!(
                "# Aviso: {}. Executando inline...",
                result.error
            )));
            spawn_execute_plans_inline_fallback(plans, statuses, distro, cancel, event_tx);
            return;
        }

        for plan in &plans {
            let exit_code = result.exit_codes.get(&plan.app_index).copied().unwrap_or(-1);
            let success = exit_code == 0;

            if exit_code == -1 {
                // Group was not reached (cancelled or terminal closed early)
                continue;
            }

            let reboot = statuses
                .get(plan.app_index)
                .map(|s| action_requires_reboot(&s.app, plan.action.as_str()))
                .unwrap_or(false);

            let _ = event_tx.send(AppEvent::ActionFinished {
                index: plan.app_index,
                success,
                reboot,
            });

            if reboot && success {
                break;
            }
        }

        let _ = event_tx.send(AppEvent::AllPlansFinished);
    });
}

fn spawn_execute_plans_inline_fallback(
    plans: Vec<ActionPlan>,
    statuses: Vec<AppStatus>,
    distro: Distro,
    cancel: Arc<AtomicBool>,
    event_tx: UnboundedSender<AppEvent>,
) {
    for plan in &plans {
        if cancel.load(Ordering::Relaxed) {
            break;
        }

        let Some(status) = statuses.get(plan.app_index) else { continue };
        let action_str = plan.action.as_str();

        let map = match plan.action {
            Action::Install => &status.app.install,
            Action::Update => &status.app.update,
            Action::Uninstall => &status.app.uninstall,
        };

        let Some(commands) = select_commands(map, &distro) else { continue };

        let _ = event_tx.send(AppEvent::CommandOutput(format!(
            "\n==> {} ({})",
            status.app.name, action_str
        )));

        let mut success = true;

        for cmd in commands {
            if cancel.load(Ordering::Relaxed) {
                success = false;
                break;
            }

            let _ = event_tx.send(AppEvent::CommandOutput(format!("$ {}", cmd)));

            let result = run_command(cmd,  &cancel, None);
            match result {
                Ok(r) => {
                    for line in r.output.lines() {
                        if !line.trim().is_empty() {
                            let _ = event_tx.send(AppEvent::CommandOutput(line.to_string()));
                        }
                    }
                    let _ = event_tx
                        .send(AppEvent::CommandOutput(format!("exit={}", r.exit_code)));
                    if r.exit_code != 0 {
                        success = false;
                        break;
                    }
                }
                Err(e) => {
                    let _ = event_tx.send(AppEvent::CommandOutput(format!("ERRO: {}", e)));
                    success = false;
                    break;
                }
            }
        }

        let reboot = action_requires_reboot(&status.app, action_str);
        let _ = event_tx.send(AppEvent::ActionFinished {
            index: plan.app_index,
            success,
            reboot,
        });

        if reboot && success {
            break;
        }
    }

    let _ = event_tx.send(AppEvent::AllPlansFinished);
}
