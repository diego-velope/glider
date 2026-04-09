# Glider

An endless runner / momentum platformer inspired by *Alto's Adventure*, built in Rust + Macroquad, compiled to WASM and deployed on Vercel.

## How to Play

You're automatically skiing down an infinite snowy mountain. One button controls everything:

### TV Remote Controls
| Button | Action |
|--------|--------|
| **D-Pad Up / OK / Select** | Jump (tap) / Backflip (hold in air) |
| **Return / Back** | Pause |

### Keyboard Controls (for desktop testing)
| Key | Action |
|-----|--------|
| **↑ Arrow / Space / Enter** | Jump (tap) / Backflip (hold in air) |
| **Escape / Backspace** | Pause |

### Core Mechanics

- **Auto-run**: Your character moves rightward, continuously accelerating
- **Jump**: Tap to leap over rocks and chasms
- **Backflip**: Hold jump while airborne to rotate — land cleanly for a combo boost
- **Slope physics**: Downhill = speed boost, Uphill = drag
- **Combos**: Land clean backflips to increase your score multiplier
- **Obstacles**: Rocks kill on contact, chasms (gaps) = death

### Scoring

```
score += distance_travelled × combo_multiplier
combo_multiplier += 0.5 per clean backflip landing
combo_multiplier resets to 1.0 on crash or rough landing
```

### Difficulty Progression

The world gets harder as you travel:

| Level | Distance | Changes |
|-------|----------|---------|
| Dawn | 0+ | Gentle slopes, no gaps, few rocks |
| Morning | 500+ | Small gaps, occasional rocks |
| Midday | 1500+ | Bigger gaps, more rocks, faster |
| Dusk | 3000+ | Steep hills, gaps everywhere |
| Night | 6000+ | Max speed, tight gaps, lots of rocks |

## Building & Running Locally

### Prerequisites
- Rust toolchain with `wasm32-unknown-unknown` target
- For local preview: `npx serve` or any static file server

### Build
```bash
./build.sh
```

### Local Preview
```bash
npx serve dist
# Open http://localhost:3000
```

## Tech Stack

| Component | Tool |
|-----------|------|
| Language | Rust 2021 |
| Framework | Macroquad 0.4 |
| WASM Target | `wasm32-unknown-unknown` |
| Deployment | Vercel (static) |
| Assets | Kenney Block Pack, Pixel Skies |

## Credits

- **Game**: Built by [diego-velope](https://github.com/diego-velope)
- **Assets**: [Kenney](https://kenney.nl/assets) block pack, Pixel Skies background
- **Engine**: [Macroquad](https://github.com/not-fl3/macroquad) by Fedor Logachev
