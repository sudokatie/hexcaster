//! Input handling.

use crossterm::event::{self, Event, KeyCode, KeyEvent};
use std::time::Duration;

/// Game inputs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Input {
    Move(usize), // Direction 0-5
    Attack,
    CastSpell,
    CraftSpell,
    SelectSpell(usize),
    Confirm,
    Cancel,
    Wait,
    Quit,
}

/// Poll for input with timeout.
pub fn poll_input(timeout_ms: u64) -> Option<Input> {
    if event::poll(Duration::from_millis(timeout_ms)).ok()? {
        if let Event::Key(key) = event::read().ok()? {
            return key_to_input(key);
        }
    }
    None
}

fn key_to_input(key: KeyEvent) -> Option<Input> {
    match key.code {
        // Movement (vim keys + numpad style)
        KeyCode::Char('e') | KeyCode::Char('6') => Some(Input::Move(0)), // East
        KeyCode::Char('w') | KeyCode::Char('9') => Some(Input::Move(1)), // NE
        KeyCode::Char('q') | KeyCode::Char('7') => Some(Input::Move(2)), // NW
        KeyCode::Char('a') | KeyCode::Char('4') => Some(Input::Move(3)), // West
        KeyCode::Char('z') | KeyCode::Char('1') => Some(Input::Move(4)), // SW
        KeyCode::Char('x') | KeyCode::Char('3') => Some(Input::Move(5)), // SE

        // Actions
        KeyCode::Char(' ') => Some(Input::Attack),
        KeyCode::Char('c') => Some(Input::CastSpell),
        KeyCode::Char('r') => Some(Input::CraftSpell),
        KeyCode::Char('.') | KeyCode::Char('5') => Some(Input::Wait),
        KeyCode::Enter => Some(Input::Confirm),
        KeyCode::Esc => Some(Input::Cancel),

        // Spell selection (use F keys to avoid numpad conflict)
        KeyCode::F(1) => Some(Input::SelectSpell(0)),
        KeyCode::F(2) => Some(Input::SelectSpell(1)),
        KeyCode::F(3) => Some(Input::SelectSpell(2)),

        // Quit
        KeyCode::Char('Q') => Some(Input::Quit),

        _ => None,
    }
}
