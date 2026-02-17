# Configuration

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
