# gh-slot

🎰 A terminal-based slot machine game written in Rust.

## Description

An interactive slot machine game that runs in your terminal! Features 3 spinning reels with 21 unique emoji symbols, 7 paylines for win detection, and smooth animations.

## Features

- **3 Spinning Reels**: Each reel displays 3 symbols with smooth rotation animation
- **21 Unique Symbols**: Beautiful emoji symbols including fruits, gems, and special icons
- **7 Paylines**: Multiple ways to win with horizontal, diagonal, and special line patterns
- **Keyboard Controls**: Intuitive controls for spinning and stopping reels
- **Real-time Animation**: Smooth spinning with 0.74-second rotation cycles
- **Terminal UI**: Clean, colorful display that works in any terminal

## Controls

- **Space**: Start all reels spinning
- **← (Left Arrow)**: Stop the left reel
- **↓ (Down Arrow)**: Stop the middle reel  
- **→ (Right Arrow)**: Stop the right reel
- **ESC**: Exit the game

## Installation

```bash
gh extension install kouji-sasaya/gh-slot
```

## Usage

```bash
gh slot
```

## Building from Source

```bash
git clone https://github.com/kouji-sasaya/gh-slot.git
cd gh-slot
cargo build --release
./target/release/gh-slot
```

## Game Rules

1. Press **Space** to start all reels spinning
2. Use arrow keys to stop each reel individually (left to right recommended)
3. Win by matching 3 identical symbols on any of the 7 paylines:
   - Top row (symbols 1-1-1)
   - Middle row (symbols 2-2-2) 
   - Bottom row (symbols 3-3-3)
   - Diagonal down (symbols 1-2-3)
   - Diagonal up (symbols 3-2-1)
   - V-shape (symbols 1-2-1)
   - Mountain (symbols 3-2-3)

## Version

Current version: **v1.0.0**

## Development

### Prerequisites

- Rust (latest stable version)
- Cargo

### Building

To build the project:

```bash
cargo build
```

To run the program:

```bash
cargo run
```

To build for distribution:

```bash
./script/build.sh
```

This will create a binary in the `dist/` directory for your platform.

## Project Structure

```
gh-slot/
├── src/
│   └── main.rs          # Main application code
├── script/
│   └── build.sh         # Build script for GitHub CLI extension
├── Cargo.toml           # Rust project configuration
└── README.md            # This file
```