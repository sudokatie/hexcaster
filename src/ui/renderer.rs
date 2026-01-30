//! Map and entity rendering.

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::dungeon::DungeonTile;
use crate::hex::{Hex, HexGrid};

/// Convert hex to screen coordinates (pointy-top orientation).
pub fn hex_to_screen(hex: Hex, origin: (u16, u16)) -> (u16, u16) {
    let x = hex.x as f32 * 2.0 + hex.y as f32;
    let y = hex.y as f32 * 1.5;

    (
        (origin.0 as f32 + x * 2.0) as u16,
        (origin.1 as f32 + y) as u16,
    )
}

/// Render the dungeon map.
pub fn render_map(
    frame: &mut Frame,
    area: Rect,
    grid: &HexGrid<DungeonTile>,
    player_pos: Hex,
    entities: &[(Hex, char, Color)],
) {
    let block = Block::default().borders(Borders::ALL).title(" Dungeon ");

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Center on player
    let center = (inner.x + inner.width / 2, inner.y + inner.height / 2);

    // Build display buffer
    let mut chars: Vec<(u16, u16, char, Color)> = Vec::new();

    // Render tiles
    for (hex, tile) in grid.iter() {
        let rel_hex = hex - player_pos;
        let (sx, sy) = hex_to_screen(rel_hex, center);

        if sx >= inner.x
            && sx < inner.x + inner.width
            && sy >= inner.y
            && sy < inner.y + inner.height
        {
            let (ch, color) = match tile {
                DungeonTile::Floor => ('.', Color::DarkGray),
                DungeonTile::Wall => ('#', Color::Gray),
                DungeonTile::Stairs => ('>', Color::Yellow),
            };
            chars.push((sx, sy, ch, color));
        }
    }

    // Render entities on top
    for (hex, glyph, color) in entities {
        let rel_hex = *hex - player_pos;
        let (sx, sy) = hex_to_screen(rel_hex, center);

        if sx >= inner.x
            && sx < inner.x + inner.width
            && sy >= inner.y
            && sy < inner.y + inner.height
        {
            chars.push((sx, sy, *glyph, *color));
        }
    }

    // Draw player last (always visible)
    let (px, py) = hex_to_screen(Hex::origin(), center);
    chars.push((px, py, '@', Color::White));

    // Render to frame
    for (x, y, ch, color) in chars {
        if x < frame.area().width && y < frame.area().height {
            let span = Span::styled(ch.to_string(), Style::default().fg(color));
            let para = Paragraph::new(span);
            frame.render_widget(para, Rect::new(x, y, 1, 1));
        }
    }
}

/// Render spell targeting preview.
pub fn render_targets(frame: &mut Frame, area: Rect, targets: &[Hex], player_pos: Hex) {
    let center = (area.x + area.width / 2, area.y + area.height / 2);

    for hex in targets {
        let rel_hex = *hex - player_pos;
        let (sx, sy) = hex_to_screen(rel_hex, center);

        if sx >= area.x && sx < area.x + area.width && sy >= area.y && sy < area.y + area.height {
            let span = Span::styled(
                "*",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            );
            let para = Paragraph::new(span);
            frame.render_widget(para, Rect::new(sx, sy, 1, 1));
        }
    }
}
