# Settupper

> **Declarative package manager with a TUI - configure once, run on any machine.**

Settupper is a terminal application (TUI) that reads a `YAML` or `JSON` file with the programs you need, checks what is already installed, and runs install, update, or uninstall with one click - no need to remember distro-specific or operating-system-specific commands.

![Preview](https://github.com/devbaraus/settupper/blob/main/assets/image.png)

---

## Why use it?

- You format your computer often and are tired of reinstalling everything manually
- Your team needs a standardized environment without fragile shell scripts
- You maintain multiple machines (personal Linux, work Mac, Windows VM) and want one config
- You prefer declaring *what you want* instead of remembering *how to install it* on each OS

---

## Features

- **Interactive TUI** with list, details, and real-time log panels
- **Cross-platform**: Ubuntu, Fedora (and derivatives), macOS, Windows
- **Actions**: install, update, uninstall, smart (decides automatically)
- **Smart All**: installs/updates everything at once while respecting app dependencies
- **App dependencies** with topological ordering - if `nvm` depends on `git`, Git is installed first
- **Reboot flag** - if a package requires a reboot, the TUI stops the queue and notifies you
- **Groups** to organize and filter apps by category
- **Multiple selection** with `Space` to operate on several apps at once
- **Split-pane resizing** by dragging with the mouse
- **Dry run** (`--dry-run`) to see what would be executed without running anything
- **Export** a snapshot of the current state as JSON
- **Default config via XDG** - no path required if `~/.config/settupper/packages.yaml` exists

---

## Installation

### Using a script

Linux/macOS:

```bash
curl -fsSL https://raw.githubusercontent.com/devbaraus/settupper/main/install.sh | sh
```

Windows PowerShell:

```powershell
irm https://raw.githubusercontent.com/devbaraus/settupper/main/install.ps1 | iex
```

Windows CMD:

```cmd
powershell -NoProfile -ExecutionPolicy Bypass -Command "irm https://raw.githubusercontent.com/devbaraus/settupper/main/install.ps1 | iex"
```

By default, the scripts fetch the latest release from GitHub, detect the operating system and architecture, download the matching compiled binary, and install it to:

- Linux/macOS: `~/.local/bin/settupper`
- Windows: `%LOCALAPPDATA%\Programs\settupper\bin\settupper.exe`

After that, if the installation directory is in your `PATH`, just run:

```bash
settupper
```

To install a specific version:

```bash
curl -fsSL https://raw.githubusercontent.com/devbaraus/settupper/main/install.sh | SETTUPPER_VERSION=v0.1.5 sh
```

In PowerShell:

```powershell
$env:SETTUPPER_VERSION = "v0.1.5"; irm https://raw.githubusercontent.com/devbaraus/settupper/main/install.ps1 | iex
```

If your terminal cannot find the `settupper` command, add the installation directory to `PATH`. On Linux/macOS:

```bash
export PATH="$HOME/.local/bin:$PATH"
```

### Manual

```bash
# Run directly without installing (recommended for trying it out)
cargo run

# Install release v0.1.5 as a tool
cargo install --git https://github.com/devbaraus/settupper --tag v0.1.5 --bin settupper --locked --force

# Local development
git clone https://github.com/devbaraus/settupper
cd settupper
cargo build --release
./target/release/settupper examples/packages.yaml
```

---

## Usage

```bash
# Open the TUI with your config file
settupper my-packages.yaml

# Use the default config at ~/.config/settupper/packages.yaml
settupper

# Show what would be executed without running anything
settupper --dry-run my-packages.yaml
```

---

## Keys

| Key         | Action                                                |
|-------------|-------------------------------------------------------|
| `Space`     | Select / deselect item                                |
| `Escape`    | Clear selection                                       |
| `i`         | Install selected item(s)                              |
| `u`         | Update selected item(s)                               |
| `d`         | Uninstall selected item(s)                            |
| `a`         | Smart: install or update based on status              |
| `Shift+A`   | Smart All: all visible apps (respects dependencies)   |
| `r`         | Reload status                                         |
| `e`         | Export JSON snapshot                                  |
| `q`         | Quit                                                  |

---

## Configuration File Format

```yaml
version: 1

groups:
  - id: dev-tools
    name: Development Tools
  - id: runtimes
    name: Runtimes

apps:
  - id: git
    name: Git
    group: dev-tools
    description: Version control
    check:
      default:
        - command -v git
      windows:
        - where git
    actions:
      install:
        ubuntu:
          - sudo apt-get install -y git
        fedora:
          - sudo dnf install -y git
        macos:
          - brew install git
        windows:
          - winget install --id Git.Git -e
      update:
        ubuntu:
          - sudo apt-get install --only-upgrade -y git
        fedora:
          - sudo dnf upgrade -y git
        macos:
          - brew upgrade git
        windows:
          - winget upgrade --id Git.Git -e
      uninstall:
        ubuntu:
          - sudo apt-get remove -y git
        fedora:
          - sudo dnf remove -y git
        macos:
          - brew uninstall git
        windows:
          - winget uninstall --id Git.Git -e

  - id: nvm
    name: NVM
    group: runtimes
    description: Node Version Manager
    depends_on:
      - git                   # git will be installed before nvm
    reboot_on:
      install: false
    check:
      default:
        - test -d "$HOME/.nvm"
    actions:
      install:
        default:
          - curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.3/install.sh | bash
```

### Available app fields

| Field         | Required    | Description |
|---------------|-------------|-----------|
| `id`          | **yes**     | Unique identifier (generated from `name` if omitted) |
| `name`        | **yes**     | Name displayed in the TUI |
| `description` | no          | Short description |
| `group`       | no          | Group ID used to filter in the TUI |
| `depends_on`  | no          | List of app IDs that must be installed first |
| `reboot_on`   | no          | Action -> bool map indicating whether a reboot is required |
| `check`       | no          | Commands to check whether the app is installed (by distro or `default`) |
| `actions.install` | no     | Installation commands by distro |
| `actions.update`  | no     | Update commands by distro |
| `actions.uninstall` | no   | Removal commands by distro |

### Supported distro keys

`ubuntu`, `fedora`, `macos`, `windows`, `default` (fallback for any distro)

---

## Stack

| Lib | Usage |
|-----|-----|
| [Ratatui](https://github.com/ratatui-org/ratatui) | Framework TUI |
| [Serde](https://github.com/serde-rs/serde) | YAML config parser |
| [Tokio](https://docs.rs/tokio/latest/tokio/) | Dependency management and execution |

---

## Disclaimer

This project was built entirely through **Vibe Coding** - a practice where the developer describes what they want in natural language and AI (in this case, Anthropic's [Claude](https://claude.ai)) writes all the code.

---

## License

MIT
