use crate::app::{Action, ActionPlan, AppEvent};
use crate::config::PackageConfig;
use crate::core::{
    action_requires_reboot, resolve_status, run_command, select_commands, AppStatus, Distro,
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
        let mut statuses = Vec::new();
        for app in &config.apps {
            statuses.push(resolve_status(app, &distro));
        }
        let _ = event_tx.send(AppEvent::StatusesLoaded(statuses));
    });
}

pub fn spawn_validate_password(password: String, event_tx: UnboundedSender<AppEvent>) {
    tokio::task::spawn_blocking(move || {
        if crate::core::validate_sudo_password(&password) {
            let _ = event_tx.send(AppEvent::PasswordValidated);
        } else {
            let _ = event_tx.send(AppEvent::PasswordRejected);
        }
    });
}

pub fn spawn_execute_plans(
    plans: Vec<ActionPlan>,
    statuses: Vec<AppStatus>,
    distro: Distro,
    sudo_password: Option<String>,
    dry_run: bool,
    cancel: Arc<AtomicBool>,
    event_tx: UnboundedSender<AppEvent>,
) {
    tokio::task::spawn_blocking(move || {
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

            let Some(commands) = select_commands(map, &distro) else {
                let _ = event_tx.send(AppEvent::CommandOutput(format!(
                    "# {} ({}): sem comandos para esta distro",
                    status.app.name, action_str
                )));
                continue;
            };

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

                if dry_run {
                    let _ = event_tx.send(AppEvent::CommandOutput("[dry-run]".to_string()));
                    continue;
                }

                let tx_clone = event_tx.clone();
                let (line_tx, mut line_rx) = tokio::sync::mpsc::unbounded_channel::<String>();

                let result = run_command(
                    cmd,
                    sudo_password.as_deref(),
                    &cancel,
                    Some(&line_tx),
                );

                match result {
                    Ok(r) => {
                        let _ = event_tx.send(AppEvent::CommandOutput(format!("exit={}", r.exit_code)));
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
    });
}
