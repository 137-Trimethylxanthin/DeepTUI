use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::app::App;

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let has_usage = app.usage.is_some() || app.is_loading_usage;
    let has_balance = app.balance.is_some() || app.is_loading_balance;

    let balance_height: u16 = if has_balance { 9 } else { 5 };
    let usage_height: u16 = if has_usage { 9 } else { 0 };
    let content_height = balance_height + usage_height + if has_usage { 1 } else { 0 };

    let chunks = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(content_height),
        Constraint::Min(1),
    ])
    .margin(1)
    .split(area);

    render_title(frame, chunks[0]);

    let content_chunks = Layout::vertical([
        Constraint::Length(balance_height),
        Constraint::Length(usage_height),
    ])
    .split(chunks[1]);

    render_balance(frame, content_chunks[0], app);

    if has_usage {
        render_usage(frame, content_chunks[1], app);
    }

    render_footer(frame, chunks[2], app);
}

fn render_title(frame: &mut Frame, area: Rect) {
    let title = Paragraph::new(Line::from(vec![
        Span::styled(
            "DeepSeek",
            Style::default()
                .fg(Color::LightCyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            " Credits",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
    ]));
    frame.render_widget(title, area);
}

fn render_balance(frame: &mut Frame, area: Rect, app: &App) {
    let label = Style::default().fg(Color::Gray);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Gray))
        .title(Span::styled(
            " Balance ",
            Style::default().fg(Color::LightCyan),
        ));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if app.is_loading_balance {
        let loading = Paragraph::new("Fetching...").style(Style::default().fg(Color::Yellow));
        frame.render_widget(loading, inner);
        return;
    }

    if let Some(ref err) = app.balance_error {
        let error = Paragraph::new(format!("Error: {err}"))
            .style(Style::default().fg(Color::Red))
            .wrap(Wrap { trim: true });
        frame.render_widget(error, inner);
        return;
    }

    if let Some(ref balance) = app.balance {
        let bi = &balance.balance_infos[0];
        let lines = vec![
            Line::from(vec![
                Span::styled(" Total:     ", label),
                Span::styled(
                    format!("${}", bi.total_balance),
                    Style::default()
                        .fg(Color::LightGreen)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(format!(" {}", bi.currency), label),
            ]),
            Line::from(vec![
                Span::styled(" Granted:   ", label),
                Span::styled(
                    format!("${}", bi.granted_balance),
                    Style::default().fg(Color::Yellow),
                ),
            ]),
            Line::from(vec![
                Span::styled(" Topped Up: ", label),
                Span::styled(
                    format!("${}", bi.topped_up_balance),
                    Style::default().fg(Color::LightBlue),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled(" Available: ", label),
                if balance.is_available {
                    Span::styled("Yes", Style::default().fg(Color::LightGreen))
                } else {
                    Span::styled("No", Style::default().fg(Color::Red))
                },
            ]),
        ];
        frame.render_widget(Paragraph::new(lines), inner);
        return;
    }

    let placeholder = Paragraph::new("No API key set.\nSet DEEPSEEK_API_KEY env var.")
        .style(Style::default().fg(Color::Gray))
        .wrap(Wrap { trim: true });
    frame.render_widget(placeholder, inner);
}

fn render_usage(frame: &mut Frame, area: Rect, app: &App) {
    let label = Style::default().fg(Color::Gray);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Gray))
        .title(Span::styled(
            " Usage ",
            Style::default().fg(Color::LightMagenta),
        ));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if app.is_loading_usage {
        let loading = Paragraph::new("Fetching...").style(Style::default().fg(Color::Yellow));
        frame.render_widget(loading, inner);
        return;
    }

    if let Some(ref err) = app.usage_error {
        let error = Paragraph::new(format!("Error: {err}"))
            .style(Style::default().fg(Color::Red))
            .wrap(Wrap { trim: true });
        frame.render_widget(error, inner);
        return;
    }

    if let Some(ref usage) = app.usage {
        let mut lines = vec![
            Line::from(vec![
                Span::styled(" Today:     ", label),
                Span::styled(
                    format_tokens(usage.tokens_today),
                    Style::default().fg(Color::LightGreen),
                ),
                Span::styled(" tokens", label),
            ]),
            Line::from(vec![
                Span::styled(" Month:     ", label),
                Span::styled(
                    format_tokens(usage.tokens_month),
                    Style::default().fg(Color::Yellow),
                ),
                Span::styled(" tokens", label),
            ]),
            Line::from(vec![
                Span::styled(" All Time:  ", label),
                Span::styled(
                    format_tokens(usage.tokens_all_time),
                    Style::default().fg(Color::LightCyan),
                ),
                Span::styled(" tokens", label),
            ]),
        ];

        if let Some(ref usd) = app.balance_remaining_usd {
            lines.push(Line::from(vec![
                Span::styled(" Balance:   ", label),
                Span::styled(format!("${usd}"), Style::default().fg(Color::LightBlue)),
            ]));
        }

        frame.render_widget(Paragraph::new(lines), inner);
        return;
    }

    let placeholder = Paragraph::new("Set DEEPSEEK_PLATFORM_TOKEN for usage stats.")
        .style(Style::default().fg(Color::Gray))
        .wrap(Wrap { trim: true });
    frame.render_widget(placeholder, inner);
}

fn render_footer(frame: &mut Frame, area: Rect, app: &App) {
    let now = chrono::Local::now().format("%H:%M:%S").to_string();
    let bal_update = app
        .last_balance_update
        .as_ref()
        .map(|t| format!("bal:{}", t.format("%H:%M:%S")))
        .unwrap_or_else(|| "bal:never".into());
    let usage_update = app
        .last_usage_update
        .as_ref()
        .map(|t| format!("use:{}", t.format("%H:%M:%S")))
        .unwrap_or_else(|| "use:never".into());

    let footer_text = Line::from(vec![Span::styled(
        format!(" [R]efresh  [Q]uit  |  {bal_update}  {usage_update}  |  {now}"),
        Style::default().fg(Color::Gray),
    )]);

    let footer = Paragraph::new(footer_text);
    frame.render_widget(footer, area);
}

fn format_tokens(n: u64) -> String {
    if n >= 1_000_000_000 {
        format!("{:.2}B", n as f64 / 1_000_000_000.0)
    } else if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 10_000 {
        format!("{:.0}K", n as f64 / 1_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}
