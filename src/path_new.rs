// 新しいパス管理機能の追加関数
//
// ホスト別パス管理とインタラクティブ設定機能を提供します。

use crate::config::{Config, LocalPath, RemotePath};
use colored::*;
use std::collections::HashMap;
use std::io::{self, Write};

/// ローカルパスエイリアスを追加します
pub fn add_local_path(name: &str, path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut config = Config::load()?;
    
    if config.local_paths.contains_key(name) {
        println!("{}: ローカルパス '{}' は既に存在します", "WARN".yellow(), name);
        return Ok(());
    }
    
    let local_path = LocalPath {
        path: path.to_string(),
    };
    
    config.local_paths.insert(name.to_string(), local_path);
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
    
    let remote_path = RemotePath {
        path: path.to_string(),
    };
    
    host_paths.insert(path_name.to_string(), remote_path);
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
        for (name, local_path) in &config.local_paths {
            let expanded_path = Config::expand_path(&local_path.path);
            println!("  {} -> {}", name.cyan(), expanded_path);
        }
        println!();
    }
    
    // ホスト別リモートパス表示
    if !config.host_paths.is_empty() {
        println!("{}", "リモートパス（ホスト別）:".bold().yellow());
        for (host_name, paths) in &config.host_paths {
            println!("  {}:", host_name.cyan().bold());
            for (path_name, remote_path) in paths {
                println!("    {} -> {}", path_name.cyan(), remote_path.path);
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