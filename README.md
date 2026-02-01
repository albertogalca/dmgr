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

Use arrow keys to navigate, Enter to select, Ctrl+C to quit.

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

## Configuration

Configuration files use TOML format. Project config (`.dmgr.toml`) overrides global config (`~/.config/dmgr/config.toml`).

See [`.dmgr.example.toml`](.dmgr.example.toml) for a complete example with all options.

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

[sparkle]
private_key = "~/.sparkle/eddsa_private_key"
appcast_url = "https://example.com/appcast.xml"
appcast_output = "appcast.xml"

[distribution]
changelog = "Changelog.md"

[distribution.github]
enabled = true
repo = "owner/repo"
tag_prefix = "v"

[distribution.s3]
enabled = false
bucket = "my-bucket"
prefix = "apps/myapp"
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

2. **Build & notarize**:

   ```bash
   dmgr archive --scheme MyApp --notarize
   ```

   This will:
   - Create Xcode archive
   - Export signed .app bundle
   - Create DMG with Applications symlink
   - Sign the DMG
   - Notarize with Apple
   - Staple the notarization ticket

3. **Distribute**:

   ```bash
   dmgr distribute ./MyApp-1.0.0.dmg --github --github-repo owner/repo
   ```

   This will:
   - Extract app version from DMG
   - Parse changelog for release notes
   - Create GitHub release and upload DMG
   - Sign DMG with EdDSA for Sparkle
   - Generate/update appcast.xml

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
