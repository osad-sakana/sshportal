// ホスト管理機能
//
// このモジュールは、SSH接続先ホストの追加、削除、一覧表示、
// および接続を行う機能を提供します。

use crate::config::{Config, Host};
use colored::*;

/// ホストを追加します
/// 
/// 指定された名前、接続文字列、ポート番号、秘密鍵パスで新しいホストを設定に追加します。
/// 同名のホストが既に存在する場合は警告メッセージを表示します。
/// 
/// # 引数
/// * `name` - ホストのエイリアス名
/// * `connection` - SSH接続文字列（例: "user@hostname"）
/// * `port` - SSH接続ポート番号
/// * `key_path` - SSH秘密鍵のパス（オプション）
/// 
/// # 戻り値
/// 成功時は()、失敗時はエラーを返します。
pub fn add_host(name: &str, connection: &str, port: u16, key_path: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    // 現在の設定を読み込み
    let mut config = Config::load()?;
    
    // 同名のホストが既に存在するかチェック
    if config.hosts.contains_key(name) {
        println!("{}: ホスト '{}' は既に存在します", "WARN".yellow(), name);
        return Ok(());
    }

    // 新しいホスト情報を作成
    let host = Host {
        connection: connection.to_string(),
        port,
        key_path: key_path.map(|k| Config::expand_path(k)),
    };

    // 設定にホストを追加し、保存
    config.hosts.insert(name.to_string(), host);
    config.save()?;

    println!("{}: ホスト '{}' を追加しました", "INFO".green(), name);
    Ok(())
}

/// ホストを削除します
/// 
/// 指定された名前のホストを設定から削除します。
/// ホストが存在しない場合はエラーメッセージを表示します。
/// 
/// # 引数
/// * `name` - 削除するホストのエイリアス名
/// 
/// # 戻り値
/// 成功時は()、失敗時はエラーを返します。
pub fn remove_host(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    // 現在の設定を読み込み
    let mut config = Config::load()?;

    // ホストが存在するかチェック
    if !config.hosts.contains_key(name) {
        println!("{}: ホスト '{}' が見つかりません", "ERROR".red(), name);
        return Ok(());
    }

    // ホストを削除し、設定を保存
    config.hosts.remove(name);
    config.save()?;

    println!("{}: ホスト '{}' を削除しました", "INFO".green(), name);
    Ok(())
}

/// 設定されているホストの一覧を表示します
/// 
/// 全ての設定済みホストを名前、接続文字列、ポート番号と共に表示します。
/// ホストが設定されていない場合はその旨を表示します。
/// 
/// # 戻り値
/// 成功時は()、失敗時はエラーを返します。
pub fn list_hosts() -> Result<(), Box<dyn std::error::Error>> {
    // 現在の設定を読み込み
    let config = Config::load()?;

    // ホストが設定されているかチェック
    if config.hosts.is_empty() {
        println!("設定されているホストはありません");
        return Ok(());
    }

    // ホスト一覧を表示
    println!("{}", "設定済みホスト:".bold());
    for (name, host) in &config.hosts {
        let key_info = if let Some(ref key) = host.key_path {
            format!(" (key: {})", key)
        } else {
            String::new()
        };
        println!("  {} -> {}:{}{}", name.cyan(), host.connection, host.port, key_info.dimmed());
    }

    Ok(())
}

/// 指定されたホストにSSH接続します
/// 
/// 設定から指定された名前のホストを検索し、SSH接続を実行します。
/// ホストが見つからない場合はエラーメッセージと利用可能なコマンドを表示します。
/// 
/// # 引数
/// * `name` - 接続するホストのエイリアス名
/// 
/// # 戻り値
/// 成功時は()、失敗時はエラーを返します。
pub fn connect_host(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    // 現在の設定を読み込み
    let config = Config::load()?;

    // 指定されたホストを検索
    let host = match config.hosts.get(name) {
        Some(host) => host,
        None => {
            println!("{}: ホスト '{}' が見つかりません", "ERROR".red(), name);
            println!("利用可能なホストを確認するには 'sshportal list-hosts' を使用してください");
            return Ok(());
        }
    };

    println!("{}: ホスト '{}' に接続中...", "INFO".blue(), name);
    
    // SSH接続コマンドを実行
    let mut cmd = std::process::Command::new("ssh");
    cmd.arg(&host.connection)
        .arg("-p")
        .arg(&host.port.to_string());
    
    // 秘密鍵が指定されている場合は追加
    if let Some(ref key_path) = host.key_path {
        cmd.arg("-i").arg(key_path);
    }
    
    cmd.status()?;

    Ok(())
}