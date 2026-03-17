# Hexcaster

A tactical roguelike on hex grids with spell-crafting.

## Features

- **Hex Grid Combat**: Move and fight on a hexagonal grid
- **Rune Collection**: Find magical runes dropped by enemies
- **Spell Crafting**: Combine runes to create custom spells
- **Procedural Dungeons**: Each run generates unique layouts
- **Unlockable Content**: Earn new runes through achievements
- **Terminal UI**: Beautiful TUI powered by ratatui

## Installation

```bash
cargo install hexcaster
```

Or build from source:

```bash
git clone https://github.com/sudokatie/hexcaster
cd hexcaster
cargo build --release
```

## Controls

| Key | Action |
|-----|--------|
| `q/w/e/a/z/x` | Move (hex directions) |
| `Space` | Melee attack |
| `r` | Craft spell from runes |
| `c` | Cast spell (opens spell menu) |
| `F1-F3` | Select spell |
| `.` | Wait/end turn |
| `Esc` | Cancel |
| `Q` | Quit |

## Rune Types

**Elements** (what the spell does):
- Fire, Ice, Lightning, Earth, Void
- Light, Dark, Poison (new in v0.2)

**Shapes** (area of effect):
- Point, Line, Cone, Ring, Burst
- Wall, Chain, Nova (new in v0.2)

**Modifiers** (enhancements):
- Power, Range, Duration
- Homing, Piercing, Vampiric, Silent (new in v0.2)

## How to Play

1. Move around the dungeon using hex movement keys
2. Collect runes (*) by walking over them
3. Press `r` to craft a spell from your runes
4. Press `c` then `F1-F3` to cast your spells
5. Reach the stairs (>) to descend
6. Survive 3 floors to win!

## Unlockable Content

Not all runes are available from the start. Earn achievements to unlock advanced runes:

**Elements** (4 locked):
| Rune | Unlock Condition |
|------|------------------|
| Light | Win your first game |
| Dark | Defeat a boss |
| Void | Reach floor 3 |
| Poison | Kill 50 enemies total |

**Shapes** (3 locked):
| Rune | Unlock Condition |
|------|------------------|
| Wall | Reach floor 2 |
| Chain | Use all 6 base elements in one run |
| Nova | Have 5 spells at once |

**Modifiers** (4 locked):
| Rune | Unlock Condition |
|------|------------------|
| Homing | Craft your first spell |
| Piercing | Complete a floor without damage |
| Vampiric | Win with less than 10 HP |
| Silent | Complete a floor without killing |

The base elements (Fire, Ice, Lightning, Earth), shapes (Point, Line, Cone, Ring, Burst), and modifiers (Power, Range, Duration, Split, Echo) are always available.

## Roadmap

### v0.2 (Complete)
- [x] Ranged enemy AI (maintain optimal distance, cast spells)
- [x] Patrol enemy AI (wander until player spotted)
- [x] More spell combinations (light, dark, poison elements; wall, chain, nova shapes)
- [x] New modifiers (homing, piercing, vampiric, silent)

### v0.3 (Complete)
- [x] Boss encounters at floor ends
- [x] Daily challenges with leaderboards
- [x] Achievement system (22 achievements)
- [x] Unlockable content (11 runes tied to achievements)
- [x] Online leaderboards (optional)

## Online Features

The game supports optional online leaderboards. Enable them by building with the `online` feature:

```bash
cargo build --release --features online
```

Configure your player name in `~/.config/hexcaster/config.yaml`:

```yaml
online:
  enabled: true
  player_name: "YourName"
  server_url: "https://hexcaster.blackabee.com/api"
```

When enabled, your daily challenge scores are submitted to the global leaderboard. You can compete with other players for the best daily run.

## License

MIT
