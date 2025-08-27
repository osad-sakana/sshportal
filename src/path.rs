// パス管理とファイル転送機能
//
// このモジュールは、ローカルおよびリモートパスのエイリアス管理と
// SCPを使用したファイル転送機能を提供します。

use crate::config::{Config, Path};
use colored::*;
use std::collections::HashMap;
use std::io::{self, Write};

/// パスエイリアスを追加します
/// 
/// 指定された名前でローカルまたはリモートパスのエイリアスを作成します。
/// 同名のパスが既に存在する場合は警告メッセージを表示します。
/// 
/// # 引数
/// * `name` - パスのエイリアス名
/// * `path` - パスの文字列
/// * `is_remote` - リモートパスかどうかのフラグ
/// 
/// # 戻り値
/// 成功時は()、失敗時はエラーを返します。
pub fn add_path(name: &str, path: &str, is_remote: bool) -> Result<(), Box<dyn std::error::Error>> {
    // 現在の設定を読み込み
    let mut config = Config::load()?;

    // 旧バージョンのパス形式を使用（互換性のため）
    let old_paths = config.paths.get_or_insert_with(HashMap::new);

    // 同名のパスが既に存在するかチェック
    if old_paths.contains_key(name) {
        println!("{}: パス '{}' は既に存在します", "WARN".yellow(), name);
        return Ok(());
    }

    // 新しいパス情報を作成
    let path_entry = Path {
        path: path.to_string(),
        is_remote,
    };

    // 設定にパスを追加し、保存
    old_paths.insert(name.to_string(), path_entry);
    config.save()?;

    let path_type = if is_remote { "リモート" } else { "ローカル" };
    println!("{}: {} パス '{}' を追加しました", "INFO".green(), path_type, name);
    Ok(())
}

/// パスエイリアスを削除します
/// 
/// 指定された名前のパスエイリアスを設定から削除します。
/// パスが存在しない場合はエラーメッセージを表示します。
/// 
/// # 引数
/// * `name` - 削除するパスのエイリアス名
/// 
/// # 戻り値
/// 成功時は()、失敗時はエラーを返します。
pub fn remove_path(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    // 現在の設定を読み込み
    let mut config = Config::load()?;

    // 旧バージョンのパス形式を使用（互換性のため）
    let old_paths = config.paths.get_or_insert_with(HashMap::new);

    // パスが存在するかチェック
    if !old_paths.contains_key(name) {
        println!("{}: パス '{}' が見つかりません", "ERROR".red(), name);
        return Ok(());
    }

    // パスを削除し、設定を保存
    old_paths.remove(name);
    config.save()?;

    println!("{}: パス '{}' を削除しました", "INFO".green(), name);
    Ok(())
}


/// SCPを使用してファイルをコピーします
/// 
/// パスエイリアスとホストエイリアスを解決し、SCPコマンドを実行します。
/// ローカル⇔リモート、リモート⇔ローカル、リモート⇔リモートのコピーに対応します。
/// 
/// # 引数
/// * `src` - コピー元の指定（パスエイリアスまたは実際のパス）
/// * `dst` - コピー先の指定（パスエイリアスまたは実際のパス）
/// 
/// # 戻り値
/// 成功時は()、失敗時はエラーを返します。
pub fn copy_files(src: &str, dst: &str) -> Result<(), Box<dyn std::error::Error>> {
    // 現在の設定を読み込み
    let config = Config::load()?;

    // コピー元とコピー先の詳細を解析
    let (src_path, src_host) = parse_path_spec(src, &config)?;
    let (dst_path, dst_host) = parse_path_spec(dst, &config)?;

    println!("{}: {} から {} にコピー中...", "INFO".blue(), src, dst);

    // SCPコマンドを構築
    let mut cmd = std::process::Command::new("scp");
    cmd.arg("-r"); // 再帰的コピーのオプション

    // コピー元がローカルかどうかを事前に判定
    let src_is_local = src_host.is_none();

    // コピー元の設定
    if let Some(ref host) = src_host {
        // リモートホストからのコピーの場合
        if let Some(host_config) = config.hosts.get(host) {
            // エイリアスホストの場合：設定からポート番号と接続情報を取得
            cmd.arg("-P").arg(host_config.port.to_string());
            // 秘密鍵が指定されている場合は追加
            if let Some(ref key_path) = host_config.key_path {
                cmd.arg("-i").arg(key_path);
            }
            cmd.arg(format!("{}:{}", host_config.connection, src_path));
        } else {
            // 直接指定ホストの場合：デフォルトポート22を使用
            cmd.arg("-P").arg("22");
            cmd.arg(format!("{}:{}", host, src_path));
        }
    } else {
        // ローカルファイルからのコピーの場合
        let expanded_src = Config::expand_path(&src_path);
        cmd.arg(expanded_src);
    }

    // コピー先の設定
    if let Some(ref host) = dst_host {
        // リモートホストへのコピーの場合
        if let Some(host_config) = config.hosts.get(host) {
            // エイリアスホストの場合：設定からポート番号と接続情報を取得
            // コピー元がローカルの場合のみポート番号を指定
            if src_is_local {
                cmd.arg("-P").arg(host_config.port.to_string());
                // 秘密鍵が指定されている場合は追加
                if let Some(ref key_path) = host_config.key_path {
                    cmd.arg("-i").arg(key_path);
                }
            }
            cmd.arg(format!("{}:{}", host_config.connection, dst_path));
        } else {
            // 直接指定ホストの場合：デフォルトポート22を使用
            // コピー元がローカルの場合のみポート番号を指定
            if src_is_local {
                cmd.arg("-P").arg("22");
            }
            cmd.arg(format!("{}:{}", host, dst_path));
        }
    } else {
        // ローカルファイルへのコピーの場合
        let expanded_dst = Config::expand_path(&dst_path);
        cmd.arg(expanded_dst);
    }

    // SCPコマンドを実行
    let status = cmd.status()?;

    // 結果の表示
    if status.success() {
        println!("{}: コピーが正常に完了しました", "INFO".green());
    } else {
        println!("{}: コピーに失敗しました", "ERROR".red());
    }

    Ok(())
}

/// パス指定文字列を解析します
/// 
/// "host:path"形式の文字列を解析し、ホスト名とパスに分離します。
/// パスエイリアスとホストエイリアスの解決も行います。
/// 
/// # 引数
/// * `spec` - 解析するパス指定文字列
/// * `config` - 現在の設定
/// 
/// # 戻り値
/// (パス文字列, オプションのホスト名)のタプル、またはエラー
fn parse_path_spec(spec: &str, config: &Config) -> Result<(String, Option<String>), Box<dyn std::error::Error>> {
    // コロンが含まれる場合はリモートパスとして処理
    if spec.contains(':') {
        let parts: Vec<&str> = spec.splitn(2, ':').collect();
        let host = parts[0].to_string();
        let path = parts[1].to_string();

        // ホスト名が設定に存在するかチェック
        if config.hosts.contains_key(&host) {
            // パス部分がパスエイリアスかチェック（旧形式との互換性）
            if let Some(ref old_paths) = config.paths {
                if old_paths.contains_key(&path) {
                    let path_entry = &old_paths[&path];
                    // リモートパスでない場合はエラー
                    if !path_entry.is_remote {
                        return Err(format!("パス '{}' はリモートパスではありません", path).into());
                    }
                    return Ok((path_entry.path.clone(), Some(host)));
                }
            }
            // 直接パスの場合
            return Ok((path, Some(host)));
        }

        // ケース2: ホスト名が直接のSSH接続文字列の可能性（user@hostname形式）
        if host.contains('@') || is_valid_hostname(&host) {
            // 直接SSH接続文字列として扱う
            return Ok((path, Some(host)));
        }

        // ケース3: 不明なホスト形式
        return Err(format!("ホスト '{}' が見つからず、有効なSSH接続文字列でもありません", host).into());
    }

    // コロンが含まれない場合はローカルパスまたはパスエイリアス（旧形式との互換性）
    if let Some(ref old_paths) = config.paths {
        if old_paths.contains_key(spec) {
            let path_entry = &old_paths[spec];
            // リモートパスの場合はホスト指定が必要
            if path_entry.is_remote {
                return Err(format!("パス '{}' はリモートパスですが、ホストが指定されていません", spec).into());
            }
            return Ok((path_entry.path.clone(), None));
        }
    }

    // パスエイリアスでない場合は文字列をそのまま返す
    Ok((spec.to_string(), None))
}

/// ホスト名が有効かどうかをチェックします
/// 
/// 基本的なホスト名の形式をチェックします（RFC準拠ではない簡易版）
/// 
/// # 引数
/// * `hostname` - チェックするホスト名
/// 
/// # 戻り値
/// 有効なホスト名の場合はtrue
fn is_valid_hostname(hostname: &str) -> bool {
    if hostname.is_empty() || hostname.len() > 253 {
        return false;
    }

    // 基本的なホスト名の規則をチェック
    // - 英数字とハイフン、ピリオドのみ
    // - ハイフンで始まらない、終わらない
    // - 連続するピリオドがない
    let chars: Vec<char> = hostname.chars().collect();
    
    for (i, &ch) in chars.iter().enumerate() {
        match ch {
            'a'..='z' | 'A'..='Z' | '0'..='9' => continue,
            '-' => {
                if i == 0 || i == chars.len() - 1 {
                    return false;
                }
            }
            '.' => {
                if i == 0 || i == chars.len() - 1 {
                    return false;
                }
                if i > 0 && chars[i - 1] == '.' {
                    return false;
                }
            }
            _ => return false,
        }
    }
    
    // IPアドレスの場合も有効とする
    if is_valid_ip_address(hostname) {
        return true;
    }
    
    true
}

/// IPアドレス（IPv4）が有効かどうかをチェックします
/// 
/// # 引数
/// * `ip` - チェックするIPアドレス文字列
/// 
/// # 戻り値
/// 有効なIPv4アドレスの場合はtrue
fn is_valid_ip_address(ip: &str) -> bool {
    let parts: Vec<&str> = ip.split('.').collect();
    if parts.len() != 4 {
        return false;
    }
    
    for part in parts {
        if let Ok(_num) = part.parse::<u8>() {
            if part.len() > 1 && part.starts_with('0') {
                return false; // 先頭ゼロは無効
            }
        } else {
            return false;
        }
    }
    
    true
}

/// ローカルパスエイリアスを追加します
pub fn add_local_path(name: &str, path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut config = Config::load()?;
    
    if config.local_paths.contains_key(name) {
        println!("{}: ローカルパス '{}' は既に存在します", "WARN".yellow(), name);
        return Ok(());
    }
    
    config.local_paths.insert(name.to_string(), path.to_string());
    config.save()?;
    
    println!("{}: ローカルパス '{}' を追加しました", "INFO".green(), name);
    Ok(())
}

/// ホスト別リモートパスエイリアスを追加します
pub fn add_host_path(host_name: &str, path_name: &str, path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut config = Config::load()?;
    
    if !config.hosts.contains_key(host_name) {
        println!("{}: ホスト '{}' が存在しません", "ERROR".red(), host_name);
        return Ok(());
    }
    
    let host_paths = config.host_paths.entry(host_name.to_string()).or_insert_with(HashMap::new);
    
    if host_paths.contains_key(path_name) {
        println!("{}: ホスト '{}' のパス '{}' は既に存在します", "WARN".yellow(), host_name, path_name);
        return Ok(());
    }
    
    host_paths.insert(path_name.to_string(), path.to_string());
    config.save()?;
    
    println!("{}: ホスト '{}' にパス '{}' を追加しました", "INFO".green(), host_name, path_name);
    Ok(())
}

/// 新しいパス一覧表示機能
pub fn list_paths_new() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load()?;
    
    // ローカルパス表示
    if !config.local_paths.is_empty() {
        println!("{}", "ローカルパス:".bold().green());
        for (name, path) in &config.local_paths {
            let expanded_path = Config::expand_path(path);
            println!("  {} -> {}", name.cyan(), expanded_path);
        }
        println!();
    }
    
    // ホスト別リモートパス表示
    if !config.host_paths.is_empty() {
        println!("{}", "リモートパス（ホスト別）:".bold().yellow());
        for (host_name, paths) in &config.host_paths {
            println!("  {}:", host_name.cyan().bold());
            for (path_name, path) in paths {
                println!("    {} -> {}", path_name.cyan(), path);
            }
        }
        println!();
    }
    
    // 旧形式のパス表示（互換性）
    if let Some(ref old_paths) = config.paths {
        if !old_paths.is_empty() {
            println!("{}", "旧形式のパス（移行推奨）:".bold().dimmed());
            for (name, path) in old_paths {
                let path_type = if path.is_remote { "リモート" } else { "ローカル" };
                let expanded_path = if !path.is_remote {
                    Config::expand_path(&path.path)
                } else {
                    path.path.clone()
                };
                println!("  {} ({}) -> {}", name.cyan(), path_type.yellow(), expanded_path);
            }
        }
    }
    
    if config.local_paths.is_empty() && config.host_paths.is_empty() && 
       (config.paths.is_none() || config.paths.as_ref().unwrap().is_empty()) {
        println!("設定されているパスはありません");
    }
    
    Ok(())
}

/// インタラクティブにパスを追加します
pub fn add_path_interactive() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "=== インタラクティブ パス追加 ===".bold().blue());
    
    // パスタイプの選択
    println!("パスタイプを選択してください:");
    println!("1. ローカルパス");
    println!("2. リモートパス（ホスト別）");
    print!("選択 [1-2]: ");
    io::stdout().flush()?;
    
    let mut choice = String::new();
    io::stdin().read_line(&mut choice)?;
    let choice = choice.trim();
    
    match choice {
        "1" => add_local_path_interactive(),
        "2" => add_remote_path_interactive(),
        _ => {
            println!("{}: 無効な選択です", "ERROR".red());
            Ok(())
        }
    }
}

/// インタラクティブにローカルパスを追加します
fn add_local_path_interactive() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n{}", "--- ローカルパス追加 ---".bold());
    
    // パス名の入力
    print!("パス名（エイリアス）: ");
    io::stdout().flush()?;
    let mut name = String::new();
    io::stdin().read_line(&mut name)?;
    let name = name.trim();
    
    if name.is_empty() {
        println!("{}: パス名は必須です", "ERROR".red());
        return Ok(());
    }
    
    // パスの入力
    print!("パス: ");
    io::stdout().flush()?;
    let mut path = String::new();
    io::stdin().read_line(&mut path)?;
    let path = path.trim();
    
    if path.is_empty() {
        println!("{}: パスは必須です", "ERROR".red());
        return Ok(());
    }
    
    // 確認表示
    println!("\n{}", "=== 設定確認 ===".bold());
    println!("パス名: {}", name.cyan());
    println!("パス: {}", path);
    println!("タイプ: {}", "ローカル".green());
    
    print!("\nこの設定で追加しますか？ [y/N]: ");
    io::stdout().flush()?;
    let mut confirm = String::new();
    io::stdin().read_line(&mut confirm)?;
    let confirm = confirm.trim().to_lowercase();
    
    if confirm == "y" || confirm == "yes" {
        add_local_path(name, path)?;
        println!("{}: インタラクティブ追加が完了しました", "SUCCESS".green());
    } else {
        println!("{}: キャンセルされました", "INFO".yellow());
    }
    
    Ok(())
}

/// インタラクティブにリモートパスを追加します
fn add_remote_path_interactive() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n{}", "--- リモートパス追加 ---".bold());
    
    // 利用可能なホストを表示
    let config = Config::load()?;
    if config.hosts.is_empty() {
        println!("{}: ホストが設定されていません。先にホストを追加してください", "ERROR".red());
        return Ok(());
    }
    
    println!("利用可能なホスト:");
    for host_name in config.hosts.keys() {
        println!("  - {}", host_name.cyan());
    }
    
    // ホスト名の入力
    print!("ホスト名: ");
    io::stdout().flush()?;
    let mut host_name = String::new();
    io::stdin().read_line(&mut host_name)?;
    let host_name = host_name.trim();
    
    if !config.hosts.contains_key(host_name) {
        println!("{}: ホスト '{}' が存在しません", "ERROR".red(), host_name);
        return Ok(());
    }
    
    // パス名の入力
    print!("パス名（エイリアス）: ");
    io::stdout().flush()?;
    let mut path_name = String::new();
    io::stdin().read_line(&mut path_name)?;
    let path_name = path_name.trim();
    
    if path_name.is_empty() {
        println!("{}: パス名は必須です", "ERROR".red());
        return Ok(());
    }
    
    // パスの入力
    print!("リモートパス: ");
    io::stdout().flush()?;
    let mut path = String::new();
    io::stdin().read_line(&mut path)?;
    let path = path.trim();
    
    if path.is_empty() {
        println!("{}: パスは必須です", "ERROR".red());
        return Ok(());
    }
    
    // 確認表示
    println!("\n{}", "=== 設定確認 ===".bold());
    println!("ホスト: {}", host_name.cyan());
    println!("パス名: {}", path_name.cyan());
    println!("パス: {}", path);
    println!("タイプ: {}", "リモート".yellow());
    
    print!("\nこの設定で追加しますか？ [y/N]: ");
    io::stdout().flush()?;
    let mut confirm = String::new();
    io::stdin().read_line(&mut confirm)?;
    let confirm = confirm.trim().to_lowercase();
    
    if confirm == "y" || confirm == "yes" {
        add_host_path(host_name, path_name, path)?;
        println!("{}: インタラクティブ追加が完了しました", "SUCCESS".green());
    } else {
        println!("{}: キャンセルされました", "INFO".yellow());
    }
    
    Ok(())
}