// Hana - 日本語学習用プログラミング言語

mod ast;
mod interpreter;
mod lexer;
mod parser;
mod repl;
mod token;

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        // ファイル実行モード
        match repl::run_file(&args[1]) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("エラー: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        // REPLモード
        repl::run_repl();
    }
}
