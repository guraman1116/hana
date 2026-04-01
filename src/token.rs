use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // リテラル
    Number(f64),
    String(String),
    Boolean(bool),
    Null,

    // 識別子
    Identifier(String),

    // 演算子
    Plus,
    Minus,
    Multiply,
    Divide,
    Modulo,
    Equal,        // =
    EqualEqual,   // ==
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,

    // キーワード
    If,
    Elif,
    Else,
    For,         // 回繰り返す用
    While,
    Of,          // の（プロパティアクセス）
    LoopCount,   // 回数
    Function,    // とは
    Return,
    Print,
    Input,
    And,
    Or,
    Not,
    True,
    False,

    // 区切り子
    Colon,
    LeftParen,
    RightParen,
    LeftBracket,
    RightBracket,
    Comma,
    Dot,         // 配列区切りの。用

    // その他
    Newline,
    Indent,
    Dedent,
    EOF,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenType,
    pub line: usize,
    pub column: usize,
}

impl Token {
    pub fn new(kind: TokenType, line: usize, column: usize) -> Self {
        Token { kind, line, column }
    }
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenType::Number(n) => write!(f, "{}", n),
            TokenType::String(s) => write!(f, "\"{}\"", s),
            TokenType::Boolean(b) => write!(f, "{}", b),
            TokenType::Null => write!(f, "なし"),
            TokenType::Identifier(s) => write!(f, "{}", s),
            TokenType::Plus => write!(f, "+"),
            TokenType::Minus => write!(f, "-"),
            TokenType::Multiply => write!(f, "*"),
            TokenType::Divide => write!(f, "/"),
            TokenType::Modulo => write!(f, "%"),
            TokenType::Equal => write!(f, "="),
            TokenType::EqualEqual => write!(f, "=="),
            TokenType::NotEqual => write!(f, "!="),
            TokenType::Less => write!(f, "<"),
            TokenType::LessEqual => write!(f, "<="),
            TokenType::Greater => write!(f, ">"),
            TokenType::GreaterEqual => write!(f, ">="),
            TokenType::If => write!(f, "もし"),
            TokenType::Elif => write!(f, "そうでなければ"),
            TokenType::Else => write!(f, "違えば"),
            TokenType::For => write!(f, "回"),
            TokenType::While => write!(f, "の間"),
            TokenType::Of => write!(f, "の"),
            TokenType::LoopCount => write!(f, "回数"),
            TokenType::Function => write!(f, "とは"),
            TokenType::Return => write!(f, "戻す"),
            TokenType::Print => write!(f, "表示"),
            TokenType::Input => write!(f, "聞く"),
            TokenType::And => write!(f, "そして"),
            TokenType::Or => write!(f, "または"),
            TokenType::Not => write!(f, "ではない"),
            TokenType::True => write!(f, "真"),
            TokenType::False => write!(f, "偽"),
            TokenType::Colon => write!(f, ":"),
            TokenType::LeftParen => write!(f, "("),
            TokenType::RightParen => write!(f, ")"),
            TokenType::LeftBracket => write!(f, "["),
            TokenType::RightBracket => write!(f, "]"),
            TokenType::Comma => write!(f, ","),
            TokenType::Dot => write!(f, "."),
            TokenType::Newline => write!(f, "\\n"),
            TokenType::Indent => write!(f, "<indent>"),
            TokenType::Dedent => write!(f, "<dedent>"),
            TokenType::EOF => write!(f, "<eof>"),
        }
    }
}
