use crate::interpreter::Interpreter;
use crate::lexer::Lexer;
use crate::parser::Parser;
use std::fs;
use std::io::{self, Write};

pub fn run_file(filename: &str) -> Result<(), String> {
    let source = fs::read_to_string(filename)
        .map_err(|e| format!("ファイルを読み込めません: {}", e))?;

    let mut lexer = Lexer::new(&source);
    let tokens = lexer.tokenize()?;

    let mut parser = Parser::new(tokens);
    let program = parser.parse()?;

    let mut interpreter = Interpreter::new();
    interpreter.interpret(&program)?;

    Ok(())
}

pub fn run_repl() {
    let mut interpreter = Interpreter::new();
    let mut buffer = String::new();
    let mut in_block = false;

    println!("Hana（花） v0.2 - 日本語学習用プログラミング言語");
    println!("終了するには「exit」と入力してください");
    println!();

    loop {
        let prompt = if in_block { "... " } else { ">>> " };
        print!("{}", prompt);
        io::stdout().flush().unwrap();

        buffer.clear();

        match io::stdin().read_line(&mut buffer) {
            Ok(0) => {
                println!();
                break;
            }
            Ok(_) => {
                let line_owned = buffer.clone();
                let line = line_owned.trim();

                if line == "exit" {
                    break;
                }

                if line.is_empty() {
                    continue;
                }

                if in_block && line.is_empty() {
                    if let Err(e) = execute(&mut interpreter, &buffer) {
                        eprintln!("エラー: {}", e);
                    }
                    buffer.clear();
                    in_block = false;
                    continue;
                }

                if in_block {
                    buffer.push('\n');
                    buffer.push_str(line);
                    continue;
                }

                match execute(&mut interpreter, line) {
                    Ok(_) => {}
                    Err(e) => {
                        eprintln!("エラー: {}", e);
                        buffer.clear();
                        in_block = false;
                    }
                }
            }
            Err(e) => {
                eprintln!("読み込みエラー: {}", e);
                break;
            }
        }
    }
}

fn execute(interpreter: &mut Interpreter, code: &str) -> Result<(), String> {
    let mut lexer = Lexer::new(code);
    let tokens = lexer.tokenize()?;

    let mut parser = Parser::new(tokens);
    let program = parser.parse()?;

    interpreter.interpret(&program)?;
    Ok(())
}
