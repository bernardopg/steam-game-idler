<div align="center">
<img src="./public/logo.png" width="80" alt="app logo">

<h1>Steam Game Idler</h1>

Steam Game Idler (SGI) is a Steam automation tool for farming trading cards, managing achievements, and boosting playtime across all games in your Steam library.

This is a **cross-platform fork** of [`zevnda/steam-game-idler`](https://github.com/zevnda/steam-game-idler) that adds first-class **Linux** support by delegating all Steam operations to the sibling [`steam-utility-multiplataform`](https://github.com/bernardopg/steam-utility-multiplataform) .NET CLI, plus native Linux packaging (`.deb`, `.rpm`, `.AppImage`, AUR).

See how it stacks up against other software, such as [ArchiSteamFarm](https://steamgameidler.com/alternatives/archisteamfarm), [Steam Achievement Manager](https://steamgameidler.com/alternatives/steam-achievement-manager), and [Idle Master](https://steamgameidler.com/alternatives/idle-master).

[![Release][release]](https://github.com/bernardopg/steam-game-idler/releases/latest)
[![Build][build]](https://github.com/bernardopg/steam-game-idler/actions/workflows/release.yml)
[![CI][ci]](https://github.com/bernardopg/steam-game-idler/actions/workflows/ci.yml)
[![License][license-badge]](./LICENSE)

<img src="./public/example.png" width="700" alt="example image"><br />
</div>

# Installation
You must have the **[Steam client](https://store.steampowered.com/about)** installed, running, and signed in to at least one account.

Grab the latest build from the **[releases page](https://github.com/bernardopg/steam-game-idler/releases/latest)**:

### Windows
1. Download the `setup.exe` installer (or the portable `.zip`)
2. Run the installer and follow the steps

### Linux
- **Debian/Ubuntu** — install the `.deb`:
  ```bash
  sudo apt install ./steam-game-idler_*.deb
  ```
- **Fedora/RHEL** — install the `.rpm`:
  ```bash
  sudo dnf install ./steam-game-idler-*.rpm
  ```
- **Any distro** — download the `.AppImage`, mark it executable, and run it:
  ```bash
  chmod +x steam-game-idler_*.AppImage
  ./steam-game-idler_*.AppImage
  ```
- **Arch Linux (AUR)** — install via your AUR helper:
  ```bash
  yay -S steam-game-idler-git
  ```

Or build it yourself — see [Linux Development](#linux-development) below, or the upstream **[build guide](https://steamgameidler.com/docs/get-started/build-it-yourself)**.

# Linux Development
This fork includes a Linux development path that uses the sibling `steam-utility-multiplataform` repository.

From the parent `SGI` workspace:

```bash
./steam-game-idler/scripts/dev-linux.sh
```

The script resolves `SteamUtility.Cli`, clears stale dev/runtime state, and runs Tauri with `next dev --webpack`. Webpack is used for the Tauri dev WebView because Turbopack/HMR was unstable with WebKitGTK during card farming validation.

Card farming on Linux intentionally limits concurrent Steam API idlers to reduce Steam IPC pressure.

# Features
Refer to the **[documentation](https://steamgameidler.com/docs/)** for a detailed guide on each feature

* **[Card Farming](https://steamgameidler.com/docs/features/card-farming)**: Farm trading cards to sell for a profit or use in badge crafting
* **[Achievement Unlocker](https://steamgameidler.com/docs/features/achievement-unlocker)**: Unlock achievements automatically with human-like behavior
* **[Achievement Manager](https://steamgameidler.com/docs/features/achievement-manager)**: Manually unlock or lock any achievement for any game
* **[Inventory Manager](https://steamgameidler.com/docs/features/inventory-manager)**: Easily sell your inventory items on the Steam marketplace
* **[Playtime Booster](https://steamgameidler.com/docs/features/playtime-booster)**: Increase a game's total playtime by idling it manually
* **[Automatic Idler](https://steamgameidler.com/docs/features/auto-idler)**: Automatically idle chosen games when SGI launches
* **[Task Scheduling](https://steamgameidler.com/docs/features/task-scheduling)**: When one feature finishes, automatically start the next
* **[Free Game Alerts](https://steamgameidler.com/docs/features/free-games)**: Get notified when there are free Steam games to claim
* **Fully Open Source**: Rest assured that what you're downloading and running is safe
* **Actively Maintained**: Regular updates with new features and bug fixes

# Supported Languages
Contribute to this project by adding new translations or improving existing ones. Open an issue or PR on **[this fork](https://github.com/bernardopg/steam-game-idler/issues)**, or see the upstream **[translation guide](https://github.com/zevnda/steam-game-idler/discussions/148)**.

| Language             | Flag | Language            | Flag | Language   | Flag |
| -------------------- | ---- | ------------------- | ---- | ---------- | ---- |
| Chinese (Simplified) | 🇨🇳    | Czech               | 🇨🇿    | English    | 🇬🇧    |
| Finnish              | 🇫🇮    | French              | 🇫🇷    | German     | 🇩🇪    |
| Indonesian           | 🇮🇩    | Italian             | 🇮🇹    | Macedonian | 🇲🇰    |
| Polish               | 🇵🇱    | Portuguese (Brazil) | 🇧🇷    | Romanian   | 🇷🇴    |
| Russian              | 🇷🇺    | Spanish             | 🇪🇸    | Turkish    | 🇹🇷    |
| Ukrainian            | 🇺🇦    |                     |      |            |      |

<sup>*Some languages may only have partial support*</sup>

# Credits
This fork builds on the original **[Steam Game Idler](https://github.com/zevnda/steam-game-idler)** by [zevnda](https://github.com/zevnda). All credit for the original application goes to the upstream author.

# License
Released under the **[MIT License](./LICENSE)**.

Copyright © 2024-2026 zevnda (original author)<br />
Copyright © 2025-2026 Bernardo Pinto Gomes (fork maintainer)

[release]: https://img.shields.io/github/v/release/bernardopg/steam-game-idler?style=flat-square&color=%232d6acc&label=Version
[build]: https://img.shields.io/github/actions/workflow/status/bernardopg/steam-game-idler/release.yml?style=flat-square&color=%2313a135&label=Build
[ci]: https://img.shields.io/github/actions/workflow/status/bernardopg/steam-game-idler/ci.yml?style=flat-square&color=%232d6acc&label=CI
[license-badge]: https://img.shields.io/github/license/bernardopg/steam-game-idler?style=flat-square&color=%23a82869&label=License

