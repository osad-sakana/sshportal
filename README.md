# sshportal

RustでSSH接続とSCP転送を簡略化するzshプラグインです。

## 特徴

- **インタラクティブ設定**: デフォルトで対話形式。引数を覚える必要なし
- **ホスト管理**: SSH接続先をエイリアス名で管理、SSH秘密鍵認証サポート
- **柔軟なパス管理**: ローカルパスとホスト固有パスを分離管理
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
# ホストの追加（デフォルトでインタラクティブ）
sshportal add-host
# → ホスト名、接続文字列、ポート、秘密鍵パス（オプション）を順次入力

# ホストの削除
sshportal remove-host prod

# ホストの一覧表示
sshportal list-hosts

# ホストへの接続
sshportal connect prod
```

### パス管理

```bash
# パスの追加（デフォルトでインタラクティブ）
sshportal add-paths
# → 1) ローカルパス または 2) ホスト固有パス を選択
# → 選択に応じてパス名と実際のパスを入力

# パスの削除
sshportal remove-path docs

# パスの一覧表示
sshportal list-paths
```

### ファイル転送

```bash
# ローカルパス → ホスト別リモートパス
sshportal copy downloads prod:webroot

# ローカルパス → 直接パス
sshportal copy downloads prod:/var/www/html/

# 混在使用
sshportal copy ~/myfile.txt prod:webroot
sshportal copy downloads staging:webroot  # 同じパス名でも異なるホストで異なる実パス
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
      "connection": "user@prod.example.com",
      "port": 22
    },
    "staging": {
      "connection": "admin@staging.example.com",
      "port": 2222,
      "key_path": "~/.ssh/id_rsa_staging"
    }
  },
  "local_paths": {
    "downloads": "~/Downloads",
    "projects": "~/projects"
  },
  "host_paths": {
    "prod": {
      "configs": "/etc/nginx",
      "webroot": "/var/www/html"
    },
    "staging": {
      "api": "/opt/api"
    }
  }
}
```

### 設定の説明

- **hosts**: SSH接続先の設定。秘密鍵認証が必要な場合は`key_path`を指定
- **local_paths**: ローカルマシンのパスエイリアス
- **host_paths**: 各ホスト固有のパスエイリアス

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
# セットアップ（全て対話形式）
sshportal add-host
# → ホスト名: dev
# → 接続文字列: alice@dev.example.com
# → ポート: 2222
# → SSH秘密鍵パス（オプション）: ~/.ssh/id_rsa_dev

sshportal add-paths
# → パスの種類を選択: 1) ローカル 2) ホスト固有
# → 1) ローカルパス選択時
#    パス名: uploads
#    実際のパス: ~/uploads
# → 2) ホスト固有パス選択時
#    ホスト名: dev
#    パス名: www
#    実際のパス: /var/www

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