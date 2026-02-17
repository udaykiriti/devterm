# devdash

Terminal-first developer productivity dashboard with **btop-inspired professional UI**.

## Features

### Modern UI Design
- **btop-inspired visual aesthetics** with gradient colors and smooth animations
- Rounded borders and modern iconography (Nerd Fonts)
- Professional color scheme optimized for dark terminals
- Smooth animations and visual feedback
- Glowing accents and pulsing indicators
- High-contrast readable text with proper hierarchy

### Multi-pane TUI Layout
- **Git Status** - Branch info, staged/unstaged/untracked files with color-coded indicators
- **System Metrics** - CPU, Memory, Load with gauges, graphs, and health scoring
- **Open PRs** - GitHub pull requests with inline preview
- **Docker Containers** - Running containers status
- **AWS EC2** - Instance monitoring
- **Custom Plugins** - Extensible command-based widgets

### Navigation (btop-style)
- `Tab` / `Shift+Tab` - Cycle through panes
- Arrow keys or `h/j/k/l` - Navigate by direction
- `1..6` - Jump directly to a pane
- `:` - Command palette (`refresh`, `reload`, `compact`, `focus <pane>`, `quit`)
- `F5`/`r` - Refresh data
- `F10`/`q` - Quit
- Mouse support (click to focus panes)

### List Interaction
- `j/k` or `↑/↓` - Navigate list items
- `Enter` - Open detail modal with metadata
- Context-aware modals show:
  - PR URLs, update times, body snippets
  - Container IDs, images, ports
  - EC2 instance types, availability zones, IPs
  - Process details (PID, command, runtime, CPU, memory)

### Customization
- **Resizable panes**:
  - `Ctrl+←/→` or `+/-` - Resize focused pane width
  - `Ctrl+↑/↓` - Resize top vs bottom rows
- **Responsive layouts**:
  - `auto` - Adapts to terminal size
  - `compact` - Minimal space mode
  - `cockpit` - Full dashboard view
- **Alert thresholds** - Configurable CPU/memory warning levels
- **Color-coded status** - Visual feedback based on system health

### Performance
- Background async updates with `tokio`
- Cached API responses (AWS, GitHub)
- Smart refresh with freshness indicators
- Delta tracking for CPU/memory changes
- Rolling averages and peak hold metrics

### Plugin System
- Runtime command-based plugins
- 30-second timeout protection
- Error handling with visual feedback
- Simple TOML configuration

### Visual Effects
- Animated spinners and progress indicators
- Wave and orbit animations
- Flash effects on data refresh
- Pulsing colors for active states
- Trend micro-graphs with sparklines
- Health badges and status glyphs

## Run

```bash
cargo run
```

## Configuration

Create `devdash.toml` in project root or `~/.config/devdash/config.toml`:

```toml
refresh_seconds = 5
cache_seconds = 120

[alerts]
cpu_warn_pct = 65.0
cpu_crit_pct = 85.0
mem_warn_pct = 70.0
mem_crit_pct = 88.0
stale_warn_secs = 2.5
stale_crit_secs = 5.0

[system_ui]
# auto | compact | cockpit
layout_mode = "auto"

[github]
repo = "owner/repo"
token_env = "GITHUB_TOKEN"

[aws]
region = "us-west-2"
profile = "default"

[[plugins]]
name = "k8s"
command = "kubectl"
args = ["get", "pods", "-A", "--no-headers"]
shell = false

[[plugins]]
name = "custom"
command = "your-script.sh"
args = []
shell = true
```

## Color Scheme

The UI uses a carefully crafted color palette inspired by btop and modern terminals:

- **Background**: Deep space blues (#0D1117, #161B22)
- **Accents**: Electric blue (#58A6FF), Purple (#A37AFF), Magenta (#F269FF)
- **Status**: 
  - Good: Bright green (#3FB950, #56D969)
  - Warning: Vivid yellow (#FFB800, #FFD65B)
  - Critical: Bright red (#FF5555, #FF7878)
- **Text**: High contrast white (#E6EDF3) with dimmed variants

## Data Sources

- **Git**: `git status --porcelain --branch`
- **Docker**: `docker ps`
- **AWS EC2**: `aws ec2 describe-instances` (with cached fallback)
- **PRs**: GitHub API (with `gh` CLI fallback)
- **System**: `sysinfo` crate for cross-platform metrics
- **Plugins**: Custom command outputs

## Recent Improvements

### Bug Fixes (v0.1.1)
- Fixed array bounds checking in animations
- Fixed mutex poisoning recovery in system monitoring
- Added timeout protection for plugin commands
- Fixed type conversion safety in cursor navigation
- Improved channel error handling
- Added configuration validation with clear error messages

### UI Enhancements (v0.2.0)
- Complete btop-inspired visual redesign
- Modern gradient color scheme
- Smooth animations and transitions
- Rounded borders throughout
- Nerd Font icons for better visual hierarchy
- Glowing effects and pulsing indicators
- Enhanced modal dialogs
- Professional footer with Unicode symbols

## Requirements

- Rust 1.70+
- Terminal with true color support
- Nerd Fonts (recommended: JetBrainsMono Nerd Font, FiraCode Nerd Font)
- Optional: `git`, `docker`, `aws`, `gh` CLI tools

## Visual Showcase

The dashboard features:
- **Header**: Brand name, status indicators, focused pane, mode, and timestamp
- **Pane Navigation Bar**: Icon-based quick navigation with visual focus
- **System Pane**: Dual gauges, mini-graphs, health badges, rolling stats
- **List Panes**: Clean item presentation with status icons
- **Footer**: Comprehensive keybind reference with Unicode glyphs
- **Modals**: Centered dialogs with rounded borders and spacing

Each pane degrades gracefully and displays actionable setup errors when tools or auth contexts are missing.

## License

MIT
