use clap::{Parser, Subcommand};

/// clicense-server - 开源许可证 API 服务器
#[derive(Parser, Debug)]
#[command(name = "clicense-server", version, about = "An open source license API server", arg_required_else_help = true)]
pub struct Cli {
    /// Enable verbose output (show config paths, file details, HTTP info, etc.)
    #[arg(short, long, global = true)]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// 初始化服务器（写入内置许可证模板）
    Init {
        #[arg(long)]
        licenses_dir: Option<String>,
        #[arg(long)]
        force: bool,
    },
    /// 查看/设置服务器配置
    Config {
        key: Option<String>,
        value: Option<String>,
        #[arg(long)]
        list: bool,
        #[arg(long)]
        reset: Option<String>,
    },
    /// 从远程 API 克隆许可证模板
    Clone {
        url: String,
        #[arg(long)]
        licenses_dir: Option<String>,
        #[arg(long)]
        force: bool,
    },
    /// 显示版本号
    Version,
    /// 启动 API 服务器
    Run {
        #[arg(long)]
        host: Option<String>,
        #[arg(long)]
        port: Option<u16>,
        #[arg(long)]
        licenses_dir: Option<String>,
    },
    /// 添加许可证模板
    Add {
        file: String,
        #[arg(long)]
        name: String,
        #[arg(long)]
        force: bool,
        #[arg(long)]
        licenses_dir: Option<String>,
    },
    /// 删除许可证模板
    Remove {
        names: Vec<String>,
        #[arg(long, conflicts_with = "names")]
        all: bool,
        #[arg(long)]
        licenses_dir: Option<String>,
    },
    /// 列出许可证模板
    List {
        name: Option<String>,
        #[arg(long)]
        licenses_dir: Option<String>,
    },
}
