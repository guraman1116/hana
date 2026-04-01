use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum Expr {
    // リテラル
    Number(f64),
    String(String),
    Boolean(bool),
    Null,
    Array(Vec<Expr>),

    // 変数参照
    Variable(String),

    // 二項演算
    BinaryOp {
        left: Box<Expr>,
        op: BinaryOperator,
        right: Box<Expr>,
    },

    // 単項演算
    UnaryOp {
        op: UnaryOperator,
        expr: Box<Expr>,
    },

    // プロパティアクセス (文字列の文字数, 配列のN番目など)
    PropertyAccess {
        object: Box<Expr>,
        property: Property,
    },

    // 関数呼び出し
    FunctionCall {
        name: String,
        args: Vec<Expr>,
    },

    // 配列インデックス
    Index {
        array: Box<Expr>,
        index: Box<Expr>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinaryOperator {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    And,
    Or,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnaryOperator {
    Neg,   // -
    Not,   // ではない
}

#[derive(Debug, Clone)]
pub enum Property {
    Length,    // 文字数 or 要素数
    Index(f64), // N番目 (Nは0-based index)
}

#[derive(Debug, Clone)]
pub enum Stmt {
    // 式文（関数呼び出しなど）
    Expr(Expr),

    // 代入
    Assignment {
        name: String,
        value: Expr,
    },

    // if/elif/else
    If {
        condition: Expr,
        then_branch: Vec<Stmt>,
        elif_branches: Vec<(Expr, Vec<Stmt>)>,
        else_branch: Option<Vec<Stmt>>,
    },

    // N回 繰り返す
    ForLoop {
        count: Expr,
        body: Vec<Stmt>,
    },

    // Xの間 繰り返す
    WhileLoop {
        condition: Expr,
        body: Vec<Stmt>,
    },

    // 関数定義
    FunctionDef {
        name: String,
        params: Vec<String>,
        body: Vec<Stmt>,
    },

    // 戻り値
    Return(Expr),
}

// プログラム全体
pub type Program = Vec<Stmt>;

// 値のランタイム表現
#[derive(Debug, Clone)]
pub enum Value {
    Number(f64),
    String(String),
    Boolean(bool),
    Array(Vec<Value>),
    Null,
    Function {
        params: Vec<String>,
        body: Vec<Stmt>,
        closure: HashMap<String, Value>,
    },
}

impl Value {
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Number(_) => "数値",
            Value::String(_) => "文字列",
            Value::Boolean(_) => "真偽",
            Value::Array(_) => "配列",
            Value::Null => "なし",
            Value::Function { .. } => "関数",
        }
    }
}
