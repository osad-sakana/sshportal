// 設定ファイルの管理機能
//
// このモジュールは、sshportalの設定ファイル（JSON形式）の
// 読み込み、保存、および設定データ構造の管理を行います。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// SSH接続ホストの情報を保持する構造体
/// 
/// ホスト名、ユーザー名、ポート番号、秘密鍵パスを含む接続情報を管理します。
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Host {
    /// SSH接続文字列（例: "user@hostname"）
    pub connection: String,
    /// SSH接続ポート番号（デフォルト: 22）
    pub port: u16,
    /// SSH秘密鍵のパス（オプション）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_path: Option<String>,
}

/// パス情報を保持する構造体
/// 
/// ローカルまたはリモートパスの情報を管理し、
/// ファイル転送時のエイリアスとして使用されます。
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Path {
    /// パスの文字列表現
    pub path: String,
    /// リモートパスかどうかのフラグ（true: リモート、false: ローカル）
    pub is_remote: bool,
}

/// sshportalの設定全体を管理する構造体
/// 
/// ホスト情報とパス情報のハッシュマップを含み、
/// JSON形式でシリアライズ/デシリアライズされます。
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    /// ホスト名をキーとするホスト情報のマップ
    pub hosts: HashMap<String, Host>,
    /// パス名をキーとするパス情報のマップ
    pub paths: HashMap<String, Path>,
}

/// デフォルト設定の実装
impl Default for Config {
    /// 空の設定を作成します
    /// 
    /// ホストとパスのハッシュマップは初期化時は空になります。
    fn default() -> Self {
        Config {
            hosts: HashMap::new(),
            paths: HashMap::new(),
        }
    }
}

impl Config {
    /// 設定ディレクトリのパスを取得します
    /// 
    /// ~/.config/sshportal/ ディレクトリのPathBufを返します。
    /// ホームディレクトリが見つからない場合はパニックします。
    pub fn config_dir() -> PathBuf {
        dirs::home_dir()
            .expect("ホームディレクトリが見つかりません")
            .join(".config")
            .join("sshportal")
    }

    /// 設定ファイルのパスを取得します
    /// 
    /// ~/.config/sshportal/config.json のPathBufを返します。
    pub fn config_file() -> PathBuf {
        Self::config_dir().join("config.json")
    }

    /// 設定ファイルを読み込みます
    /// 
    /// 設定ファイルが存在しない場合は、新しいディレクトリとデフォルト設定を作成します。
    /// 既存のファイルが存在する場合は、その内容を解析して設定を読み込みます。
    /// 
    /// # 戻り値
    /// 成功時はConfig構造体、失敗時はエラーを返します。
    pub fn load() -> Result<Config, Box<dyn std::error::Error>> {
        let config_file = Self::config_file();
        
        // 設定ファイルが存在しない場合の処理
        if !config_file.exists() {
            let config_dir = Self::config_dir();
            // 設定ディレクトリを作成
            fs::create_dir_all(&config_dir)?;
            // デフォルト設定を作成・保存
            let default_config = Config::default();
            default_config.save()?;
            return Ok(default_config);
        }

        // 設定ファイルを読み込み、JSONとして解析
        let content = fs::read_to_string(config_file)?;
        let config: Config = serde_json::from_str(&content)?;
        Ok(config)
    }

    /// 設定をファイルに保存します
    /// 
    /// 現在の設定をJSON形式で設定ファイルに書き込みます。
    /// 設定ディレクトリが存在しない場合は作成します。
    /// 
    /// # 戻り値
    /// 成功時は()、失敗時はエラーを返します。
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config_dir = Self::config_dir();
        // 設定ディレクトリを作成（既に存在する場合は何もしない）
        fs::create_dir_all(&config_dir)?;
        
        let config_file = Self::config_file();
        // 設定を整形されたJSON形式でシリアライズ
        let content = serde_json::to_string_pretty(self)?;
        // ファイルに書き込み
        fs::write(config_file, content)?;
        Ok(())
    }

    /// パス文字列を展開します
    /// 
    /// チルダ（~）で始まるパスをホームディレクトリの絶対パスに展開します。
    /// その他のパスはそのまま返します。
    /// 
    /// # 引数
    /// * `path` - 展開するパス文字列
    /// 
    /// # 戻り値
    /// 展開されたパス文字列
    pub fn expand_path(path: &str) -> String {
        if path.starts_with("~/") {
            // ~/で始まる場合、ホームディレクトリに展開
            dirs::home_dir()
                .expect("ホームディレクトリが見つかりません")
                .join(&path[2..])
                .to_string_lossy()
                .to_string()
        } else {
            // その他の場合はそのまま返す
            path.to_string()
        }
    }
}