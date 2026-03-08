mod commands;
mod error;
mod io;

use clap::{Parser, Subcommand};

use commands::convert::{ConvertArgs, run_convert};
use commands::grid_to_puzzle::{GridToPuzzleArgs, run_grid_to_puzzle};
use commands::solve::{SolveArgs, run_solve};
use commands::template::{TemplateArgs, run_template};
use error::CliError;

#[derive(Parser)]
#[command(name = "nonokit", about = "Nonogram CLI tool")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// パズルを解く
    Solve(SolveArgs),
    /// 空のパズルテンプレートを生成する
    Template(TemplateArgs),
    /// 画像をドットグリッド JSON に変換する
    Convert(ConvertArgs),
    /// ドットグリッドからパズルのクルーを計算する
    GridToPuzzle(GridToPuzzleArgs),
}

fn main() {
    let cli = Cli::parse();
    let result: Result<(), CliError> = match cli.command {
        Commands::Solve(args) => run_solve(args),
        Commands::Template(args) => run_template(args),
        Commands::Convert(args) => run_convert(args),
        Commands::GridToPuzzle(args) => run_grid_to_puzzle(args),
    };
    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
