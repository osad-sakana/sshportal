// コマンドライン引数の定義と処理
//
// このモジュールは、clapライブラリを使用してコマンドライン引数を定義し、
// 適切な機能モジュールに処理を委譲します。

use clap::{Parser, Subcommand};
use crate::host;
use crate::path;

/// sshportalのメインコマンドライン構造体
/// 
/// clapライブラリを使用してコマンドライン引数を解析します。
/// サブコマンド形式で各機能を提供します。
#[derive(Parser)]
#[command(name = "sshportal")]
#[command(about = "SSH接続とSCP転送管理ツール")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

/// 利用可能なサブコマンドの定義
/// 
/// ホスト管理、パス管理、ファイル転送の各機能を
/// サブコマンドとして提供します。
#[derive(Subcommand)]
pub enum Commands {
    /// 新しいホストを追加
    #[command(about = "新しいホストを追加")]
    AddHost {
        #[arg(help = "ホストのエイリアス名")]
        name: String,
        #[arg(help = "接続文字列（user@hostname）")]
        connection: String,
        #[arg(short, long, default_value = "22", help = "SSHポート番号")]
        port: u16,
        #[arg(short = 'i', long, help = "SSH秘密鍵のパス")]
        identity_file: Option<String>,
    },
    /// ホストを削除
    #[command(about = "ホストを削除")]
    RemoveHost {
        #[arg(help = "ホストのエイリアス名")]
        name: String,
    },
    /// 設定済みホストの一覧表示
    #[command(about = "設定済みホストの一覧表示")]
    ListHosts,
    /// ホストに接続
    #[command(about = "ホストに接続")]
    Connect {
        #[arg(help = "ホストのエイリアス名")]
        host: String,
    },
    /// パスエイリアスを追加
    #[command(about = "パスエイリアスを追加")]
    AddPath {
        #[arg(help = "パスのエイリアス名")]
        name: String,
        #[arg(help = "パスの場所")]
        path: String,
        #[arg(short = 'r', long, help = "リモートパスとしてマーク")]
        remote: bool,
    },
    /// パスエイリアスを削除
    #[command(about = "パスエイリアスを削除")]
    RemovePath {
        #[arg(help = "パスのエイリアス名")]
        name: String,
    },
    /// 設定済みパスの一覧表示
    #[command(about = "設定済みパスの一覧表示")]
    ListPaths,
    /// パスエイリアスを使用してSCPでファイルをコピー
    #[command(about = "パスエイリアスを使用してSCPでファイルをコピー")]
    Copy {
        #[arg(help = "コピー元パス（エイリアスまたは実際のパス）")]
        src: String,
        #[arg(help = "コピー先パス（エイリアスまたはhost:path）")]
        dst: String,
    },
}

/// コマンドを処理します
/// 
/// 解析されたコマンドライン引数に基づいて、適切な機能モジュールの
/// 関数を呼び出します。各サブコマンドは対応する機能に委譲されます。
/// 
/// # 引数
/// * `cli` - 解析されたコマンドライン引数
/// 
/// # 戻り値
/// 成功時は()、失敗時はエラーを返します。
pub fn handle_command(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    match cli.command {
        // ホスト管理コマンド
        Commands::AddHost { name, connection, port, identity_file } => {
            host::add_host(&name, &connection, port, identity_file.as_deref())
        }
        Commands::RemoveHost { name } => {
            host::remove_host(&name)
        }
        Commands::ListHosts => {
            host::list_hosts()
        }
        Commands::Connect { host } => {
            host::connect_host(&host)
        }
        // パス管理コマンド
        Commands::AddPath { name, path, remote } => {
            path::add_path(&name, &path, remote)
        }
        Commands::RemovePath { name } => {
            path::remove_path(&name)
        }
        Commands::ListPaths => {
            path::list_paths()
        }
        // ファイル転送コマンド
        Commands::Copy { src, dst } => {
            path::copy_files(&src, &dst)
        }
    }
}