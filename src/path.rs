// パス管理とファイル転送機能
//
// このモジュールは、ローカルおよびリモートパスのエイリアス管理と
// SCPを使用したファイル転送機能を提供します。

use crate::config::{Config, Path};
use colored::*;
use std::collections::HashMap;

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

/// 設定されているパスの一覧を表示します
/// 
/// 全ての設定済みパスをエイリアス名、種類（ローカル/リモート）、
/// 実際のパスと共に表示します。ローカルパスの場合はチルダ展開も行います。
/// 
/// # 戻り値
/// 成功時は()、失敗時はエラーを返します。
pub fn list_paths() -> Result<(), Box<dyn std::error::Error>> {
    // 現在の設定を読み込み
    let config = Config::load()?;

    // 旧バージョンのパス（互換性のため）
    if let Some(ref old_paths) = config.paths {
        if old_paths.is_empty() {
            println!("設定されているパスはありません（旧形式）");
        } else {
            // パス一覧を表示
            println!("{}", "設定済みパス（旧形式）:".bold());
            for (name, path) in old_paths {
                let path_type = if path.is_remote { "リモート" } else { "ローカル" };
                // ローカルパスの場合は展開表示
                let expanded_path = if !path.is_remote {
                    Config::expand_path(&path.path)
                } else {
                    path.path.clone()
                };
                println!("  {} ({}) -> {}", name.cyan(), path_type.yellow(), expanded_path);
            }
        }
    } else {
        println!("設定されているパスはありません");
    }

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