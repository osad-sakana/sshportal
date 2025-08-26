// sshportal - SSH接続とSCP転送を簡略化するzshプラグイン
//
// このプログラムはSSH接続先とパスのエイリアス管理を提供し、
// zshでの効率的なSSH作業をサポートします。

mod config;   // 設定ファイルの読み書き機能
mod host;     // ホスト管理機能
mod path;     // パス管理とファイル転送機能
mod commands; // コマンドライン引数の定義と処理

use clap::Parser;
use commands::{Cli, handle_command};
use colored::*;

/// メイン関数
/// 
/// コマンドライン引数を解析し、適切なサブコマンドを実行します。
/// エラーが発生した場合は、色付きでエラーメッセージを表示し、
/// 終了コード1でプログラムを終了します。
fn main() {
    // コマンドライン引数を解析
    let cli = Cli::parse();
    
    // コマンドを実行し、エラーが発生した場合は適切に処理
    if let Err(e) = handle_command(cli) {
        // 赤色でエラーメッセージを表示
        eprintln!("{}: {}", "Error".red(), e);
        std::process::exit(1);
    }
}