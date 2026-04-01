use crate::token::{Token, TokenType};
use std::iter::Peekable;
use std::str::Chars;

pub struct Lexer<'a> {
    input: Peekable<Chars<'a>>,
    line: usize,
    column: usize,
    indent_stack: Vec<usize>,
    dedent_queue: usize,
    at_start_of_line: bool,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Lexer {
            input: input.chars().peekable(),
            line: 1,
            column: 1,
            indent_stack: vec![0],
            dedent_queue: 0,
            at_start_of_line: true,
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();

        loop {
            if self.at_start_of_line {
                self.process_indent(&mut tokens)?;
                self.at_start_of_line = false;
            }

            if self.dedent_queue > 0 {
                tokens.push(Token::new(TokenType::Dedent, self.line, self.column));
                self.dedent_queue -= 1;
                continue;
            }

            match self.peek() {
                None => {
                    while self.indent_stack.len() > 1 {
                        tokens.push(Token::new(TokenType::Dedent, self.line, self.column));
                        self.indent_stack.pop();
                    }
                    tokens.push(Token::new(TokenType::EOF, self.line, self.column));
                    break;
                }
                Some(&ch) => {
                    if ch == '\n' {
                        self.consume_newline(&mut tokens);
                    } else if ch.is_whitespace() {
                        self.consume_whitespace();
                    } else if ch == '#' {
                        self.consume_comment();
                    } else if ch == '"' || ch == '「' {
                        tokens.push(self.read_string()?);
                    } else if ch == '[' || ch == '［' {
                        tokens.push(Token::new(TokenType::LeftBracket, self.line, self.column));
                        self.advance();
                    } else if ch == ']' || ch == '］' {
                        tokens.push(Token::new(TokenType::RightBracket, self.line, self.column));
                        self.advance();
                    } else if ch == '(' || ch == '（' {
                        tokens.push(Token::new(TokenType::LeftParen, self.line, self.column));
                        self.advance();
                    } else if ch == ')' || ch == '）' {
                        tokens.push(Token::new(TokenType::RightParen, self.line, self.column));
                        self.advance();
                    } else if ch == ':' || ch == '：' {
                        tokens.push(Token::new(TokenType::Colon, self.line, self.column));
                        self.advance();
                    } else if ch == ',' || ch == '，' || ch == '。' {
                        tokens.push(Token::new(TokenType::Comma, self.line, self.column));
                        self.advance();
                    } else {
                        tokens.push(self.read_token()?);
                    }
                }
            }
        }

        Ok(tokens)
    }

    fn peek(&mut self) -> Option<&char> {
        self.input.peek()
    }

    fn advance(&mut self) {
        if let Some(ch) = self.input.next() {
            if ch == '\n' {
                self.line += 1;
                self.column = 1;
                self.at_start_of_line = true;
            } else {
                self.column += 1;
            }
        }
    }

    fn consume_newline(&mut self, tokens: &mut Vec<Token>) {
        tokens.push(Token::new(TokenType::Newline, self.line, self.column));
        self.advance();
    }

    fn consume_whitespace(&mut self) {
        while let Some(&ch) = self.peek() {
            if ch == ' ' || ch == '　' {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn consume_comment(&mut self) {
        while let Some(&ch) = self.peek() {
            if ch == '\n' {
                break;
            }
            self.advance();
        }
    }

    fn process_indent(&mut self, tokens: &mut Vec<Token>) -> Result<(), String> {
        let mut indent = 0;
        while let Some(&ch) = self.peek() {
            if ch == ' ' {
                indent += 1;
                self.advance();
            } else if ch == '　' {
                indent += 1;
                self.advance();
            } else {
                break;
            }
        }

        let current_indent = *self.indent_stack.last().unwrap();

        if indent > current_indent {
            self.indent_stack.push(indent);
            tokens.push(Token::new(TokenType::Indent, self.line, self.column));
        } else if indent < current_indent {
            while self.indent_stack.len() > 1 && *self.indent_stack.last().unwrap() > indent {
                self.indent_stack.pop();
                self.dedent_queue += 1;
            }

            if *self.indent_stack.last().unwrap() != indent {
                return Err(format!("インデントが正しくありません。{}行目", self.line));
            }
        }

        Ok(())
    }

    fn read_string(&mut self) -> Result<Token, String> {
        let start_col = self.column;
        self.advance();

        let mut content = String::new();

        while let Some(&ch) = self.peek() {
            match ch {
                '"' | '」' => {
                    self.advance();
                    return Ok(Token::new(TokenType::String(content), self.line, start_col));
                }
                '{' => {
                    self.advance();
                    content.push(ch);
                    let mut brace_depth = 1;
                    while let Some(&next_ch) = self.peek() {
                        self.advance();
                        if next_ch == '}' {
                            brace_depth -= 1;
                            content.push(next_ch);
                            if brace_depth == 0 {
                                break;
                            }
                        } else if next_ch == '{' {
                            brace_depth += 1;
                            content.push(next_ch);
                        } else {
                            content.push(next_ch);
                        }
                    }
                }
                '\n' => {
                    return Err(format!("文字列が閉じていません。{}行目", self.line));
                }
                _ => {
                    content.push(ch);
                    self.advance();
                }
            }
        }

        Err(format!("文字列が閉じていません。{}行目", self.line))
    }

    fn read_token(&mut self) -> Result<Token, String> {
        let start_col = self.column;

        if let Some(&ch) = self.peek() {
            if ch.is_ascii_digit() {
                return Ok(self.read_number());
            }
        }

        let mut token_str = String::new();
        while let Some(&ch) = self.peek() {
            if ch.is_whitespace() || ch == '\n' || ch == '#' {
                break;
            }
            if ch == '"' || ch == '「' || ch == '」' {
                break;
            }
            if ch == ',' || ch == '，' || ch == '。' {
                break;
            }
            if ch == ':' || ch == '：' {
                break;
            }
            if ch == '[' || ch == '］' || ch == ']' || ch == '［' {
                break;
            }
            if ch == '(' || ch == '）' || ch == ')' || ch == '（' {
                break;
            }

            let normalized = self.normalize_char(ch);
            token_str.push(normalized);
            self.advance();
        }

        if token_str.is_empty() {
            return Err(format!("無効なトークン。{}行目", self.line));
        }

        if let Some(token_type) = self.match_operator(&token_str) {
            return Ok(Token::new(token_type, self.line, start_col));
        }

        let token_type = self.match_keyword_or_identifier(&token_str);
        Ok(Token::new(token_type, self.line, start_col))
    }

    fn read_number(&mut self) -> Token {
        let start_col = self.column;
        let mut num_str = String::new();
        let mut has_dot = false;

        while let Some(&ch) = self.peek() {
            let normalized = self.normalize_char(ch);
            if normalized.is_ascii_digit() {
                num_str.push(normalized);
                self.advance();
            } else if normalized == '.' && !has_dot {
                num_str.push(normalized);
                has_dot = true;
                self.advance();
            } else {
                break;
            }
        }

        let value: f64 = num_str.parse().unwrap_or(0.0);
        Token::new(TokenType::Number(value), self.line, start_col)
    }

    fn normalize_char(&self, ch: char) -> char {
        match ch {
            '０' => '0', '１' => '1', '２' => '2', '３' => '3', '４' => '4',
            '５' => '5', '６' => '6', '７' => '7', '８' => '8', '９' => '9',
            '＋' => '+', '－' => '-', '＊' => '*', '／' => '/',
            '＝' => '=', '！' => '!', '（' => '(', '）' => ')',
            '［' => '[', '］' => ']', '，' => ',', '：' => ':',
            _ => ch,
        }
    }

    fn match_operator(&self, s: &str) -> Option<TokenType> {
        match s {
            "+" => Some(TokenType::Plus),
            "-" => Some(TokenType::Minus),
            "*" => Some(TokenType::Multiply),
            "/" => Some(TokenType::Divide),
            "%" => Some(TokenType::Modulo),
            "=" => Some(TokenType::Equal),
            "==" => Some(TokenType::EqualEqual),
            "!=" => Some(TokenType::NotEqual),
            "<" => Some(TokenType::Less),
            "<=" => Some(TokenType::LessEqual),
            ">" => Some(TokenType::Greater),
            ">=" => Some(TokenType::GreaterEqual),
            "の" => Some(TokenType::Of),
            _ => None,
        }
    }

    fn match_keyword_or_identifier(&self, s: &str) -> TokenType {
        match s {
            "もし" | "モシ" => TokenType::If,
            "そうでなければ" | "またはもし" | "又はもし" => TokenType::Elif,
            "違えば" | "ちがえば" => TokenType::Else,
            "回" | "カイ" => TokenType::For,
            "の間" | "のあいだ" => TokenType::While,
            "回数" => TokenType::LoopCount,
            "とは" | "〜とは" | "トハ" => TokenType::Function,
            "戻す" | "もどす" | "モドス" => TokenType::Return,
            "表示" | "ひょうじ" => TokenType::Print,
            "聞く" | "きく" => TokenType::Input,
            "そして" | "且つ" | "かつ" => TokenType::And,
            "または" | "又は" => TokenType::Or,
            "ではない" | "ない" => TokenType::Not,
            "真" | "ほんとう" => TokenType::True,
            "偽" | "うそ" => TokenType::False,
            "なし" | "null" => TokenType::Null,
            _ => TokenType::Identifier(s.to_string()),
        }
    }
}
