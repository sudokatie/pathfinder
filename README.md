# pathfinder

Debug command resolution and PATH issues. Every dev has had PATH hell - this ends it.

## Why This Exists

You type `node` and something happens. But which `node`? The one from nvm? Homebrew? That ancient system install? When version mismatches break your build at 2am, you need answers fast.

`pathfinder` shows you exactly which binary will run, why it was chosen, and what alternatives exist in your PATH.

## Features

- Shows which binary will actually execute
- Lists all matches in PATH order
- Detects version of each match
- Follows symlinks to their source
- Detects broken and circular symlinks
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

## Output Examples

### Human Format (default)

```
RESOLVED: /Users/dev/.nvm/versions/node/v20.10.0/bin/node

All matches in PATH order:

1. /Users/dev/.nvm/versions/node/v20.10.0/bin/node  <- SELECTED
   version: v20.10.0
   symlink: no

2. /usr/local/bin/node
   version: v16.20.2
   symlink: -> ../Cellar/node/16.20.2/bin/node

3. /usr/bin/node
   version: (broken symlink)
   symlink: -> /etc/alternatives/node (DEAD)
```

### JSON Format

```json
{
  "command": "node",
  "resolved": "/Users/dev/.nvm/versions/node/v20.10.0/bin/node",
  "matches": [
    {
      "path": "/Users/dev/.nvm/versions/node/v20.10.0/bin/node",
      "selected": true,
      "version": "v20.10.0",
      "symlink": null,
      "executable": true
    }
  ],
  "path_searched": [
    "/Users/dev/.nvm/versions/node/v20.10.0/bin",
    "/usr/local/bin",
    "/usr/bin"
  ]
}
```

### Explain Mode

```
'node' resolves to /Users/dev/.nvm/versions/node/v20.10.0/bin/node because
/Users/dev/.nvm/versions/node/v20.10.0/bin appears at position 3 in your PATH,
before:
  - /usr/local/bin (position 5)
  - /usr/bin (position 8)
```

### PATH Analysis

```
PATH Analysis:

Directories: 15
Valid: 13
Issues found: 2

1. /usr/local/nonexistent (position 7)
   Warning: Directory does not exist
   Suggestion: Remove from PATH or create the directory

2. /usr/local/bin (position 12)
   Warning: Duplicate entry (first at position 3)
   Suggestion: Remove duplicate entries
```

### Diff Mode

```
Command Comparison:

              node        npm
            ----------  ----------
Status:       Found       Found
Path:       .../node    .../npm
Version:    v20.10.0    10.2.3
Source Dir: .../bin     .../bin

OK: All commands from same directory.
```

## Platform Notes

### Unix (macOS, Linux)

- PATH is parsed as colon-separated entries
- Empty PATH entries represent the current directory
- Executables are detected by file permissions (execute bit)
- Scripts with shebangs (#!) are validated for interpreter existence
- Symlinks are fully resolved, including chained links

### Windows

- PATH is parsed as semicolon-separated entries
- Quoted paths with spaces are handled correctly
- Executables are detected by PATHEXT extensions
- Default extensions: .COM, .EXE, .BAT, .CMD, .VBS, .VBE, .JS, .JSE, .WSF, .WSH, .MSC, .PS1
- App Execution Aliases (WindowsApps) are supported

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Command found successfully |
| 1 | Command not found in PATH |
| 2 | Invalid arguments or usage error |
| 3 | PATH analysis found critical issues |

## Known Limitations

- Cannot detect shell aliases (these are shell-specific)
- Cannot detect shell functions (these are shell-specific)
- Cannot detect shell builtins
- Version detection may fail for commands requiring specific arguments

## License

MIT

---

Katie
