# Hexcaster

A tactical roguelike on hex grids with spell-crafting.

## Features

- **Hex Grid Combat**: Move and fight on a hexagonal grid
- **Rune Collection**: Find magical runes dropped by enemies
- **Spell Crafting**: Combine runes to create custom spells
- **Procedural Dungeons**: Each run generates unique layouts
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

## Roadmap

### v0.2 (Complete)
- [x] Ranged enemy AI (maintain optimal distance, cast spells)
- [x] Patrol enemy AI (wander until player spotted)
- [x] More spell combinations (light, dark, poison elements; wall, chain, nova shapes)
- [x] New modifiers (homing, piercing, vampiric, silent)

### v0.3 (Planned)
- [ ] Boss encounters at floor ends
- [ ] More dungeon floor types

See FEATURE-BACKLOG.md in the clawd repo for detailed acceptance criteria.

## License

MIT
