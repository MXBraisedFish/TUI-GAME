![LOGO](./README-i18n/image/logo.png)

**[简体中文](./README-i18n/README-zh-cn.md)** | **[日本語](./README-i18n/README-ja-jp.md)**

# This project is

A collection of classic lightweight games playable in the terminal, built with Rust and Lua. It realizes the idea of playing games within a terminal environment, supporting internationalization (multi-language) and cross-platform compatibility.
Windows，Linux，MacOS

> Latest Official Version:
> [![Release](https://img.shields.io/github/v/release/MXBraisedFish/TUI-GAME?maxAge=3600&label=Release&labelColor=cc8400&color=ffa500)](https://github.com/MXBraisedFish/TUI-GAME/releases/latest)

## Table of Contents

- [Implemented Games](#Implemented-Games)
- [Language Support](#Language-Support)
- [Platform Support](#Platform-Support)
- [Installation Guide](#Installation-Guide)
  - [Windows](#Windows)
  - [Linux](#Linux)
  - [MacOS](#MacOS)
- [UI Screenshots](#UI-Screenshots)
- [Support This Project](#Support-This-Project)

## Implemented Games

- 2048
- Blackjack
- Color Memory Game
- Lights Out
- Maze Escape
- Memory Flip
- Minesweeper
- Rock Paper Scissors

## Language Support

- English
- Chinese
- Japanese

## Platform Support

- Windows
- Linux (Bugs pending testing)
- macOS (Bugs pending testing)

## Installation Guide

### Windows

#### - Terminal Script Installation (Recommended)

> Includes all automated services (Pre-compiled, Auto-update, Quick uninstall)

```shell
# Create a new folder
mkdir tui-game

# Enter the folder
cd tui-game

# Pull the installation script
# Mirror Source
curl -L -o windows-tui-game-init.bat https://fastly.jsdelivr.net/gh/MXBraisedFish/TUI-GAME@main/windows-tui-game-init.bat
# Official Source
curl -L -o windows-tui-game-init.bat https://raw.githubusercontent.com/MXBraisedFish/TUI-GAME/main/windows-tui-game-init.bat

# Run the installation script
windows-tui-game-init.bat

```

#### - Download Compiled Version

> No quick uninstaller, no auto-updater

```text
Go to the Releases page:
https://github.com/MXBraisedFish/TUI-GAME/releases/latest
Download the archive: tui-game-windows.zip
Extract: tui-game-windows.zip
Run the file: tui-game.exe

```

#### - Source Code

> No compilation, no quick uninstaller, no auto-updater

```shell
# Create a new folder
mkdir tui-game
# Enter the folder
cd tui-game
# Clone the repository
git clone https://github.com/MXBraisedFish/TUI-GAME.git
# Run/Debug
cargo run
```

### Linux

#### - Terminal Script Installation (Recommended)

> Includes all automated services (Pre-compiled, Auto-update, Quick uninstall)

```shell
# Create a new folder
mkdir tui-game

# Enter the folder
cd tui-game

# Pull the installation script
# Mirror Source
curl -L -o linux-tui-game-init.sh https://fastly.jsdelivr.net/gh/MXBraisedFish/TUI-GAME@main/linux-tui-game-init.sh
# Official Source
curl -L -o linux-tui-game-init.sh https://raw.githubusercontent.com/MXBraisedFish/TUI-GAME/main/linux-tui-game-init.sh

# Run the installation script
sh linux-tui-game-init.sh

```

#### - Download Compiled Version

> No quick uninstaller, no auto-updater

```text
Go to the Releases page:
https://github.com/MXBraisedFish/TUI-GAME/releases/latest
Download the archive: tui-game-linux.tar.gz
Extract: tui-game-linux.tar.gz
Run the file: tui-game bytecode file

```

#### - Source Code

```shell
# Create a new folder
mkdir tui-game
# Enter the folder
cd tui-game
# Clone the repository
git clone https://github.com/MXBraisedFish/TUI-GAME.git
# Run/Debug
cargo run
```

### MacOS

#### - Terminal Script Installation (Recommended)

> Includes all automated services (Pre-compiled, Auto-update, Quick uninstall)

```shell
# Create a new folder
mkdir tui-game

# Enter the folder
cd tui-game

# Pull the installation script
# Mirror Source
curl -L -o macos-tui-game-init.sh https://fastly.jsdelivr.net/gh/MXBraisedFish/TUI-GAME@main/macos-tui-game-init.sh
# Official Source
curl -L -o macos-tui-game-init.sh https://raw.githubusercontent.com/MXBraisedFish/TUI-GAME/main/macos-tui-game-init.sh

# Run the installation script
sh macos-tui-game-init.sh

```

#### - Download Compiled Version

> No quick uninstaller, no auto-updater

```text
Go to the Releases page:
https://github.com/MXBraisedFish/TUI-GAME/releases/latest
Download the archive: tui-game-macos.zip
Extract: tui-game-macos.zip
Run the file: tui-game bytecode file

```

#### - Source Code

```shell
# Create a new folder
mkdir tui-game
# Enter the folder
cd tui-game
# Clone the repository
git clone https://github.com/MXBraisedFish/TUI-GAME.git
# Run/Debug
cargo run
```

## UI Screenshots

### Home and Game List

![Home](./README-i18n/image/main-page.png)
![Game List](./README-i18n/image/game-list.png)

### 2048

![2048](./README-i18n/image/2048.png)

### Blackjack

![Blackjack](./README-i18n/image/blackjack.png)

### Color Memory Game

![Color Memory Game](./README-i18n/image/colormemory.png)

### Lights Out

![Lights Out](./README-i18n/image/lightout.png)

### Maze Escape

![Maze Escape](./README-i18n/image/mazeescape.png)

### Memory Flip

![Memory Flip](./README-i18n/image/memoryflip.png)

### Minesweeper

![inesweeper](./README-i18n/image/minesweeper.png)

### Rock Paper Scissors

![Rock Paper Scissors](./README-i18n/image/rockpaperscissors.png)

## Support This Project

If you like this project, please give my repository a star! This is my motivation to keep updating. If you have better ideas or suggestions, feel free to open an Issue.

The macOS and Linux versions have not been tested as I do not have the relevant hardware. If you find any bugs, please provide feedback. Thank you very much!

GitHub Repo: [MXBraisedFish/TUI-GAME](https://github.com/MXBraisedFish/TUI-GAME)
