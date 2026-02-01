# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Megaquarium is a Rust CLI tool for analyzing and validating aquarium configurations in the Megaquarium video game. It reads game data files from Steam installations and save files to check fish compatibility, calculate minimum viable tank requirements, and validate against complex biological/behavioral rules.

## Build & Development Commands

```bash
# Build (requires Rust nightly for trait_alias feature)
cargo build

# Run tests
cargo test

# Run the CLI
cargo run -- <subcommand>

# Format code
cargo fmt
```

## CLI Commands

- `lookup <search>` - Find species/tanks by partial name match
- `extract <save>` - Extract aquarium from save file as s-expression
- `check <species=count...>` - Validate fish combination (e.g., `check clown_fish=3 anemone=2`)
- `list animals|tanks|food` - List game objects
- `validate` - Validate aquarium from stdin (s-expression format)
- `expand <species=count...>` - Find which existing tanks can support additional fish

Use `-g/--assume-fully-grown` with `check`/`expand` to treat all fish as adult size for predation checks.

## Architecture

**Core modules:**
- `main.rs` - CLI entry point using clap
- `data.rs` - Game data loading from Steam installation paths (Windows/macOS)
- `animal.rs` - Species, Animal, Growth stages, behavioral traits (shoaling, predation, cohabitation)
- `tank.rs` - Tank models, Environment (temperature, salinity, quality, decorations)
- `rules.rs` - Constraint engine that checks violations (temperature conflicts, predation, shoaling requirements, territorial rules)
- `check.rs` - Orchestrates validation, calculates minimum viable tank, prints results
- `aquarium.rs` - Aquarium/Exhibit structures for save file representation
- `sexpr_format.rs`/`sexpr_impl.rs` - S-expression serialization for aquarium I/O

**Data flow:**
1. `read_game_data()` loads species/tanks from game files
2. Commands parse user input into `AnimalRef` collections
3. `minimum_viable_tank()` calculates required Environment
4. `find_violations()` checks all constraints against the exhibit
5. Results printed as s-expressions or debug format

**Key domain concepts:**
- Growth stages affect predation (eggs/fry may be eaten by fish that won't eat adults)
- Cohabitation rules: congeners-only, no-conspecifics, pairs-only, communal groups
- Shoaling: some fish need minimum group sizes
- Tank requirements accumulate (decorations sum, quality takes max)

## Code Style

- Max line width: 140 chars
- Unix newlines
- Rust 2021 edition
