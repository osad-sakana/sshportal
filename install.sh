#!/bin/bash

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Directories
BIN_DIR="$HOME/.local/bin"
ZSH_PLUGIN_DIR="$HOME/.config/zsh/plugins/sshportal"
ZSHRC="$HOME/.zshrc"

echo -e "${BLUE}sshportalをインストールしています...${NC}"

# Rust/Cargoが利用可能かチェック
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}エラー: cargoが見つかりません。先にRustをインストールしてください。${NC}"
    echo "https://rustup.rs/ をご覧ください"
    exit 1
fi

# zshが利用可能かチェック
if ! command -v zsh &> /dev/null; then
    echo -e "${RED}エラー: zshが見つかりません。このプラグインはzshが必要です。${NC}"
    exit 1
fi

# バイナリをビルド
echo -e "${YELLOW}sshportalバイナリをビルドしています...${NC}"
cargo build --release

if [ ! -f "target/release/sshportal" ]; then
    echo -e "${RED}エラー: sshportalバイナリのビルドに失敗しました${NC}"
    exit 1
fi

# ディレクトリを作成
echo -e "${YELLOW}ディレクトリを作成しています...${NC}"
mkdir -p "$BIN_DIR"
mkdir -p "$ZSH_PLUGIN_DIR"

# バイナリをインストール
echo -e "${YELLOW}バイナリを $BIN_DIR にインストールしています...${NC}"
cp target/release/sshportal "$BIN_DIR/"
chmod +x "$BIN_DIR/sshportal"

# zshプラグインをインストール
echo -e "${YELLOW}zshプラグインを $ZSH_PLUGIN_DIR にインストールしています...${NC}"
cp zsh/sshportal.plugin.zsh "$ZSH_PLUGIN_DIR/"

# .local/binがPATHに含まれているかチェック
if [[ ":$PATH:" != *":$BIN_DIR:"* ]]; then
    echo -e "${YELLOW}.zshrcに $BIN_DIR をPATHに追加しています...${NC}"
    echo "" >> "$ZSHRC"
    echo "# ~/.local/binをPATHに追加" >> "$ZSHRC"
    echo "export PATH=\"\$HOME/.local/bin:\$PATH\"" >> "$ZSHRC"
fi

# プラグインを.zshrcに追加（まだ存在しない場合）
if ! grep -q "sshportal.plugin.zsh" "$ZSHRC" 2>/dev/null; then
    echo -e "${YELLOW}sshportalプラグインを.zshrcに追加しています...${NC}"
    echo "" >> "$ZSHRC"
    echo "# sshportalプラグイン" >> "$ZSHRC"
    echo "source \"\$HOME/.config/zsh/plugins/sshportal/sshportal.plugin.zsh\"" >> "$ZSHRC"
else
    echo -e "${BLUE}sshportalプラグインは既に.zshrcで設定済みです${NC}"
fi

# 初期設定を作成
echo -e "${YELLOW}設定を初期化しています...${NC}"
"$BIN_DIR/sshportal" list-hosts >/dev/null 2>&1 || true

echo -e "${GREEN}✓ インストールが正常に完了しました！${NC}"
echo ""
echo -e "${BLUE}sshportalを使い始めるには:${NC}"
echo -e "1. シェルを再起動するか次を実行: ${YELLOW}source ~/.zshrc${NC}"
echo -e "2. ホストを追加: ${YELLOW}sshportal add-host myserver user@192.168.1.100${NC}"
echo -e "3. 接続: ${YELLOW}sshportal connect myserver${NC}"
echo ""
echo -e "${BLUE}利用可能なコマンド:${NC}"
echo -e "  sshportal add-host <名前> <user@host> [-p ポート]"
echo -e "  sshportal connect <名前>"
echo -e "  sshportal list-hosts"
echo -e "  sshportal add-path <名前> <パス> [-r]"
echo -e "  sshportal copy <コピー元> <コピー先>"
echo ""
echo -e "${BLUE}エイリアス:${NC}"
echo -e "  sp        - sshportal"
echo -e "  spc       - sshportal connect"
echo -e "  spl       - sshportal list-hosts"
echo -e "  spp       - sshportal list-paths"