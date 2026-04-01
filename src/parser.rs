use crate::token::{Token, TokenType};
use crate::ast::*;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens,
            current: 0,
        }
    }

    pub fn parse(&mut self) -> Result<Program, String> {
        let mut statements = Vec::new();

        while !self.is_at_end() {
            if let TokenType::EOF = self.peek().kind {
                break;
            }
            if let TokenType::Newline = self.peek().kind {
                self.advance();
                continue;
            }
            if let TokenType::Dedent = self.peek().kind {
                self.advance();
                continue;
            }

            statements.push(self.parse_statement()?);
        }

        Ok(statements)
    }

    fn parse_statement(&mut self) -> Result<Stmt, String> {
        let token = self.peek().clone();

        match &token.kind {
            TokenType::Identifier(_) if self.check_next(TokenType::Function) => {
                self.parse_function_def()
            }
            TokenType::If => self.parse_if(),
            TokenType::For => self.parse_for(),
            TokenType::While => self.parse_while(),
            TokenType::Return => self.parse_return(),
            _ => {
                let expr = self.parse_expr()?;

                if let TokenType::Equal = self.peek().kind {
                    self.advance();
                    if let Expr::Variable(name) = expr {
                        let value = self.parse_expr()?;
                        Ok(Stmt::Assignment { name, value })
                    } else {
                        Err(format!(
                            "{}行目: 代入の左辺は変数である必要があります",
                            token.line
                        ))
                    }
                } else {
                    Ok(Stmt::Expr(expr))
                }
            }
        }
    }

    fn parse_function_def(&mut self) -> Result<Stmt, String> {
        let name = if let TokenType::Identifier(s) = &self.peek().kind {
            s.clone()
        } else {
            return Err(format!("{}行目: 関数名が必要です", self.peek().line));
        };
        self.advance();

        let mut params = Vec::new();
        if let TokenType::LeftParen = self.peek().kind {
            self.advance();
            while let TokenType::Identifier(s) = &self.peek().kind {
                params.push(s.clone());
                self.advance();
                if let TokenType::Comma = self.peek().kind {
                    self.advance();
                }
            }
            if let TokenType::RightParen = self.peek().kind {
                self.advance();
            } else {
                return Err(format!("{}行目: ')'が必要です", self.peek().line));
            }
        }

        if let TokenType::Function = self.peek().kind {
            self.advance();
        } else {
            return Err(format!("{}行目: 'とは'が必要です", self.peek().line));
        }

        if let TokenType::Colon = self.peek().kind {
            self.advance();
        } else {
            return Err(format!("{}行目: ':'が必要です", self.peek().line));
        }

        let body = self.parse_block()?;

        Ok(Stmt::FunctionDef { name, params, body })
    }

    fn parse_if(&mut self) -> Result<Stmt, String> {
        self.advance();

        let condition = self.parse_expr()?;

        if let TokenType::Colon = self.peek().kind {
            self.advance();
        } else {
            return Err(format!("{}行目: ':'が必要です", self.peek().line));
        }

        let then_branch = self.parse_block()?;
        let mut elif_branches = Vec::new();
        let mut else_branch = None;

        while let TokenType::Elif = self.peek().kind {
            self.advance();
            let condition = self.parse_expr()?;
            if let TokenType::Colon = self.peek().kind {
                self.advance();
            } else {
                return Err(format!("{}行目: ':'が必要です", self.peek().line));
            }
            let body = self.parse_block()?;
            elif_branches.push((condition, body));
        }

        if let TokenType::Else = self.peek().kind {
            self.advance();
            if let TokenType::Colon = self.peek().kind {
                self.advance();
            } else {
                return Err(format!("{}行目: ':'が必要です", self.peek().line));
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

    fn parse_for(&mut self) -> Result<Stmt, String> {
        self.advance();

        let count = self.parse_expr()?;

        if let TokenType::For = self.peek().kind {
            self.advance();
        } else {
            return Err(format!("{}行目: '繰り返す'が必要です", self.peek().line));
        }

        if let TokenType::Colon = self.peek().kind {
            self.advance();
        } else {
            return Err(format!("{}行目: ':'が必要です", self.peek().line));
        }

        let body = self.parse_block()?;

        Ok(Stmt::ForLoop { count, body })
    }

    fn parse_while(&mut self) -> Result<Stmt, String> {
        self.advance();

        if self.current > 0 {
            self.current -= 1;
        }

        let condition = self.parse_expr()?;

        if let TokenType::While = self.peek().kind {
            self.advance();
        } else {
            return Err(format!("{}行目: 'の間 繰り返す'が必要です", self.peek().line));
        }

        if let TokenType::Colon = self.peek().kind {
            self.advance();
        } else {
            return Err(format!("{}行目: ':'が必要です", self.peek().line));
        }

        let body = self.parse_block()?;

        Ok(Stmt::WhileLoop { condition, body })
    }

    fn parse_return(&mut self) -> Result<Stmt, String> {
        self.advance();
        let expr = self.parse_expr()?;
        Ok(Stmt::Return(expr))
    }

    fn parse_block(&mut self) -> Result<Vec<Stmt>, String> {
        if let TokenType::Indent = self.peek().kind {
            self.advance();
        } else {
            return Ok(Vec::new());
        }

        let mut statements = Vec::new();

        while !self.is_at_end() {
            if let TokenType::Dedent = self.peek().kind {
                self.advance();
                break;
            }
            if let TokenType::Newline = self.peek().kind {
                self.advance();
                continue;
            }
            if let TokenType::EOF = self.peek().kind {
                break;
            }

            statements.push(self.parse_statement()?);
        }

        Ok(statements)
    }

    fn parse_expr(&mut self) -> Result<Expr, String> {
        self.parse_logical_or()
    }

    fn parse_logical_or(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_logical_and()?;

        while let TokenType::Or = self.peek().kind {
            self.advance();
            let right = self.parse_logical_and()?;
            expr = Expr::BinaryOp {
                left: Box::new(expr),
                op: BinaryOperator::Or,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn parse_logical_and(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_equality()?;

        while let TokenType::And = self.peek().kind {
            self.advance();
            let right = self.parse_equality()?;
            expr = Expr::BinaryOp {
                left: Box::new(expr),
                op: BinaryOperator::And,
                right: Box::new(right),
            };
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
                expr = Expr::BinaryOp {
                    left: Box::new(expr),
                    op: operator,
                    right: Box::new(right),
                };
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
                expr = Expr::BinaryOp {
                    left: Box::new(expr),
                    op: operator,
                    right: Box::new(right),
                };
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
                expr = Expr::BinaryOp {
                    left: Box::new(expr),
                    op: operator,
                    right: Box::new(right),
                };
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
                expr = Expr::BinaryOp {
                    left: Box::new(expr),
                    op: operator,
                    right: Box::new(right),
                };
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn parse_unary(&mut self) -> Result<Expr, String> {
        if let TokenType::Minus = self.peek().kind {
            self.advance();
            let expr = self.parse_unary()?;
            return Ok(Expr::UnaryOp {
                op: UnaryOperator::Neg,
                expr: Box::new(expr),
            });
        }

        if let TokenType::Not = self.peek().kind {
            self.advance();
            let expr = self.parse_unary()?;
            return Ok(Expr::UnaryOp {
                op: UnaryOperator::Not,
                expr: Box::new(expr),
            });
        }

        self.parse_postfix()
    }

    fn parse_postfix(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_primary()?;

        loop {
            if let TokenType::Of = self.peek().kind {
                self.advance();

                if let TokenType::Identifier(prop_name) = &self.peek().kind {
                    let prop = match prop_name.as_str() {
                        "文字数" | "要素数" => Property::Length,
                        _ => {
                            if prop_name.ends_with("番目") {
                                let num_str = &prop_name[..prop_name.len() - 2];
                                if let Ok(n) = num_str.parse::<f64>() {
                                    Property::Index(n)
                                } else {
                                    return Err(format!(
                                        "{}行目: 無効なプロパティ '{}'",
                                        self.peek().line,
                                        prop_name
                                    ));
                                }
                            } else {
                                return Err(format!(
                                    "{}行目: 無効なプロパティ '{}'",
                                    self.peek().line,
                                    prop_name
                                ));
                            }
                        }
                    };
                    self.advance();
                    expr = Expr::PropertyAccess {
                        object: Box::new(expr),
                        property: prop,
                    };
                }
            }
            else if let TokenType::LeftParen = self.peek().kind {
                if let Expr::Variable(name) = expr {
                    self.advance();
                    let args = self.parse_args()?;
                    expr = Expr::FunctionCall { name, args };
                } else {
                    return Err(format!("{}行目: 関数呼び出しの構文が正しくありません", self.peek().line));
                }
            }
            else if let Expr::Variable(ref name) = expr {
                if name == "表示" {
                    if !matches!(self.peek().kind, TokenType::Newline | TokenType::EOF | TokenType::Colon) {
                        let arg = self.parse_expr()?;
                        expr = Expr::FunctionCall {
                            name: "表示".to_string(),
                            args: vec![arg],
                        };
                    }
                }
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expr, String> {
        let token = self.peek().clone();

        match &token.kind {
            TokenType::Number(n) => {
                self.advance();
                Ok(Expr::Number(*n))
            }
            TokenType::String(s) => {
                self.advance();
                Ok(Expr::String(s.clone()))
            }
            TokenType::True => {
                self.advance();
                Ok(Expr::Boolean(true))
            }
            TokenType::False => {
                self.advance();
                Ok(Expr::Boolean(false))
            }
            TokenType::Null => {
                self.advance();
                Ok(Expr::Null)
            }
            TokenType::LeftBracket => {
                self.advance();
                self.parse_array()
            }
            TokenType::LeftParen => {
                self.advance();
                let expr = self.parse_expr()?;
                let check_token = self.peek().clone();
                if let TokenType::RightParen = check_token.kind {
                    self.advance();
                    Ok(expr)
                } else {
                    Err(format!("{}行目: ')'が必要です", check_token.line))
                }
            }
            TokenType::Identifier(s) => {
                self.advance();
                Ok(Expr::Variable(s.clone()))
            }
            TokenType::Print => {
                self.advance();
                Ok(Expr::Variable("表示".to_string()))
            }
            _ => Err(format!("{}行目: 予期しないトークン: {:?}", token.line, token.kind)),
        }
    }

    fn parse_array(&mut self) -> Result<Expr, String> {
        let mut elements = Vec::new();

        while !self.is_at_end() {
            if let TokenType::RightBracket = self.peek().kind {
                self.advance();
                break;
            }

            elements.push(self.parse_expr()?);

            if let TokenType::Comma = self.peek().kind {
                self.advance();
            } else if let TokenType::RightBracket = self.peek().kind {
                self.advance();
                break;
            } else {
                return Err(format!(
                    "{}行目: ',' または ']' が必要です",
                    self.peek().line
                ));
            }
        }

        Ok(Expr::Array(elements))
    }

    fn parse_args(&mut self) -> Result<Vec<Expr>, String> {
        let mut args = Vec::new();

        while !self.is_at_end() {
            if let TokenType::RightParen = self.peek().kind {
                self.advance();
                break;
            }

            args.push(self.parse_expr()?);

            if let TokenType::Comma = self.peek().kind {
                self.advance();
            } else if let TokenType::RightParen = self.peek().kind {
                self.advance();
                break;
            } else {
                return Err(format!(
                    "{}行目: ',' または ')' が必要です",
                    self.peek().line
                ));
            }
        }

        Ok(args)
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn peek_n(&self, n: usize) -> Token {
        let idx = self.current + n;
        if idx < self.tokens.len() {
            self.tokens[idx].clone()
        } else {
            self.tokens.last().unwrap().clone()
        }
    }

    fn advance(&mut self) -> &Token {
        if self.current < self.tokens.len() {
            self.current += 1;
        }
        &self.tokens[self.current - 1]
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len()
            || matches!(self.tokens[self.current].kind, TokenType::EOF)
    }

    fn check_next(&self, kind: TokenType) -> bool {
        if self.current + 1 >= self.tokens.len() {
            return false;
        }
        self.tokens[self.current + 1].kind == kind
    }
}
