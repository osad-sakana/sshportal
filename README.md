# sshportal

RustでSSH接続とSCP転送を簡略化するzshプラグインです。

## 特徴

- **ホスト管理**: SSH接続先をエイリアス名で管理し、追加・削除・接続が可能
- **パス管理**: ローカルおよびリモートパスのショートカットを作成
- **SCP機能強化**: パスとホストのエイリアスを使用したファイルコピー
- **zsh統合**: 全コマンドの自動補完とssh/scpコマンドの補完強化
- **JSON設定**: `~/.config/sshportal/config.json`での人間が読みやすい設定管理

## インストール

```bash
# リポジトリのクローンとインストール
git clone <repository-url>
cd sshportal
./install.sh
```

インストーラーは以下の処理を行います：

1. `cargo build --release`でRustバイナリをビルド
2. バイナリを`~/.local/bin/sshportal`にインストール
3. zshプラグインを`~/.config/zsh/plugins/sshportal/`にインストール
4. `.zshrc`に必要な設定を追加

インストール後、シェルを再起動するか以下を実行してください：

```bash
source ~/.zshrc
```

## 使用方法

### ホスト管理

```bash
# ホストの追加
sshportal add-host prod user@192.168.1.100 -p 22

# ホストの削除
sshportal remove-host prod

# ホストの一覧表示
sshportal list-hosts

# ホストへの接続
sshportal connect prod
```

### パス管理

```bash
# ローカルパスの追加
sshportal add-path docs ~/Documents/projects

# リモートパスの追加
sshportal add-path webroot /var/www/html -r

# パスの削除
sshportal remove-path docs

# パスの一覧表示
sshportal list-paths
```

### ファイル転送

```bash
# エイリアスを使用したコピー
sshportal copy docs prod:webroot

# エイリアスと直接パスの混在使用
sshportal copy ~/myfile.txt prod:/tmp/
sshportal copy docs prod:/var/www/html/
```

### 便利なエイリアス

プラグインは以下のエイリアスを提供します：

- `sp` → `sshportal`
- `spc` → `sshportal connect`
- `spl` → `sshportal list-hosts`
- `spp` → `sshportal list-paths`

## 設定

設定は`~/.config/sshportal/config.json`に保存されます：

```json
{
  "hosts": {
    "prod": {
      "connection": "user@192.168.1.100",
      "port": 22
    }
  },
  "paths": {
    "docs": {
      "path": "~/Documents/projects",
      "is_remote": false
    },
    "webroot": {
      "path": "/var/www/html",
      "is_remote": true
    }
  }
}
```

## 自動補完

プラグインは以下のzsh補完を強化します：

- 全`sshportal`コマンドとオプション
- ホスト名とパス名の補完
- sshportalホストを含む`ssh`コマンドの補完強化
- パスとホストエイリアスを含む`scp`コマンドの補完強化

## 要件

- Rust（最新安定版）
- zshシェル
- macOS（主にテスト済み、その他のUnixシステムでも動作予定）

## 依存関係

- `clap` - コマンドライン引数解析
- `serde`・`serde_json` - JSONシリアライゼーション
- `colored` - カラー端末出力
- `dirs` - ディレクトリパス処理

## 使用例

```bash
# セットアップ
sshportal add-host dev alice@dev.example.com -p 2222
sshportal add-path uploads ~/uploads
sshportal add-path www /var/www -r

# 使用方法
sshportal connect dev                    # dev環境への接続
sshportal copy uploads dev:www          # uploadsをdevのwwwディレクトリにコピー
scp file.txt dev:www/                   # 補完が強化されたscpの使用
```

## エラーハンドリング

- 操作前にホストとパスの存在確認
- カラーコーディングされた明確なエラーメッセージ
- 設定ファイルの自動初期化
- 依存関係不足の適切な処理