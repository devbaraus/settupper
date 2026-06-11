mod app;
mod config;
mod core;
mod events;
mod ui;
mod workers;

use anyhow::{Context, Result};
use clap::Parser;
use crossterm::{
    event::{self, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::time::Duration;

#[derive(Parser)]
#[command(name = "settupper", about = "Declarative package manager with TUI")]
struct Args {
    /// Path or URL to the configuration file (YAML/JSON)
    config: Option<String>,

    /// Simulate actions without executing them
    #[arg(long)]
    dry_run: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let config_path = args
        .config
        .or_else(|| {
            config::find_default_config().map(|path| path.to_string_lossy().into_owned())
        })
        .context("No config file found. Pass a path or create ~/.config/settupper/packages.yaml")?;

    let pkg_config = config::load_config_source(&config_path)
        .with_context(|| format!("Failed to load config: {config_path}"))?;

    let distro = core::detect_distro();

    let mut state = app::AppState::new(pkg_config, config_path, distro, args.dry_run);

    // Initial refresh
    events::start_refresh(&mut state);

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_loop(&mut terminal, &mut state).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

async fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    state: &mut app::AppState,
) -> Result<()> {
    loop {
        terminal.draw(|f| ui::render(f, state))?;

        // Drain app events (non-blocking)
        while let Ok(ev) = state.event_rx.try_recv() {
            events::handle_app_event(state, ev);
        }

        // Poll for key events (16ms ≈ 60fps)
        if event::poll(Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != event::KeyEventKind::Release {
                    if events::handle_key_event(state, key) {
                        break; // quit
                    }
                }
            }
        }
    }

    Ok(())
}
