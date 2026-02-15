![LOGO](./image/logo.png)

**[English](../README.md)** | **[中文](./README-zh-cn.md)**

# 本プロジェクトについて

本プロジェクトは Rust と Lua をベースに開発された、ターミナル上で遊べるクラシック軽量ゲーム集です。  
ターミナルでゲームをプレイするというアイデアを実現し、国際化多言語対応およびクロスプラットフォームに対応しています。  
Windows、Linux、MacOS

> 最新正式版：
> [![Release](https://img.shields.io/github/v/release/MXBraisedFish/TUI-GAME?maxAge=3600&label=Release&labelColor=cc8400&color=ffa500)](https://github.com/MXBraisedFish/TUI-GAME/releases/latest)

## 目次

- [実装済みゲーム](#実装済みゲーム)
- [対応言語](#対応言語)
- [対応プラットフォーム](#対応プラットフォーム)
- [その他の特徴](#その他の特徴)
- [インストールガイド](#インストールガイド)
  - [Windows](#Windows)
  - [Linux](#Linux)
  - [MacOS](#MacOS)
- [画面スクリーンショット](#画面スクリーンショット)
- [本プロジェクトを支援](#本プロジェクトを支援)

## 実装済みゲーム

- 2048  
- ブラックジャック  
- カラーメモリーゲーム  
- ライトアウト  
- 迷路脱出  
- メモリーフリップ  
- マインスイーパー  
- じゃんけん  

## 対応言語

- 英語  
- 中国語  
- 日本語  

## 対応プラットフォーム

- Windows  
- Linux（バグ未検証）  
- macOS（バグ未検証）  

## インストールガイド

### Windows

#### - ターミナルスクリプトインストール（推奨）

> すべての自動サービスを含みます（コンパイル済み、自動更新、簡単アンインストール）

```Shell
# 新規フォルダ作成
mkdir tui-game

# フォルダへ移動
cd tui-game

# インストールスクリプト取得
# ミラー
curl -L -o windows-tui-game-init.bat https://fastly.jsdelivr.net/gh/MXBraisedFish/TUI-GAME@main/windows-tui-game-init.bat
# 公式
curl -L -o windows-tui-game-init.bat https://raw.githubusercontent.com/MXBraisedFish/TUI-GAME/main/windows-tui-game-init.bat

# インストールスクリプト実行
windows-tui-game-init.bat
```

#### - コンパイル済み版をダウンロード

> 簡単アンインストーラなし、自動更新なし

```text
Releases ページへ：
https://github.com/MXBraisedFish/TUI-GAME/releases/latest
tui-game-windows.zip をダウンロード
tui-game-windows.zip を解凍
tui-game.exe を実行
```

#### - ソースコード

> コンパイルなし、簡単アンインストーラなし、自動更新なし

```Shell
# 新規フォルダ作成
mkdir tui-game
# フォルダへ移動
cd tui-game
# リポジトリ取得
git clone https://github.com/MXBraisedFish/TUI-GAME.git
# デバッグ実行
cargo run
```

### Linux

#### - ターミナルスクリプトインストール（推奨）

> すべての自動サービスを含みます（コンパイル済み、自動更新、簡単アンインストール）

```Shell
# 新規フォルダ作成
mkdir tui-game

# フォルダへ移動
cd tui-game

# インストールスクリプト取得
# ミラー
curl -L -o linux-tui-game-init.sh https://fastly.jsdelivr.net/gh/MXBraisedFish/TUI-GAME@main/linux-tui-game-init.sh
# 公式
curl -L -o linux-tui-game-init.sh https://raw.githubusercontent.com/MXBraisedFish/TUI-GAME/main/linux-tui-game-init.sh

# インストールスクリプト実行
sh linux-tui-game-init.sh
```

#### - コンパイル済み版をダウンロード

> 簡単アンインストーラなし、自動更新なし

```text
Releases ページへ：
https://github.com/MXBraisedFish/TUI-GAME/releases/latest
tui-game-linux.tar.gz をダウンロード
tui-game-linux.tar.gz を解凍
tui-game バイナリを実行
```

#### - ソースコード

> コンパイルなし、簡単アンインストーラなし、自動更新なし

```Shell
# 新規フォルダ作成
mkdir tui-game
# フォルダへ移動
cd tui-game
# リポジトリ取得
git clone https://github.com/MXBraisedFish/TUI-GAME.git
# デバッグ実行
cargo run
```

### MacOS

#### - ターミナルスクリプトインストール（推奨）

> すべての自動サービスを含みます（コンパイル済み、自動更新、簡単アンインストール）

```Shell
# 新規フォルダ作成
mkdir tui-game

# フォルダへ移動
cd tui-game

# インストールスクリプト取得
# ミラー
curl -L -o macos-tui-game-init.sh https://fastly.jsdelivr.net/gh/MXBraisedFish/TUI-GAME@main/macos-tui-game-init.sh
# 公式
curl -L -o macos-tui-game-init.sh https://raw.githubusercontent.com/MXBraisedFish/TUI-GAME/main/macos-tui-game-init.sh

# インストールスクリプト実行
sh macos-tui-game-init.sh
```

#### - コンパイル済み版をダウンロード

> 簡単アンインストーラなし、自動更新なし

```text
Releases ページへ：
https://github.com/MXBraisedFish/TUI-GAME/releases/latest
tui-game-macos.zip をダウンロード
tui-game-macos.zip を解凍
tui-game バイナリを実行
```

#### - ソースコード

> コンパイルなし、簡単アンインストーラなし、自動更新なし

```Shell
# 新規フォルダ作成
mkdir tui-game
# フォルダへ移動
cd tui-game
# リポジトリ取得
git clone https://github.com/MXBraisedFish/TUI-GAME.git
# デバッグ実行
cargo run
```

## 画面スクリーンショット

### ホームおよびゲーム一覧

![主页](./image/main-page-ja-jp.png)
![游戏列表](./image/game-list-ja-jp.png)

### 2048

![2048](./image/2048-ja-jp.png)

### ブラックジャック

![二十一点](./image/blackjack-ja-jp.png)

### カラーメモリーゲーム

![颜色记忆游戏](./image/colormemory-ja-jp.png)

### ライトアウト

![点灯游戏](./image/lightout-ja-jp.png)

### 迷路脱出

![走迷宫](./image/mazeescape-ja-jp.png)

### メモリーフリップ

![记忆翻牌](./image/memoryflip-ja-jp.png)

### マインスイーパー

![扫雷](./image/minesweeper-ja-jp.png)

### じゃんけん

![石头剪刀布](./image/rockpaperscissors-ja-jp.png)

## 本プロジェクトを支援

このプロジェクトを気に入っていただけた場合は、ぜひリポジトリにスターをお願いします！
それが継続的な更新のモチベーションになります。より良いアイデアや提案があれば、Issue の提出を歓迎します。

macOS と Linux 版は未検証です。該当環境を持っていないため、もしバグを発見した場合はぜひフィードバックをお願いします。誠にありがとうございます！

GitHub Repo: [MXBraisedFish/TUI-GAME](https://github.com/MXBraisedFish/TUI-GAME)