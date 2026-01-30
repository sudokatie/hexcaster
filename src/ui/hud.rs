//! HUD elements: health, AP, inventory.

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Gauge, List, ListItem, Paragraph};

use crate::ecs::components::{ActionPoints, Health, Status};
use crate::magic::Rune;

/// Render health bar.
pub fn render_health(frame: &mut Frame, area: Rect, health: &Health) {
    let ratio = health.current as f64 / health.max as f64;
    let color = if ratio > 0.5 {
        Color::Green
    } else if ratio > 0.25 {
        Color::Yellow
    } else {
        Color::Red
    };

    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title(" HP "))
        .gauge_style(Style::default().fg(color))
        .ratio(ratio)
        .label(format!("{}/{}", health.current, health.max));

    frame.render_widget(gauge, area);
}

/// Render action points.
pub fn render_ap(frame: &mut Frame, area: Rect, ap: &ActionPoints) {
    let dots: String = (0..ap.max)
        .map(|i| if i < ap.current { '●' } else { '○' })
        .collect();

    let para = Paragraph::new(dots)
        .block(Block::default().borders(Borders::ALL).title(" AP "))
        .style(Style::default().fg(Color::Cyan));

    frame.render_widget(para, area);
}

/// Render rune inventory.
pub fn render_inventory(frame: &mut Frame, area: Rect, runes: &[Rune]) {
    let items: Vec<ListItem> = runes.iter().map(|r| ListItem::new(r.name())).collect();

    let list = List::new(items).block(Block::default().borders(Borders::ALL).title(" Runes "));

    frame.render_widget(list, area);
}

/// Render status effects.
pub fn render_status(frame: &mut Frame, area: Rect, status: &Status) {
    let text: String = status
        .effects
        .iter()
        .map(|e| format!("{:?}({})", e.kind, e.duration))
        .collect::<Vec<_>>()
        .join(" ");

    let para = Paragraph::new(if text.is_empty() {
        "No effects".to_string()
    } else {
        text
    })
    .block(Block::default().borders(Borders::ALL).title(" Status "));

    frame.render_widget(para, area);
}

/// Render message log.
pub fn render_messages(frame: &mut Frame, area: Rect, messages: &[String]) {
    let items: Vec<ListItem> = messages
        .iter()
        .rev()
        .take(area.height as usize - 2)
        .map(|m| ListItem::new(m.as_str()))
        .collect();

    let list = List::new(items).block(Block::default().borders(Borders::ALL).title(" Log "));

    frame.render_widget(list, area);
}
