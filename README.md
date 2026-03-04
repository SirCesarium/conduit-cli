# conduit-cli

A lightning-fast Minecraft mod manager built in Rust.

---

## Features

- **Fast & Lightweight**: Minimal overhead, native performance.
- **Cross-Platform**: Binaries for Windows, Linux (GNU/MUSL/ARM), and macOS.
- **Dependency Management**: Handles mod dependencies automatically.
- **Developer Friendly**: Clean CLI output and intuitive commands.

---

## 📦 Installation

### Windows

Download the `.msi` installer from the [Releases](https://github.com/SirCesarium/conduit-cli/releases) page. It will automatically add `conduit` to your PATH.

### Linux

We provide multiple options for Linux users:

- **Debian/Ubuntu**: Download the `.deb` package and install it via `sudo dpkg -i conduit.deb`.
- **Fedora/RHEL**: Download the `.rpm` package.
- **Universal**: Download the `musl-amd64` binary for a standalone, dependency-free experience.

### macOS

Download the `macos-arm64` (Apple Silicon) or `macos-intel` binary.
_Note: Since the binary is not signed, you may need to Right Click -> Open it the first time._

---

## Usage

```bash
# Display help and available commands
conduit --help

# Search mods in modrinth and get project slug for installing
conduit search

# Add a new mod to your project
conduit add <mod-name>
conduit add f:<file-path>

# Install all mods from your configuration
conduit install

# Remove a mod
conduit remove <mod-name>

# List installed mods in a tree view
conduit list

```

---

## Development

If you want to build `conduit` from source, ensure you have the Rust toolchain installed.

```bash
git clone https://github.com/SirCesarium/conduit-cli
cd conduit-cli
cargo build --release --features cli

```

---

## License

This project is licensed under the **MIT License**. See the `LICENSE` file for details.
