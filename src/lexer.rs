use crate::token::{Token, TokenType};

/// キーワード一覧（長い順）
const KEYWORDS: &[(&str, TokenType)] = &[
    // 長い順（最長マッチ優先）
    ("そうでなければ", TokenType::Elif),
    ("またはもし", TokenType::Elif),
    ("又はもし", TokenType::Elif),
    ("ほんとう", TokenType::True),
    ("繰り返す", TokenType::Repeat),
    ("ひょうじ", TokenType::Print),
    ("ではない", TokenType::Not),
    ("のあいだ", TokenType::While),
    ("ちがえば", TokenType::Else),
    ("違えば", TokenType::Else),
    ("もどす", TokenType::Return),
    ("かつ", TokenType::And),
    ("回数", TokenType::LoopCount),
    ("とは", TokenType::Function),
    ("聞く", TokenType::Input),
    ("表示", TokenType::Print),
    ("戻す", TokenType::Return),
    ("なら", TokenType::Then),
    ("ない", TokenType::Not),
    ("且つ", TokenType::And),
    ("または", TokenType::Or),
    ("又は", TokenType::Or),
    ("もし", TokenType::If),
    ("カイ", TokenType::For),
    ("偽", TokenType::False),
    ("真", TokenType::True),
    ("回", TokenType::For),
    ("の", TokenType::Of),
    ("モシ", TokenType::If),
    ("トハ", TokenType::Function),
    ("モドス", TokenType::Return),
    ("なし", TokenType::Null),
];

/// 2文字演算子（1文字演算子より優先）
const TWO_CHAR_OPS: &[(&str, TokenType)] = &[
    ("==", TokenType::EqualEqual),
    ("!=", TokenType::NotEqual),
    ("<=", TokenType::LessEqual),
    (">=", TokenType::GreaterEqual),
];

/// 1文字演算子
const ONE_CHAR_OPS: &[char] = &['+', '-', '*', '/', '%', '=', '<', '>'];

/// 文字列開始・終了
const STR_OPEN: &[char] = &['"', '「'];
const STR_CLOSE: &[char] = &['"', '」'];
const BRACKET_OPEN: &[char] = &['[', '［', '(', '（'];
const BRACKET_CLOSE: &[char] = &[']', '］', ')', '）'];

pub struct Lexer {
    chars: Vec<char>,
    pos: usize,
    line: usize,
    column: usize,
    indent_stack: Vec<usize>,
}

fn normalize_char(ch: char) -> char {
    match ch {
        '０'..='９' => char::from_u32(ch as u32 - '０' as u32 + '0' as u32).unwrap(),
        '＋' => '+', '－' => '-', '＊' => '*', '／' => '/',
        '＝' => '=', '！' => '!',
        _ => ch,
    }
}

fn is_delimiter(ch: char) -> bool {
    ch == '\n' || ch == '#' || ch == ' ' || ch == '\t' || ch == '　'
        || ch == '"' || ch == '「' || ch == '」'
        || ch == '[' || ch == '］' || ch == ']' || ch == '［'
        || ch == '(' || ch == '）' || ch == ')' || ch == '（'
        || ch == ':' || ch == '：'
        || ch == ',' || ch == '，'
        || ch == '。'
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Lexer {
            chars: input.chars().collect(),
            pos: 0,
            line: 1,
            column: 1,
            indent_stack: vec![0],
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn advance(&mut self) -> Option<char> {
        if self.pos >= self.chars.len() {
            return None;
        }
        let ch = self.chars[self.pos];
        self.pos += 1;
        if ch == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        Some(ch)
    }

    fn peek_at(&self, offset: usize) -> Option<char> {
        self.chars.get(self.pos + offset).copied()
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();

        loop {
            // インデント処理
            if self.pos == 0 || (self.pos > 0 && self.chars[self.pos - 1] == '\n') {
                self.process_indent(&mut tokens)?;
            }

            let ch = match self.peek() {
                None => {
                    // EOF: 残りDedent
                    while self.indent_stack.len() > 1 {
                        tokens.push(Token::new(TokenType::Dedent, self.line, self.column));
                        self.indent_stack.pop();
                    }
                    tokens.push(Token::new(TokenType::EOF, self.line, self.column));
                    break;
                }
                Some(c) => c,
            };

            if ch == '\n' {
                tokens.push(Token::new(TokenType::Newline, self.line, self.column));
                self.advance();
            } else if ch == ' ' || ch == '\t' || ch == '　' {
                // 行頭以外の空白はスキップ
                // （行頭は process_indent で処理済み）
                self.advance();
            } else if ch == '#' {
                self.consume_comment();
            } else if STR_OPEN.contains(&ch) {
                tokens.push(self.read_string()?);
            } else if BRACKET_OPEN.contains(&ch) {
                let tt = if ch == '[' || ch == '［' { TokenType::LeftBracket } else { TokenType::LeftParen };
                tokens.push(Token::new(tt, self.line, self.column));
                self.advance();
            } else if BRACKET_CLOSE.contains(&ch) {
                let tt = if ch == ']' || ch == '］' { TokenType::RightBracket } else { TokenType::RightParen };
                tokens.push(Token::new(tt, self.line, self.column));
                self.advance();
            } else if ch == ':' || ch == '：' {
                tokens.push(Token::new(TokenType::Colon, self.line, self.column));
                self.advance();
            } else if ch == ',' || ch == '，' || ch == '。' {
                tokens.push(Token::new(TokenType::Comma, self.line, self.column));
                self.advance();
            } else {
                // チャンク読み込み → キーワード/演算子/識別子に分割
                let start_col = self.column;
                let chunk = self.read_chunk();
                let mut split = self.split_chunk(&chunk, self.line, start_col);
                tokens.append(&mut split);
            }
        }

        Ok(tokens)
    }

    fn read_chunk(&mut self) -> String {
        let mut chunk = String::new();
        while let Some(ch) = self.peek() {
            if is_delimiter(ch) {
                break;
            }
            chunk.push(normalize_char(ch));
            self.advance();
        }
        chunk
    }

    fn split_chunk(&self, chunk: &str, line: usize, start_col: usize) -> Vec<Token> {
        let chars: Vec<char> = chunk.chars().collect();
        let mut tokens = Vec::new();
        let mut pos = 0;
        let mut col = start_col;

        while pos < chars.len() {
            // 数字
            if chars[pos].is_ascii_digit() {
                let num_start = col;
                let mut num_str = String::new();
                let mut has_dot = false;
                while pos < chars.len() && (chars[pos].is_ascii_digit() || (chars[pos] == '.' && !has_dot)) {
                    if chars[pos] == '.' { has_dot = true; }
                    num_str.push(chars[pos]);
                    pos += 1;
                    col += 1;
                }
                let value: f64 = num_str.parse().unwrap_or(0.0);
                tokens.push(Token::new(TokenType::Number(value), line, num_start));
                continue;
            }

            // 2文字演算子
            if pos + 1 < chars.len() {
                let two = format!("{}{}", chars[pos], chars[pos + 1]);
                if let Some(tt) = TWO_CHAR_OPS.iter().find(|(op, _)| *op == two.as_str()).map(|(_, tt)| tt.clone()) {
                    tokens.push(Token::new(tt, line, col));
                    pos += 2;
                    col += 2;
                    continue;
                }
            }

            // 1文字演算子
            if ONE_CHAR_OPS.contains(&chars[pos]) {
                let tt = match chars[pos] {
                    '+' => TokenType::Plus,
                    '-' => TokenType::Minus,
                    '*' => TokenType::Multiply,
                    '/' => TokenType::Divide,
                    '%' => TokenType::Modulo,
                    '=' => TokenType::Equal,
                    '<' => TokenType::Less,
                    '>' => TokenType::Greater,
                    _ => unreachable!(),
                };
                tokens.push(Token::new(tt, line, col));
                pos += 1;
                col += 1;
                continue;
            }

            // 最長キーワードマッチ
            let mut matched_kw: Option<(&str, TokenType)> = None;
            for &(kw, ref tt) in KEYWORDS {
                let kw_chars: Vec<char> = kw.chars().collect();
                if pos + kw_chars.len() <= chars.len()
                    && chars[pos..pos + kw_chars.len()] == kw_chars[..]
                    && (matched_kw.is_none() || kw_chars.len() > matched_kw.as_ref().unwrap().0.chars().count())
                {
                    matched_kw = Some((kw, tt.clone()));
                }
            }

            if let Some((kw, tt)) = matched_kw {
                tokens.push(Token::new(tt, line, col));
                let kw_len = kw.chars().count();
                pos += kw_len;
                col += kw_len;
                continue;
            }

            // 識別子
            let ident_start = col;
            let mut ident = String::new();
            while pos < chars.len() {
                let c = chars[pos];
                // 次がキーワード開始なら止める
                let mut starts_kw = false;
                for &(kw, _) in KEYWORDS {
                    let kw_chars: Vec<char> = kw.chars().collect();
                    if pos + kw_chars.len() <= chars.len()
                        && chars[pos..pos + kw_chars.len()] == kw_chars[..]
                    {
                        starts_kw = true;
                        break;
                    }
                }
                if starts_kw || ONE_CHAR_OPS.contains(&c) || c.is_ascii_digit() || is_delimiter(c) {
                    break;
                }
                ident.push(c);
                pos += 1;
                col += 1;
            }

            if !ident.is_empty() {
                tokens.push(Token::new(TokenType::Identifier(ident), line, ident_start));
            } else {
                // 未知の1文字（正規化後）
                pos += 1;
                col += 1;
            }
        }

        tokens
    }

    fn consume_comment(&mut self) {
        while let Some(ch) = self.peek() {
            if ch == '\n' { break; }
            self.advance();
        }
    }

    fn process_indent(&mut self, tokens: &mut Vec<Token>) -> Result<(), String> {
        let mut indent = 0;
        while let Some(ch) = self.peek() {
            if ch == ' ' || ch == '　' {
                indent += 1;
                self.advance();
            } else if ch == '\t' {
                indent += 4;
                self.advance();
            } else {
                break;
            }
        }

        let current = *self.indent_stack.last().unwrap();
        if indent > current {
            self.indent_stack.push(indent);
            tokens.push(Token::new(TokenType::Indent, self.line, self.column));
        } else if indent < current {
            while self.indent_stack.len() > 1 && *self.indent_stack.last().unwrap() > indent {
                self.indent_stack.pop();
                tokens.push(Token::new(TokenType::Dedent, self.line, self.column));
            }
            if *self.indent_stack.last().unwrap() != indent {
                return Err(format!("インデントが正しくありません。{}行目", self.line));
            }
        }
        Ok(())
    }

    fn read_string(&mut self) -> Result<Token, String> {
        let start_col = self.column;
        let _open = self.advance().unwrap();

        let mut content = String::new();
        while let Some(ch) = self.peek() {
            if STR_CLOSE.contains(&ch) {
                self.advance();
                return Ok(Token::new(TokenType::String(content), self.line, start_col));
            }
            if ch == '\n' {
                return Err(format!("文字列が閉じていません。{}行目", self.line));
            }
            content.push(ch);
            self.advance();
        }
        Err(format!("文字列が閉じていません。{}行目", self.line))
    }
}
