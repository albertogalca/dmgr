# dmgr

macOS app distribution manager — archive, sign, notarize, and distribute.

## Installation

### Homebrew

```bash
brew tap albertogalca/dmgr
brew install dmgr
```

### Shell script

```bash
curl -fsSL https://raw.githubusercontent.com/albertogalca/dmgr/main/install.sh | bash
```

### From source

```bash
cargo install --path .
```

## Prerequisites

Required tools:

- `xcodebuild` (Xcode Command Line Tools)
- `codesign` (macOS)
- `create-dmg` (`npm install -g create-dmg` or `brew install create-dmg`)
- `xcrun` (Xcode Command Line Tools)

Run `dmgr doctor` to verify your setup.

## Commands

### `dmgr doctor`

Check system for required tools and configuration.

```bash
dmgr doctor
```

### `dmgr profile`

Manage signing identity and notarization profiles.

```bash
# List available signing identities
dmgr profile --list

# Save signing identity to config
dmgr profile --name "Developer ID Application: Your Name (TEAM_ID)"

# Save team ID
dmgr profile --team-id TEAM_ID

# Create notarization keychain profile (prompts for Apple ID, app-specific password, team ID)
dmgr profile --create-keychain myprofile
```

### `dmgr archive`

Build, sign, and package app for distribution.

```bash
# Basic usage
dmgr archive --scheme MyApp

# With notarization
dmgr archive --scheme MyApp --notarize

# Custom output directory
dmgr archive --scheme MyApp --output ./dist

# Override signing identity
dmgr archive --scheme MyApp --identity "Developer ID Application: Name (TEAM_ID)"

# Dry run (show commands without executing)
dmgr archive --scheme MyApp --dry-run
```

Options:

- `--scheme <SCHEME>` — Xcode scheme to build (required)
- `--config <CONFIG>` — Build configuration, default: `Release`
- `--output <DIR>` — Output directory for DMG, default: current directory
- `--identity <IDENTITY>` — Signing identity (overrides config)
- `--notarize` — Notarize the DMG after creation
- `--dry-run` — Show commands without executing

## Configuration

Configuration files use TOML format. Project config (`.dmgr.toml`) overrides global config (`~/.config/dmgr/config.toml`).

### Example `.dmgr.toml`

```toml
[signing]
identity = "Developer ID Application: Your Name (TEAM_ID)"
team_id = "TEAM_ID"

[notarization]
keychain_profile = "myprofile"

[dmg]
volume_name = "My App"
window_width = 600
window_height = 400
icon_size = 100
app_icon_x = 150
app_icon_y = 200
applications_x = 450
applications_y = 200
background = "dmg-background.png"
```

## Workflow

1. **Setup** (one-time):

   ```bash
   dmgr doctor                              # Verify tools
   dmgr profile --create-keychain myprofile # Create notarization profile
   ```

2. **Build & distribute**:
   ```bash
   dmgr archive --scheme MyApp --notarize
   ```

This will:

1. Create Xcode archive
2. Export signed .app bundle
3. Create DMG with Applications symlink
4. Sign the DMG
5. Notarize with Apple
6. Staple the notarization ticket

## License

MIT
