# dmgr

macOS app distribution manager — archive, sign, notarize, and distribute.

## Installation

### Homebrew

```bash
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
- `create-dmg` (`brew install create-dmg`)
- `xcrun` (Xcode Command Line Tools)

Optional tools (for distribution):

- `gh` (`brew install gh`) — GitHub releases
- `aws` (`brew install awscli`) — S3 uploads
- `sign_update` (Sparkle framework) — EdDSA signing for appcast

Run `dmgr doctor` to verify your setup.

## Usage

Run `dmgr` without arguments to launch the interactive menu:

```
     _
  __| |_ __ ___   __ _ _ __
 / _` | '_ ` _ \ / _` | '__|
| (_| | | | | | | (_| | |
 \__,_|_| |_| |_|\__, |_|
                 |___/

macOS app distribution manager
https://github.com/albertogalca/dmgr

> 1. Doctor      - Check system for required tools
  2. Profile     - Manage signing identity and profiles
  3. Archive     - Build, sign, and package app
  4. Distribute  - Upload DMG and generate appcast
```

Use arrow keys to navigate, Enter to select, Ctrl+C to quit. After completing a command, you'll be prompted to run another command without restarting.

### Global Options

- `-q, --quiet` — Suppress logo, colors, and use minimal output (ideal for CI/CD pipelines)

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

### `dmgr distribute`

Distribute a DMG to GitHub releases, S3, and generate Sparkle appcast.

```bash
# Upload to GitHub release
dmgr distribute ./MyApp-1.0.0.dmg --github --github-repo owner/repo

# Upload to S3
dmgr distribute ./MyApp-1.0.0.dmg --s3 --s3-bucket my-bucket --s3-prefix apps/myapp

# Both targets
dmgr distribute ./MyApp-1.0.0.dmg --github --github-repo owner/repo --s3 --s3-bucket my-bucket

# With existing appcast to merge
dmgr distribute ./MyApp-1.0.0.dmg --github --github-repo owner/repo --appcast ./appcast.xml

# Custom changelog location
dmgr distribute ./MyApp-1.0.0.dmg --github --github-repo owner/repo --changelog ./CHANGELOG.md

# Skip changelog (provide download URL manually)
dmgr distribute ./MyApp-1.0.0.dmg --skip-changelog --download-url https://example.com/app.dmg

# Dry run
dmgr distribute ./MyApp-1.0.0.dmg --github --github-repo owner/repo --dry-run
```

Options:

- `--changelog <PATH>` — Path to changelog file, default: `Changelog.md`
- `--appcast <PATH>` — Existing appcast.xml to merge with
- `--output-appcast <PATH>` — Output path for appcast.xml, default: `appcast.xml`
- `--github` — Enable GitHub release
- `--github-repo <OWNER/REPO>` — GitHub repository
- `--s3` — Enable S3 upload
- `--s3-bucket <BUCKET>` — S3 bucket name
- `--s3-prefix <PREFIX>` — S3 key prefix
- `--download-url <URL>` — Override download URL in appcast
- `--skip-changelog` — Skip changelog requirement
- `--dry-run` — Show commands without executing

### `dmgr release`

Build, sign, notarize, and distribute in one command. Combines `archive` and `distribute` for a streamlined release workflow.

```bash
# Release to GitHub
dmgr release --scheme MyApp --github

# Release to S3
dmgr release --scheme MyApp --s3 --s3-bucket my-bucket

# Release to both targets
dmgr release --scheme MyApp --github --s3 --s3-bucket my-bucket

# With custom output directory
dmgr release --scheme MyApp --github -o ./dist

# Dry run
dmgr release --scheme MyApp --github --dry-run
```

Options:

- `--scheme <SCHEME>` — Xcode scheme to build (required)
- `--config <CONFIG>` — Build configuration, default: `Release`
- `-o, --output <DIR>` — Output directory for DMG, default: current directory
- `--identity <IDENTITY>` — Signing identity (overrides config)
- `--github` — Enable GitHub release
- `--github-repo <OWNER/REPO>` — GitHub repository (uses config if not specified)
- `--s3` — Enable S3 upload
- `--s3-bucket <BUCKET>` — S3 bucket name (uses config if not specified)
- `--s3-prefix <PREFIX>` — S3 key prefix
- `--changelog <PATH>` — Path to changelog file
- `--skip-changelog` — Skip changelog requirement
- `--dry-run` — Show commands without executing

## Configuration

Configuration files use TOML format. Project config (`.dmgr.toml`) overrides global config (`~/.config/dmgr/config.toml`).

### Example `.dmgr.toml`

```toml
# dmgr configuration example
# Copy to .dmgr.toml (project) or ~/.config/dmgr/config.toml (global)
# Project config overrides global config

[signing]
# Full signing identity name (run `dmgr profile --list` to see available)
identity = "Developer ID Application: Your Name (TEAM_ID)"
# Team ID (10-character alphanumeric)
team_id = "TEAM_ID"

[notarization]
# Keychain profile for notarytool (create with `dmgr profile --create-keychain <name>`)
keychain_profile = "my-notarization-profile"

[dmg]
# Volume name shown when DMG is mounted (defaults to app name)
volume_name = "My App"
# DMG window dimensions
window_width = 600
window_height = 400
# Icon size in DMG window
icon_size = 100
# App icon position
app_icon_x = 150
app_icon_y = 200
# Applications folder symlink position
applications_x = 450
applications_y = 200
# Background image (relative to project root)
background = "dmg-background.png"

[sparkle]
# EdDSA private key for Sparkle signing
private_key = "~/.sparkle/eddsa_private_key"
# Public URL where appcast.xml will be hosted
appcast_url = "https://example.com/appcast.xml"
# Local output path for generated appcast
appcast_output = "appcast.xml"

[distribution]
# Path to changelog file
changelog = "CHANGELOG.md"

[distribution.github]
# Enable GitHub releases
enabled = true
# Repository in owner/repo format
repo = "owner/repo"
# Tag prefix (e.g., "v" creates tags like v1.0.0)
tag_prefix = "v"

[distribution.s3]
# Enable S3 uploads
enabled = false
# S3 bucket name
bucket = "my-bucket"
# Key prefix within bucket
prefix = "apps/myapp"
# AWS region
region = "us-east-1"
```

## Workflow

1. **Setup** (one-time):

   ```bash
   dmgr doctor                              # Verify tools
   dmgr profile --create-keychain myprofile # Create notarization profile
   gh auth login                            # Authenticate GitHub CLI (optional)
   aws configure                            # Configure AWS CLI (optional)
   ```

2. **Release** (single command):

   ```bash
   dmgr release --scheme MyApp --github
   ```

   This will:
   - Create Xcode archive
   - Export signed .app bundle
   - Create DMG with Applications symlink
   - Sign and notarize the DMG
   - Staple the notarization ticket
   - Extract app version from DMG
   - Parse changelog for release notes
   - Create GitHub release and upload DMG
   - Sign DMG with EdDSA for Sparkle
   - Generate/update appcast.xml

   **Or use separate commands for more control:**

3. **Build & notarize** (step 1):

   ```bash
   dmgr archive --scheme MyApp --notarize
   ```

4. **Distribute** (step 2):

   ```bash
   dmgr distribute ./MyApp-1.0.0.dmg --github --github-repo owner/repo
   ```

### CI/CD Usage

Use the `--quiet` flag for cleaner CI output:

```bash
dmgr --quiet release --scheme MyApp --github
```

This suppresses the logo and colors, using plain text output suitable for build logs.

## Changelog Format

The distribute command parses your changelog to extract release notes. Supported formats:

```markdown
## 1.2.0 (100)

- Added new feature
- Fixed bug

## [1.1.0] - 2024-01-15

- Initial release
```

## License

MIT
