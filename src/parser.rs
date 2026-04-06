use crate::token::{Token, TokenType};
use crate::ast::*;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Result<Program, String> {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            if matches!(self.peek().kind, TokenType::EOF | TokenType::Newline | TokenType::Dedent) {
                self.advance();
                continue;
            }
            statements.push(self.parse_statement()?);
        }
        Ok(statements)
    }

    fn parse_statement(&mut self) -> Result<Stmt, String> {
        match &self.peek().kind {
            TokenType::If => self.parse_if(),
            TokenType::For => self.parse_for_loop(),
            TokenType::While => self.parse_while_loop(),
            TokenType::Return => self.parse_return(),
            TokenType::Identifier(_) if self.is_function_def() => self.parse_function_def(),
            _ => self.parse_expr_or_assignment(),
        }
    }

    /// 関数定義かチェック: `Nameとは(params)：` のパターン
    fn is_function_def(&self) -> bool {
        let i = self.current;
        // Identifier
        if !matches!(self.tokens.get(i).map(|t| &t.kind), Some(TokenType::Identifier(_))) {
            return false;
        }
        // とは (Function)
        if !matches!(self.tokens.get(i + 1).map(|t| &t.kind), Some(TokenType::Function)) {
            return false;
        }
        // (
        if !matches!(self.tokens.get(i + 2).map(|t| &t.kind), Some(TokenType::LeftParen)) {
            return false;
        }
        true
    }

    fn parse_function_def(&mut self) -> Result<Stmt, String> {
        let name = match &self.peek().kind {
            TokenType::Identifier(s) => s.clone(),
            _ => return Err(format!("{}行目: 関数名が必要です", self.peek().line)),
        };
        self.advance(); // name

        // とは
        if matches!(self.peek().kind, TokenType::Function) {
            self.advance();
        } else {
            return Err(format!("{}行目: 'とは'が必要です", self.peek().line));
        }

        // params
        let mut params = Vec::new();
        if matches!(self.peek().kind, TokenType::LeftParen) {
            self.advance(); // (
            while !matches!(self.peek().kind, TokenType::RightParen | TokenType::EOF) {
                if matches!(self.peek().kind, TokenType::Comma) {
                    self.advance();
                    continue;
                }
                match &self.peek().kind {
                    TokenType::Identifier(s) => {
                        params.push(s.clone());
                        self.advance();
                    }
                    _ => return Err(format!("{}行目: パラメータ名が必要です", self.peek().line)),
                }
            }
            if matches!(self.peek().kind, TokenType::RightParen) {
                self.advance(); // )
            }
        }

        // ：
        if matches!(self.peek().kind, TokenType::Colon) {
            self.advance();
        }

        let body = self.parse_block()?;
        Ok(Stmt::FunctionDef { name, params, body })
    }

    fn parse_if(&mut self) -> Result<Stmt, String> {
        self.advance(); // もし
        let condition = self.parse_expr()?;
        // なら を消費
        if matches!(self.peek().kind, TokenType::Then) {
            self.advance();
        }
        if matches!(self.peek().kind, TokenType::Colon) {
            self.advance();
        }

        let then_branch = self.parse_block()?;
        let mut elif_branches = Vec::new();
        let mut else_branch = None;

        while matches!(self.peek().kind, TokenType::Elif) {
            self.advance();
            let elif_cond = self.parse_expr()?;
            if matches!(self.peek().kind, TokenType::Then) {
                self.advance();
            }
            if matches!(self.peek().kind, TokenType::Colon) {
                self.advance();
            }
            let body = self.parse_block()?;
            elif_branches.push((elif_cond, body));
        }

        if matches!(self.peek().kind, TokenType::Else) {
            self.advance();
            if matches!(self.peek().kind, TokenType::Colon) {
                self.advance();
            }
            else_branch = Some(self.parse_block()?);
        }

        Ok(Stmt::If {
            condition,
            then_branch,
            elif_branches,
            else_branch,
        })
    }

    /// `N 回 繰り返す：` または `回 N 繰り返す：`
    /// For トークン = "回" が先頭か、expr の後に For が続くパターン
    fn parse_for_loop(&mut self) -> Result<Stmt, String> {
        self.advance(); // 回 (For)

        // 回 の後に数式がある場合: `回 N 繰り返す：`
        if !matches!(self.peek().kind, TokenType::Repeat | TokenType::Colon | TokenType::EOF) {
            let count = self.parse_expr()?;
            // 繰り返す
            if matches!(self.peek().kind, TokenType::Repeat) {
                self.advance();
            }
            if matches!(self.peek().kind, TokenType::Colon) {
                self.advance();
            }
            let body = self.parse_block()?;
            return Ok(Stmt::ForLoop { count, body });
        }

        // `回 N 繰り返す：` のパターン（回だけ先頭にある）は自然には来ない
        Err(format!("{}行目: 繰り返しの回数を指定してください (例: 3回 繰り返す：)", self.line()))
    }

    /// `Xの間 繰り返す：` - While(の間) トークンで開始
    /// 実際には `condition の間 繰り返す：` なので、
    /// parse_statement が expression の結果 While トークンに当たった場合に呼ばれるが、
    /// 普通は expr_or_assignment 経由で expression parsing される。
    /// ここはパーサーが While で開始する場合のハンドリング。
    fn parse_while_loop(&mut self) -> Result<Stmt, String> {
        // 「の間」が先頭 → 前の式が条件式としてパース済みのはずだが、
        // ここでは condition だけパースして Repeat を待つ
        // Note: 実際の文法は `Xの間繰り返す：` だが、
        // lexer が `の間` を While トークンにするので、
        // `parse_statement` が TokenType::While でここに来る。
        // しかし condition が「の間」の前にあるため、ここでは対応できない。
        // 対策: while は parse_expr_or_assignment 側で処理する
        Err(format!("{}行目: 'の間'は条件式の後に使用してください (例: X > 0の間繰り返す：)", self.line()))
    }

    fn parse_return(&mut self) -> Result<Stmt, String> {
        self.advance();
        let expr = self.parse_expr()?;
        Ok(Stmt::Return(expr))
    }

    fn parse_expr_or_assignment(&mut self) -> Result<Stmt, String> {
        let expr = self.parse_expr()?;

        // 代入チェック: `name = value`
        if matches!(self.peek().kind, TokenType::Equal) {
            if let Expr::Variable(name) = expr {
                self.advance(); // =
                let value = self.parse_expr()?;
                return Ok(Stmt::Assignment { name, value });
            } else {
                return Err(format!("{}行目: 代入の左辺は変数である必要があります", self.line()));
            }
        }

        // `condition の間 繰り返す：` パターン
        if matches!(self.peek().kind, TokenType::While) {
            let condition = expr;
            self.advance(); // の間
            if matches!(self.peek().kind, TokenType::Repeat) {
                self.advance(); // 繰り返す
            }
            if matches!(self.peek().kind, TokenType::Colon) {
                self.advance(); // ：
            }
            let body = self.parse_block()?;
            return Ok(Stmt::WhileLoop { condition, body });
        }

        // `N 回 繰り返す：` パターン（expr の後に For が続く）
        if matches!(self.peek().kind, TokenType::For) {
            let count = expr;
            self.advance(); // 回
            if matches!(self.peek().kind, TokenType::Repeat) {
                self.advance(); // 繰り返す
            }
            if matches!(self.peek().kind, TokenType::Colon) {
                self.advance(); // ：
            }
            let body = self.parse_block()?;
            return Ok(Stmt::ForLoop { count, body });
        }

        Ok(Stmt::Expr(expr))
    }

    fn parse_block(&mut self) -> Result<Vec<Stmt>, String> {
        // Newline をスキップして Indent を探す
        while matches!(self.peek().kind, TokenType::Newline) {
            self.advance();
        }
        if matches!(self.peek().kind, TokenType::Indent) {
            self.advance();
        } else {
            return Ok(Vec::new());
        }

        let mut statements = Vec::new();
        while !matches!(self.peek().kind, TokenType::Dedent | TokenType::EOF) {
            if matches!(self.peek().kind, TokenType::Newline) {
                self.advance();
                continue;
            }
            statements.push(self.parse_statement()?);
        }
        if matches!(self.peek().kind, TokenType::Dedent) {
            self.advance();
        }
        Ok(statements)
    }

    // --- Expression parsing (演算子優先順位) ---

    fn parse_expr(&mut self) -> Result<Expr, String> {
        self.parse_logical_or()
    }

    fn parse_logical_or(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_logical_and()?;
        while matches!(self.peek().kind, TokenType::Or) {
            self.advance();
            let right = self.parse_logical_and()?;
            expr = Expr::BinaryOp { left: Box::new(expr), op: BinaryOperator::Or, right: Box::new(right) };
        }
        Ok(expr)
    }

    fn parse_logical_and(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_equality()?;
        while matches!(self.peek().kind, TokenType::And) {
            self.advance();
            let right = self.parse_equality()?;
            expr = Expr::BinaryOp { left: Box::new(expr), op: BinaryOperator::And, right: Box::new(right) };
        }
        Ok(expr)
    }

    fn parse_equality(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_comparison()?;
        loop {
            let op = match self.peek().kind {
                TokenType::EqualEqual => Some(BinaryOperator::Equal),
                TokenType::NotEqual => Some(BinaryOperator::NotEqual),
                _ => None,
            };
            if let Some(operator) = op {
                self.advance();
                let right = self.parse_comparison()?;
                expr = Expr::BinaryOp { left: Box::new(expr), op: operator, right: Box::new(right) };
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn parse_comparison(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_term()?;
        loop {
            let op = match self.peek().kind {
                TokenType::Less => Some(BinaryOperator::Less),
                TokenType::LessEqual => Some(BinaryOperator::LessEqual),
                TokenType::Greater => Some(BinaryOperator::Greater),
                TokenType::GreaterEqual => Some(BinaryOperator::GreaterEqual),
                _ => None,
            };
            if let Some(operator) = op {
                self.advance();
                let right = self.parse_term()?;
                expr = Expr::BinaryOp { left: Box::new(expr), op: operator, right: Box::new(right) };
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn parse_term(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_factor()?;
        loop {
            let op = match self.peek().kind {
                TokenType::Plus => Some(BinaryOperator::Add),
                TokenType::Minus => Some(BinaryOperator::Sub),
                _ => None,
            };
            if let Some(operator) = op {
                self.advance();
                let right = self.parse_factor()?;
                expr = Expr::BinaryOp { left: Box::new(expr), op: operator, right: Box::new(right) };
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn parse_factor(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_unary()?;
        loop {
            let op = match self.peek().kind {
                TokenType::Multiply => Some(BinaryOperator::Mul),
                TokenType::Divide => Some(BinaryOperator::Div),
                TokenType::Modulo => Some(BinaryOperator::Mod),
                _ => None,
            };
            if let Some(operator) = op {
                self.advance();
                let right = self.parse_unary()?;
                expr = Expr::BinaryOp { left: Box::new(expr), op: operator, right: Box::new(right) };
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn parse_unary(&mut self) -> Result<Expr, String> {
        if matches!(self.peek().kind, TokenType::Minus) {
            self.advance();
            let expr = self.parse_unary()?;
            return Ok(Expr::UnaryOp { op: UnaryOperator::Neg, expr: Box::new(expr) });
        }
        if matches!(self.peek().kind, TokenType::Not) {
            self.advance();
            let expr = self.parse_unary()?;
            return Ok(Expr::UnaryOp { op: UnaryOperator::Not, expr: Box::new(expr) });
        }
        self.parse_postfix()
    }

    fn parse_postfix(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_primary()?;

        loop {
            match &self.peek().kind {
                TokenType::Of => {
                    // プロパティアクセス: Xの文字数, Xの0番目
                    self.advance();
                    match &self.peek().kind {
                        TokenType::Identifier(prop_name) => {
                            let prop = match prop_name.as_str() {
                                "文字数" | "要素数" => Property::Length,
                                s if s.ends_with("番目") => {
                                    let num_str = &s[..s.len() - 2];
                                    num_str.parse::<f64>()
                                        .map(Property::Index)
                                        .map_err(|_| format!("{}行目: 無効なインデックス '{}'", self.line(), s))?
                                }
                                _ => return Err(format!("{}行目: 無効なプロパティ '{}'", self.line(), prop_name)),
                            };
                            self.advance();
                            expr = Expr::PropertyAccess { object: Box::new(expr), property: prop };
                        }
                        TokenType::Number(n) => {
                            // `Xの0番目` → Number(0) + Identifier("番目")
                            let idx = *n;
                            self.advance();
                            if matches!(&self.peek().kind, TokenType::Identifier(s) if s == "番目") {
                                self.advance();
                            }
                            expr = Expr::PropertyAccess { object: Box::new(expr), property: Property::Index(idx) };
                        }
                        _ => return Err(format!("{}行目: 'の'の後にプロパティ名が必要です", self.line())),
                    }
                }
                TokenType::LeftParen => {
                    // 関数呼び出し: X(args)
                    if let Expr::Variable(name) = expr.clone() {
                        self.advance();
                        let args = self.parse_call_args()?;
                        expr = Expr::FunctionCall { name, args };
                    } else {
                        break;
                    }
                }
                _ => {
                    // `表示 X` (括弧なしの関数呼び出し)
                    if let Expr::Variable(ref name) = expr {
                        if name == "表示" && !matches!(self.peek().kind, TokenType::Newline | TokenType::EOF | TokenType::Colon | TokenType::Dedent) {
                            let arg = self.parse_expr()?;
                            expr = Expr::FunctionCall { name: "表示".to_string(), args: vec![arg] };
                            continue;
                        }
                    }
                    break;
                }
            }
        }

        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expr, String> {
        let token = self.peek().clone();
        match &token.kind {
            TokenType::Number(n) => { self.advance(); Ok(Expr::Number(*n)) }
            TokenType::String(s) => { self.advance(); Ok(Expr::String(s.clone())) }
            TokenType::True => { self.advance(); Ok(Expr::Boolean(true)) }
            TokenType::False => { self.advance(); Ok(Expr::Boolean(false)) }
            TokenType::Null => { self.advance(); Ok(Expr::Null) }
            TokenType::LeftBracket => {
                self.advance();
                self.parse_array_literal()
            }
            TokenType::LeftParen => {
                self.advance();
                let expr = self.parse_expr()?;
                if matches!(self.peek().kind, TokenType::RightParen) {
                    self.advance();
                    Ok(expr)
                } else {
                    Err(format!("{}行目: ')'が必要です", self.line()))
                }
            }
            TokenType::Identifier(s) => { self.advance(); Ok(Expr::Variable(s.clone())) }
            TokenType::Print => { self.advance(); Ok(Expr::Variable("表示".to_string())) }
            TokenType::LoopCount => { self.advance(); Ok(Expr::Variable("回数".to_string())) }
            _ => Err(format!("{}行目: 予期しないトークン: {}", self.line(), token.kind)),
        }
    }

    fn parse_array_literal(&mut self) -> Result<Expr, String> {
        let mut elements = Vec::new();
        while !matches!(self.peek().kind, TokenType::RightBracket | TokenType::EOF) {
            elements.push(self.parse_expr()?);
            if matches!(self.peek().kind, TokenType::Comma) {
                self.advance();
            }
        }
        if matches!(self.peek().kind, TokenType::RightBracket) {
            self.advance();
        }
        Ok(Expr::Array(elements))
    }

    fn parse_call_args(&mut self) -> Result<Vec<Expr>, String> {
        let mut args = Vec::new();
        while !matches!(self.peek().kind, TokenType::RightParen | TokenType::EOF) {
            if matches!(self.peek().kind, TokenType::Comma) {
                self.advance();
                continue;
            }
            args.push(self.parse_expr()?);
        }
        if matches!(self.peek().kind, TokenType::RightParen) {
            self.advance();
        }
        Ok(args)
    }

    // --- Helpers ---

    fn peek(&self) -> &Token {
        self.tokens.get(self.current).unwrap_or(self.tokens.last().unwrap())
    }

    fn advance(&mut self) -> &Token {
        if self.current < self.tokens.len() {
            self.current += 1;
        }
        &self.tokens[self.current - 1]
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len() || matches!(self.tokens[self.current].kind, TokenType::EOF)
    }

    fn line(&self) -> usize {
        self.peek().line
    }
}
