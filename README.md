# pathfinder

Debug command resolution and PATH issues. Every dev has had PATH hell - this ends it.

## Why This Exists?

You type `node` and something happens. But which `node`? The one from nvm? Homebrew? That ancient system install? When version mismatches break your build at 2am, you need answers fast.

`pathfinder` shows you exactly which binary will run, why it was chosen, and what alternatives exist in your PATH.

## Features

- Shows which binary will actually execute
- Lists all matches in PATH order  
- Detects version of each match
- Follows symlinks to their source
- Analyzes PATH for issues (duplicates, missing dirs)
- JSON output for scripting
- Cross-platform (Linux, macOS, Windows)

## Quick Start

```bash
# Install
cargo install pathfinder

# Basic usage
pathfinder node

# See all details
pathfinder python --explain

# Check for PATH problems
pathfinder --analyze

# Compare related commands
pathfinder --diff node npm
```

## Usage

```
pathfinder [OPTIONS] [COMMAND]

Arguments:
  [COMMAND]  Command to resolve

Options:
  -j, --json        Output as JSON
  -p, --plain       Output without colors/Unicode
  -a, --analyze     Analyze PATH for issues
  -e, --explain     Explain resolution in plain English
  -d, --diff        Compare multiple commands
  -t, --timeout     Version detection timeout (ms) [default: 2000]
      --no-version  Skip version detection
      --no-color    Disable colors
  -h, --help        Print help
  -V, --version     Print version
```

## License

MIT

---

Katie
