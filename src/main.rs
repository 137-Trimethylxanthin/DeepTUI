mod api;
mod app;
mod platform;
mod ui;

use std::io;

use app::App;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let api_key = std::env::var("DEEPSEEK_API_KEY").ok();
    let platform_token = std::env::var("DEEPSEEK_PLATFORM_TOKEN").ok();

    let mut app = App::new(api_key, platform_token);

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run(&mut terminal, &mut app).await;

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result?;
    Ok(())
}

async fn run(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> anyhow::Result<()> {
    let balance_secs: u64 = std::env::var("DEEPSEEK_BALANCE_SECS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(30);

    let usage_secs: u64 = std::env::var("DEEPSEEK_USAGE_SECS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(900);

    let mut balance_tick = tokio::time::interval(std::time::Duration::from_secs(balance_secs));
    let mut usage_tick = tokio::time::interval(std::time::Duration::from_secs(usage_secs));

    balance_tick.tick().await;
    app.refresh_balance().await;

    if app.has_platform() {
        usage_tick.tick().await;
        app.refresh_usage().await;
    }

    loop {
        terminal.draw(|f| ui::render(f, app))?;

        tokio::select! {
            _ = balance_tick.tick() => {
                app.refresh_balance().await;
            }
            _ = usage_tick.tick() => {
                if app.has_platform() {
                    app.refresh_usage().await;
                }
            }
            ev = async {
                loop {
                    if event::poll(std::time::Duration::from_millis(100))? {
                        return event::read();
                    }
                }
                #[allow(unreachable_code)]
                Ok::<Event, io::Error>(Event::Resize(0, 0))
            } => {
                if let Event::Key(key) = ev? {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => break,
                        KeyCode::Char('r') => {
                            app.refresh_balance().await;
                            if app.has_platform() {
                                app.refresh_usage().await;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    Ok(())
}
