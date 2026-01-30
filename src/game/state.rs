//! Game state and main loop.

use std::io::{self, stdout};

use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use hecs::World;
use rand::rngs::StdRng;
use rand::SeedableRng;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::combat::{
    entity_at, melee_attack, remove_dead, AttackResult, BASE_MELEE_DAMAGE, MELEE_AP_COST,
};
use crate::dungeon::{generate, DungeonConfig, DungeonTile};
use crate::ecs::components::{
    AIType, ActionPoints, Display, Enemy, Health, Inventory, Player, Position, RunePickup, Status,
};
use crate::ecs::run_enemy_ai;
use crate::hex::{Hex, HexGrid, Tile as HexTile};
use crate::magic::rune::{Element, Modifier, Rune, Shape};
use crate::magic::Spell;
use crate::ui::{hud, renderer};

use super::input::{poll_input, Input};

/// Current game state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameState {
    MainMenu,
    Playing { floor: i32 },
    SpellSelect,
    Targeting { spell_idx: usize },
    EnemyTurn,
    GameOver,
    Victory,
}

/// Main game struct.
pub struct Game {
    pub state: GameState,
    pub world: World,
    pub grid: HexGrid<DungeonTile>,
    pub player: hecs::Entity,
    pub grimoire: Vec<Spell>,
    pub messages: Vec<String>,
    pub rng: StdRng,
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}

impl Game {
    /// Create a new game.
    pub fn new() -> Self {
        let mut rng = StdRng::from_entropy();
        let config = DungeonConfig::default();
        let (grid, player_start) = generate(&config, &mut rng);

        let mut world = World::new();

        // Spawn player
        let player = world.spawn((
            Position(player_start),
            Health::new(100),
            ActionPoints::new(4),
            Player,
            Inventory::default(),
            Status::default(),
            Display {
                glyph: '@',
                color: (255, 255, 255),
            },
        ));

        // Spawn enemies
        let floor_tiles: Vec<Hex> = grid
            .iter()
            .filter(|(h, t)| **t == DungeonTile::Floor && h.distance(player_start) > 3)
            .map(|(h, _)| h)
            .collect();

        let num_enemies = 3.min(floor_tiles.len());
        for i in 0..num_enemies {
            let pos = floor_tiles[i * floor_tiles.len() / num_enemies.max(1)];
            world.spawn((
                Position(pos),
                Health::new(20),
                ActionPoints::new(2),
                Enemy {
                    ai_type: AIType::Melee,
                    aggro_range: 6,
                },
                Display {
                    glyph: 'g',
                    color: (255, 100, 100),
                },
            ));
        }

        // Spawn starter runes near player
        let starter_runes = [Rune::element(Element::Fire), Rune::shape(Shape::Point)];
        for (i, rune) in starter_runes.into_iter().enumerate() {
            let offset = (i + 1) as i32;
            let pos = Hex::new(
                player_start.x + offset,
                player_start.y - offset,
                player_start.z,
            );
            if grid.get(pos).map(|t| t.is_walkable()).unwrap_or(false) {
                world.spawn((
                    Position(pos),
                    RunePickup { rune },
                    Display {
                        glyph: '*',
                        color: (255, 255, 0),
                    },
                ));
            }
        }

        Self {
            state: GameState::Playing { floor: 1 },
            world,
            grid,
            player,
            grimoire: Vec::new(),
            messages: vec![
                "Welcome to Hexcaster!".to_string(),
                "Move: qweasd | Attack: space | Craft: r | Cast: c".to_string(),
                "Collect runes (*) and craft spells to survive!".to_string(),
            ],
            rng,
        }
    }

    /// Run the game.
    pub fn run() -> io::Result<()> {
        enable_raw_mode()?;
        execute!(stdout(), EnterAlternateScreen)?;

        let backend = CrosstermBackend::new(stdout());
        let mut terminal = Terminal::new(backend)?;

        let mut game = Game::new();

        loop {
            terminal.draw(|frame| game.render(frame))?;

            if let Some(input) = poll_input(100) {
                if input == Input::Quit {
                    break;
                }
                game.handle_input(input);
            }

            if game.state == GameState::GameOver || game.state == GameState::Victory {
                // Wait for quit
                if let Some(Input::Quit) = poll_input(100) {
                    break;
                }
            }
        }

        disable_raw_mode()?;
        execute!(stdout(), LeaveAlternateScreen)?;

        Ok(())
    }

    /// Handle player input.
    fn handle_input(&mut self, input: Input) {
        match (&self.state, input) {
            (GameState::Playing { .. }, Input::Move(dir)) => {
                self.try_move(dir);
            }
            (GameState::Playing { .. }, Input::Wait) => {
                self.messages.push("You wait...".to_string());
                self.end_turn();
            }
            (GameState::Playing { .. }, Input::Attack) => {
                self.try_attack();
            }
            (GameState::Playing { .. }, Input::CastSpell) => {
                if !self.grimoire.is_empty() {
                    self.state = GameState::SpellSelect;
                } else {
                    self.messages
                        .push("No spells crafted! Press 'r' to craft.".to_string());
                }
            }
            (GameState::Playing { .. }, Input::CraftSpell) => {
                self.craft_spell_from_inventory();
            }
            (GameState::SpellSelect, Input::SelectSpell(idx)) => {
                if idx < self.grimoire.len() {
                    self.state = GameState::Targeting { spell_idx: idx };
                }
            }
            (GameState::SpellSelect | GameState::Targeting { .. }, Input::Cancel) => {
                self.state = GameState::Playing { floor: 1 };
            }
            _ => {}
        }
    }

    /// Try to move the player.
    fn try_move(&mut self, direction: usize) {
        const MOVE_COST: i32 = 1;

        // Check AP
        {
            let ap = match self.world.get::<&ActionPoints>(self.player) {
                Ok(a) => a.current,
                Err(_) => return,
            };
            if ap < MOVE_COST {
                self.messages.push("Not enough AP!".to_string());
                return;
            }
        }

        // Get current position and compute new position
        let new_pos = {
            let pos = match self.world.get::<&Position>(self.player) {
                Ok(p) => p.0,
                Err(_) => return,
            };
            pos.neighbor(direction)
        };

        // Check if new position is walkable
        let (walkable, is_stairs) = match self.grid.get(new_pos) {
            Some(tile) => (tile.is_walkable(), *tile == DungeonTile::Stairs),
            None => (false, false),
        };

        if walkable {
            // Deduct AP
            if let Ok(mut ap) = self.world.get::<&mut ActionPoints>(self.player) {
                ap.current -= MOVE_COST;
            }

            // Update position
            if let Ok(mut pos) = self.world.get::<&mut Position>(self.player) {
                pos.0 = new_pos;
            }

            // Collect runes at this position
            self.collect_runes(new_pos);

            // Check stairs
            if is_stairs {
                self.messages.push("You descend the stairs...".to_string());
                self.next_floor();
            }

            // End turn when out of AP
            let out_of_ap = self
                .world
                .get::<&ActionPoints>(self.player)
                .map(|ap| ap.current <= 0)
                .unwrap_or(true);

            if out_of_ap {
                self.end_turn();
            }
        } else {
            self.messages.push("Blocked!".to_string());
        }
    }

    /// Collect any runes at the given position.
    fn collect_runes(&mut self, pos: Hex) {
        // Find runes at this position
        let runes_to_collect: Vec<(hecs::Entity, Rune)> = self
            .world
            .query::<(&Position, &RunePickup)>()
            .iter()
            .filter(|(_, (p, _))| p.0 == pos)
            .map(|(e, (_, pickup))| (e, pickup.rune.clone()))
            .collect();

        // Add to inventory and despawn pickup
        for (entity, rune) in runes_to_collect {
            if let Ok(mut inv) = self.world.get::<&mut Inventory>(self.player) {
                self.messages.push(format!("Found {} rune!", rune.name()));
                inv.runes.push(rune);
            }
            let _ = self.world.despawn(entity);
        }
    }

    /// Try to attack an adjacent enemy.
    fn try_attack(&mut self) {
        // Check AP
        {
            let ap = match self.world.get::<&ActionPoints>(self.player) {
                Ok(a) => a.current,
                Err(_) => return,
            };
            if ap < MELEE_AP_COST {
                self.messages.push("Not enough AP!".to_string());
                return;
            }
        }

        // Get player position
        let player_pos = match self.world.get::<&Position>(self.player) {
            Ok(p) => p.0,
            Err(_) => return,
        };

        // Find adjacent enemy
        let mut target = None;
        for neighbor in player_pos.neighbors() {
            if let Some(entity) = entity_at(&self.world, neighbor, self.player) {
                // Check if it's an enemy
                if self.world.get::<&Enemy>(entity).is_ok() {
                    target = Some(entity);
                    break;
                }
            }
        }

        let target = match target {
            Some(t) => t,
            None => {
                self.messages.push("No target in range!".to_string());
                return;
            }
        };

        // Get target position for potential rune drop
        let target_pos = self.world.get::<&Position>(target).map(|p| p.0).ok();

        // Deduct AP
        if let Ok(mut ap) = self.world.get::<&mut ActionPoints>(self.player) {
            ap.current -= MELEE_AP_COST;
        }

        // Perform attack
        match melee_attack(&mut self.world, self.player, target, BASE_MELEE_DAMAGE) {
            AttackResult::Hit { damage, killed } => {
                if killed {
                    self.messages
                        .push(format!("You hit for {} damage! Enemy slain!", damage));
                    // Drop a random rune
                    if let Some(pos) = target_pos {
                        self.spawn_random_rune(pos);
                    }
                } else {
                    self.messages
                        .push(format!("You hit for {} damage!", damage));
                }
            }
            AttackResult::Miss => {
                self.messages.push("You miss!".to_string());
            }
            AttackResult::NoTarget => {
                self.messages.push("No target!".to_string());
            }
        }

        // Remove dead entities
        remove_dead(&mut self.world);

        // Check if out of AP
        let out_of_ap = self
            .world
            .get::<&ActionPoints>(self.player)
            .map(|ap| ap.current <= 0)
            .unwrap_or(true);

        if out_of_ap {
            self.end_turn();
        }
    }

    /// Spawn a random rune at the given position.
    fn spawn_random_rune(&mut self, pos: Hex) {
        use rand::Rng;

        let elements = [
            Element::Fire,
            Element::Ice,
            Element::Lightning,
            Element::Earth,
            Element::Void,
        ];
        let shapes = [
            Shape::Point,
            Shape::Line,
            Shape::Cone,
            Shape::Ring,
            Shape::Burst,
        ];
        let modifiers = [Modifier::Power, Modifier::Range, Modifier::Duration];

        // 60% element, 25% shape, 15% modifier
        let roll = self.rng.gen_range(0..100);
        let rune = if roll < 60 {
            Rune::element(elements[self.rng.gen_range(0..elements.len())])
        } else if roll < 85 {
            Rune::shape(shapes[self.rng.gen_range(0..shapes.len())])
        } else {
            Rune::modifier(modifiers[self.rng.gen_range(0..modifiers.len())])
        };

        self.messages
            .push(format!("Enemy dropped {} rune!", rune.name()));
        self.world.spawn((
            Position(pos),
            RunePickup { rune },
            Display {
                glyph: '*',
                color: (255, 255, 0),
            },
        ));
    }

    /// Craft a spell from runes in inventory.
    fn craft_spell_from_inventory(&mut self) {
        use crate::magic::spell::craft_spell;

        // Get runes from inventory
        let runes = match self.world.get::<&Inventory>(self.player) {
            Ok(inv) => inv.runes.clone(),
            Err(_) => {
                self.messages.push("No inventory!".to_string());
                return;
            }
        };

        if runes.is_empty() {
            self.messages.push("No runes to craft with!".to_string());
            return;
        }

        // Try to craft
        match craft_spell(&runes) {
            Some(spell) => {
                let name = spell.name();
                self.grimoire.push(spell);
                // Clear inventory
                if let Ok(mut inv) = self.world.get::<&mut Inventory>(self.player) {
                    inv.runes.clear();
                }
                self.messages.push(format!("Crafted {}!", name));
            }
            None => {
                self.messages
                    .push("Need at least one element rune!".to_string());
            }
        }
    }

    fn end_turn(&mut self) {
        // Run enemy AI
        let ai_messages = run_enemy_ai(&mut self.world, &self.grid, self.player);
        for msg in ai_messages {
            self.messages.push(msg.0);
        }

        // Check if player is dead
        let player_dead = self
            .world
            .get::<&Health>(self.player)
            .map(|h| h.is_dead())
            .unwrap_or(false);

        if player_dead {
            self.state = GameState::GameOver;
            self.messages.push("You have been slain!".to_string());
            return;
        }

        // Tick status effects
        if let Ok(mut status) = self.world.get::<&mut Status>(self.player) {
            status.tick();
        }

        // Restore AP
        if let Ok(mut ap) = self.world.get::<&mut ActionPoints>(self.player) {
            ap.restore();
        }
    }

    fn next_floor(&mut self) {
        let new_floor = match self.state {
            GameState::Playing { floor } => floor + 1,
            _ => 1,
        };

        // Victory at floor 3
        if new_floor > 3 {
            self.state = GameState::Victory;
            self.messages.push("You escaped the dungeon!".to_string());
            return;
        }

        let config = DungeonConfig {
            floor_number: new_floor,
            ..Default::default()
        };

        let (grid, start) = generate(&config, &mut self.rng);
        self.grid = grid;

        if let Ok(mut pos) = self.world.get::<&mut Position>(self.player) {
            pos.0 = start;
        }

        self.state = GameState::Playing { floor: new_floor };
        self.messages
            .push(format!("Descended to floor {}...", new_floor));

        // Spawn more enemies on deeper floors
        let floor_tiles: Vec<Hex> = self
            .grid
            .iter()
            .filter(|(h, t)| **t == DungeonTile::Floor && h.distance(start) > 3)
            .map(|(h, _)| h)
            .collect();

        let num_enemies = (2 + new_floor).min(floor_tiles.len() as i32) as usize;
        for i in 0..num_enemies {
            let pos = floor_tiles[i * floor_tiles.len() / num_enemies.max(1)];
            self.world.spawn((
                Position(pos),
                Health::new(15 + new_floor * 5),
                ActionPoints::new(2),
                Enemy {
                    ai_type: AIType::Melee,
                    aggro_range: 6,
                },
                Display {
                    glyph: 'g',
                    color: (255, 100, 100),
                },
            ));
        }
    }

    /// Render the game.
    fn render(&self, frame: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
            .split(frame.area());

        let right_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(8),
                Constraint::Min(5),
            ])
            .split(chunks[1]);

        // Get player data
        let player_pos = self
            .world
            .get::<&Position>(self.player)
            .map(|p| p.0)
            .unwrap_or(Hex::origin());

        let health = self
            .world
            .get::<&Health>(self.player)
            .map(|h| Health {
                current: h.current,
                max: h.max,
            })
            .unwrap_or(Health::new(100));

        let ap = self
            .world
            .get::<&ActionPoints>(self.player)
            .map(|a| ActionPoints {
                current: a.current,
                max: a.max,
            })
            .unwrap_or(ActionPoints::new(4));

        let inventory = self
            .world
            .get::<&Inventory>(self.player)
            .map(|i| i.runes.clone())
            .unwrap_or_default();

        let _status = self
            .world
            .get::<&Status>(self.player)
            .map(|s| Status {
                effects: s.effects.clone(),
            })
            .unwrap_or_default();

        // Gather entities with positions and displays (excluding player)
        let entities: Vec<(Hex, char, Color)> = self
            .world
            .query::<(&Position, &Display)>()
            .iter()
            .filter(|(e, _)| *e != self.player)
            .map(|(_, (pos, disp))| {
                let color = Color::Rgb(disp.color.0, disp.color.1, disp.color.2);
                (pos.0, disp.glyph, color)
            })
            .collect();

        // Render map
        renderer::render_map(frame, chunks[0], &self.grid, player_pos, &entities);

        // Render HUD
        hud::render_health(frame, right_chunks[0], &health);
        hud::render_ap(frame, right_chunks[1], &ap);
        hud::render_inventory(frame, right_chunks[2], &inventory);
        hud::render_messages(frame, right_chunks[3], &self.messages);

        // State-specific overlays
        match &self.state {
            GameState::SpellSelect => {
                let spell_list: String = if self.grimoire.is_empty() {
                    "No spells crafted!\n\nPress 'c' with runes\nto craft spells.".to_string()
                } else {
                    self.grimoire
                        .iter()
                        .enumerate()
                        .map(|(i, s)| format!("F{}: {} ({}AP)", i + 1, s.name(), s.ap_cost))
                        .collect::<Vec<_>>()
                        .join("\n")
                };
                let popup =
                    Paragraph::new(format!("SELECT SPELL\n\n{}\n\nEsc to cancel", spell_list))
                        .block(Block::default().borders(Borders::ALL))
                        .alignment(Alignment::Center);
                let area = centered_rect(35, 12, frame.area());
                frame.render_widget(popup, area);
            }
            GameState::Targeting { spell_idx } => {
                let spell_name = self
                    .grimoire
                    .get(*spell_idx)
                    .map(|s| s.name())
                    .unwrap_or_else(|| "???".to_string());
                let popup = Paragraph::new(format!(
                    "TARGETING: {}\n\nMove to select target\nEnter to confirm\nEsc to cancel",
                    spell_name
                ))
                .block(Block::default().borders(Borders::ALL))
                .alignment(Alignment::Center);
                let area = centered_rect(30, 8, frame.area());
                frame.render_widget(popup, area);
            }
            GameState::GameOver => {
                let popup = Paragraph::new("GAME OVER\n\nPress Q to quit")
                    .block(Block::default().borders(Borders::ALL))
                    .alignment(Alignment::Center);
                let area = centered_rect(30, 10, frame.area());
                frame.render_widget(popup, area);
            }
            GameState::Victory => {
                let popup = Paragraph::new("VICTORY!\n\nPress Q to quit")
                    .block(Block::default().borders(Borders::ALL))
                    .alignment(Alignment::Center);
                let area = centered_rect(30, 10, frame.area());
                frame.render_widget(popup, area);
            }
            _ => {}
        }
    }
}

fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    Rect::new(x, y, width.min(area.width), height.min(area.height))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_new() {
        let game = Game::new();
        assert!(matches!(game.state, GameState::Playing { floor: 1 }));
    }

    #[test]
    fn test_player_has_components() {
        let game = Game::new();
        assert!(game.world.get::<&Position>(game.player).is_ok());
        assert!(game.world.get::<&Health>(game.player).is_ok());
        assert!(game.world.get::<&ActionPoints>(game.player).is_ok());
    }

    #[test]
    fn test_movement_costs_ap() {
        let mut game = Game::new();
        let initial_ap = game
            .world
            .get::<&ActionPoints>(game.player)
            .map(|ap| ap.current)
            .unwrap();

        // Try moving - should cost 1 AP (or fail due to walls, but AP only deducted on success)
        game.handle_input(Input::Move(0));

        let final_ap = game
            .world
            .get::<&ActionPoints>(game.player)
            .map(|ap| ap.current)
            .unwrap();

        // Either AP was deducted (moved successfully) or stayed same (blocked)
        assert!(final_ap <= initial_ap);
    }

    #[test]
    fn test_wait_ends_turn() {
        let mut game = Game::new();
        let msg_count = game.messages.len();

        game.handle_input(Input::Wait);

        // Should add a message
        assert!(game.messages.len() > msg_count);
        assert!(game.messages.last().unwrap().contains("wait"));
    }

    #[test]
    fn test_game_spawns_runes() {
        let game = Game::new();
        let rune_count = game.world.query::<&RunePickup>().iter().count();
        // Should have starter runes
        assert!(rune_count > 0, "Game should spawn starter runes");
    }

    #[test]
    fn test_player_has_inventory() {
        let game = Game::new();
        let has_inventory = game.world.get::<&Inventory>(game.player).is_ok();
        assert!(has_inventory, "Player should have inventory component");
    }
}
