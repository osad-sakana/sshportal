// パス管理とファイル転送機能
//
// このモジュールは、ローカルおよびリモートパスのエイリアス管理と
// SCPを使用したファイル転送機能を提供します。

use crate::config::{Config, Path};
use colored::*;

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

    // 同名のパスが既に存在するかチェック
    if config.paths.contains_key(name) {
        println!("{}: パス '{}' は既に存在します", "警告".yellow(), name);
        return Ok(());
    }

    // 新しいパス情報を作成
    let path_entry = Path {
        path: path.to_string(),
        is_remote,
    };

    // 設定にパスを追加し、保存
    config.paths.insert(name.to_string(), path_entry);
    config.save()?;

    let path_type = if is_remote { "リモート" } else { "ローカル" };
    println!("{}: {} パス '{}' を追加しました", "成功".green(), path_type, name);
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

    // パスが存在するかチェック
    if !config.paths.contains_key(name) {
        println!("{}: パス '{}' が見つかりません", "エラー".red(), name);
        return Ok(());
    }

    // パスを削除し、設定を保存
    config.paths.remove(name);
    config.save()?;

    println!("{}: パス '{}' を削除しました", "成功".green(), name);
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

    // パスが設定されているかチェック
    if config.paths.is_empty() {
        println!("設定されているパスはありません");
        return Ok(());
    }

    // パス一覧を表示
    println!("{}", "設定済みパス:".bold());
    for (name, path) in &config.paths {
        let path_type = if path.is_remote { "リモート" } else { "ローカル" };
        // ローカルパスの場合は展開表示
        let expanded_path = if !path.is_remote {
            Config::expand_path(&path.path)
        } else {
            path.path.clone()
        };
        println!("  {} ({}) -> {}", name.cyan(), path_type.yellow(), expanded_path);
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

    println!("{}: {} から {} にコピー中...", "情報".blue(), src, dst);

    // SCPコマンドを構築
    let mut cmd = std::process::Command::new("scp");
    cmd.arg("-r"); // 再帰的コピーのオプション

    // コピー元がローカルかどうかを事前に判定
    let src_is_local = src_host.is_none();

    // コピー元の設定
    if let Some(ref host) = src_host {
        // リモートホストからのコピーの場合
        let host_config = config.hosts.get(host).ok_or(format!("ホスト '{}' が見つかりません", host))?;
        cmd.arg("-P").arg(&host_config.port.to_string());
        cmd.arg(format!("{}:{}", host_config.connection, src_path));
    } else {
        // ローカルファイルからのコピーの場合
        let expanded_src = Config::expand_path(&src_path);
        cmd.arg(expanded_src);
    }

    // コピー先の設定
    if let Some(ref host) = dst_host {
        // リモートホストへのコピーの場合
        let host_config = config.hosts.get(host).ok_or(format!("ホスト '{}' が見つかりません", host))?;
        // コピー元がローカルの場合のみポート番号を指定
        if src_is_local {
            cmd.arg("-P").arg(&host_config.port.to_string());
        }
        cmd.arg(format!("{}:{}", host_config.connection, dst_path));
    } else {
        // ローカルファイルへのコピーの場合
        let expanded_dst = Config::expand_path(&dst_path);
        cmd.arg(expanded_dst);
    }

    // SCPコマンドを実行
    let status = cmd.status()?;

    // 結果の表示
    if status.success() {
        println!("{}: コピーが正常に完了しました", "成功".green());
    } else {
        println!("{}: コピーに失敗しました", "エラー".red());
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
            // パス部分がパスエイリアスかチェック
            if config.paths.contains_key(&path) {
                let path_entry = &config.paths[&path];
                // リモートパスでない場合はエラー
                if !path_entry.is_remote {
                    return Err(format!("パス '{}' はリモートパスではありません", path).into());
                }
                return Ok((path_entry.path.clone(), Some(host)));
            }
            // 直接パスの場合
            return Ok((path, Some(host)));
        }

        // ホストが見つからない場合は文字列をそのまま返す
        return Ok((spec.to_string(), None));
    }

    // コロンが含まれない場合はローカルパスまたはパスエイリアス
    if config.paths.contains_key(spec) {
        let path_entry = &config.paths[spec];
        // リモートパスの場合はホスト指定が必要
        if path_entry.is_remote {
            return Err(format!("パス '{}' はリモートパスですが、ホストが指定されていません", spec).into());
        }
        return Ok((path_entry.path.clone(), None));
    }

    // パスエイリアスでない場合は文字列をそのまま返す
    Ok((spec.to_string(), None))
}