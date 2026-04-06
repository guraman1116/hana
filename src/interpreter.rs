use crate::ast::*;
use std::collections::HashMap;

pub struct Interpreter {
    globals: HashMap<String, Value>,
    locals: Vec<HashMap<String, Value>>,
    loop_count: usize,
    in_loop: bool,
}

impl Interpreter {
    pub fn new() -> Self {
        let mut globals = HashMap::new();
        Self::register_builtins(&mut globals);
        Interpreter {
            globals,
            locals: Vec::new(),
            loop_count: 0,
            in_loop: false,
        }
    }

    fn register_builtins(globals: &mut HashMap<String, Value>) {
        let make_func = |params: Vec<String>| Value::Function {
            params,
            body: Vec::new(),
            closure: HashMap::new(),
        };

        globals.insert("表示".to_string(), make_func(vec!["value".to_string()]));
        globals.insert("聞く".to_string(), make_func(vec!["prompt".to_string()]));
        globals.insert("文字数".to_string(), make_func(vec!["str".to_string()]));
        globals.insert("要素数".to_string(), make_func(vec!["arr".to_string()]));
        globals.insert("数にする".to_string(), make_func(vec!["value".to_string()]));
        globals.insert("文字にする".to_string(), make_func(vec!["value".to_string()]));
        globals.insert("繋ぐ".to_string(), make_func(vec!["a".to_string(), "b".to_string()]));
        globals.insert("切り取る".to_string(), make_func(vec!["str".to_string(), "start".to_string(), "end".to_string()]));
        globals.insert("絶対値".to_string(), make_func(vec!["n".to_string()]));
        globals.insert("四捨五入".to_string(), make_func(vec!["n".to_string()]));
        globals.insert("切り上げ".to_string(), make_func(vec!["n".to_string()]));
        globals.insert("切り下げ".to_string(), make_func(vec!["n".to_string()]));
        globals.insert("乱数".to_string(), make_func(vec!["min".to_string(), "max".to_string()]));
        globals.insert("追加".to_string(), make_func(vec!["arr".to_string(), "value".to_string()]));
    }

    pub fn interpret(&mut self, program: &Program) -> Result<Value, String> {
        let mut result = Value::Null;
        for stmt in program {
            result = self.execute_statement(stmt)?;
        }
        Ok(result)
    }

    fn execute_statement(&mut self, stmt: &Stmt) -> Result<Value, String> {
        match stmt {
            Stmt::Expr(expr) => self.eval_expr(expr),
            Stmt::Assignment { name, value } => {
                let val = self.eval_expr(value)?;
                self.set_variable(name, val);
                Ok(Value::Null)
            }
            Stmt::If { condition, then_branch, elif_branches, else_branch } => {
                let cond = self.eval_expr(condition)?;
                if self.is_truthy(&cond) {
                    self.execute_block(then_branch)?;
                } else {
                    let mut executed = false;
                    for (elif_cond, elif_body) in elif_branches {
                        let elif_cond_val = self.eval_expr(elif_cond)?;
                        if self.is_truthy(&elif_cond_val) {
                            self.execute_block(elif_body)?;
                            executed = true;
                            break;
                        }
                    }
                    if !executed {
                        if let Some(else_body) = else_branch {
                            self.execute_block(else_body)?;
                        }
                    }
                }
                Ok(Value::Null)
            }
            Stmt::ForLoop { count, body } => {
                let count_val = self.eval_expr(count)?;
                if let Value::Number(n) = count_val {
                    let iterations = n as i64;
                    if iterations > 10000 {
                        return Err("繰り返しが10000回を超えました。間違っていませんか？".to_string());
                    }
                    let was_in_loop = self.in_loop;
                    self.in_loop = true;
                    for i in 0..iterations {
                        self.loop_count = i as usize;
                        self.execute_block(body)?;
                    }
                    self.in_loop = was_in_loop;
                    Ok(Value::Null)
                } else {
                    Err("繰り返し回数は数値である必要があります".to_string())
                }
            }
            Stmt::WhileLoop { condition, body } => {
                let was_in_loop = self.in_loop;
                self.in_loop = true;
                self.loop_count = 0;
                let mut iterations = 0;
                loop {
                    let cond_val = self.eval_expr(condition)?;
                    if !self.is_truthy(&cond_val) {
                        break;
                    }
                    iterations += 1;
                    if iterations > 10000 {
                        self.in_loop = was_in_loop;
                        return Err("繰り返しが10000回を超えました。間違っていませんか？".to_string());
                    }
                    self.execute_block(body)?;
                    self.loop_count += 1;
                }
                self.in_loop = was_in_loop;
                Ok(Value::Null)
            }
            Stmt::FunctionDef { name, params, body } => {
                let func = Value::Function {
                    params: params.clone(),
                    body: body.clone(),
                    closure: HashMap::new(),
                };
                self.globals.insert(name.clone(), func);
                Ok(Value::Null)
            }
            Stmt::Return(_) => Err("戻すは関数内でのみ使用できます".to_string()),
        }
    }

    fn execute_block(&mut self, statements: &[Stmt]) -> Result<Value, String> {
        self.locals.push(HashMap::new());
        let mut result = Value::Null;
        for stmt in statements {
            if let Stmt::Return(expr) = stmt {
                result = self.eval_expr(expr)?;
                self.locals.pop();
                return Ok(result);
            }
            result = self.execute_statement(stmt)?;
        }
        self.locals.pop();
        Ok(result)
    }

    fn eval_expr(&mut self, expr: &Expr) -> Result<Value, String> {
        match expr {
            Expr::Number(n) => Ok(Value::Number(*n)),
            Expr::String(s) => Ok(Value::String(self.interpolate_string_mut(s)?)),
            Expr::Boolean(b) => Ok(Value::Boolean(*b)),
            Expr::Null => Ok(Value::Null),
            Expr::Array(elements) => {
                let mut arr = Vec::new();
                for elem in elements {
                    arr.push(self.eval_expr(elem)?);
                }
                Ok(Value::Array(arr))
            }
            Expr::Variable(name) => {
                if name == "回数" {
                    if !self.in_loop {
                        return Err("回数は繰り返し内でのみ使用できます".to_string());
                    }
                    return Ok(Value::Number(self.loop_count as f64));
                }
                self.get_variable(name)
            }
            Expr::BinaryOp { left, op, right } => {
                let l = self.eval_expr(left)?;
                let r = self.eval_expr(right)?;
                self.eval_binary_op(l, *op, r)
            }
            Expr::UnaryOp { op, expr } => {
                let v = self.eval_expr(expr)?;
                self.eval_unary_op(*op, v)
            }
            Expr::PropertyAccess { object, property } => {
                let obj = self.eval_expr(object)?;
                self.eval_property_access(obj, property)
            }
            Expr::FunctionCall { name, args } => {
                let func = self.get_variable(name)?;
                let mut evaluated_args = Vec::new();
                for arg in args {
                    evaluated_args.push(self.eval_expr(arg)?);
                }
                self.call_function(name.clone(), func, evaluated_args)
            }
            Expr::Index { array, index } => {
                let arr = self.eval_expr(array)?;
                let idx = self.eval_expr(index)?;
                self.eval_index(arr, idx)
            }
        }
    }

    fn interpolate_string_mut(&mut self, s: &str) -> Result<String, String> {
        let mut result = String::new();
        let mut chars = s.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '{' {
                let mut expr_str = String::new();
                let mut brace_depth = 1;

                while let Some(&next_ch) = chars.peek() {
                    chars.next();
                    if next_ch == '}' {
                        brace_depth -= 1;
                        if brace_depth == 0 {
                            break;
                        }
                    } else if next_ch == '{' {
                        brace_depth += 1;
                    }
                    expr_str.push(next_ch);
                }

                if !expr_str.is_empty() {
                    if let Ok(expr) = self.eval_expr_from_string(&expr_str) {
                        result.push_str(&self.value_to_string(&expr));
                    } else {
                        result.push('{');
                        result.push_str(&expr_str);
                        result.push('}');
                    }
                }
            } else {
                result.push(ch);
            }
        }

        Ok(result)
    }

    fn eval_expr_from_string(&mut self, s: &str) -> Result<Value, String> {
        use crate::lexer::Lexer;
        use crate::parser::Parser;
        let mut lexer = Lexer::new(s);
        let tokens = lexer.tokenize()?;
        let mut parser = Parser::new(tokens);
        let mut program = parser.parse()?;
        if program.len() == 1 {
            if let Stmt::Expr(expr) = program.remove(0) {
                return self.eval_expr(&expr);
            }
        }
        Err(format!("補間式の解析に失敗: {}", s))
    }

    fn eval_binary_op(&self, left: Value, op: BinaryOperator, right: Value) -> Result<Value, String> {
        match (left, right) {
            (Value::Number(l), Value::Number(r)) => match op {
                BinaryOperator::Add => Ok(Value::Number(l + r)),
                BinaryOperator::Sub => Ok(Value::Number(l - r)),
                BinaryOperator::Mul => Ok(Value::Number(l * r)),
                BinaryOperator::Div => {
                    if r == 0.0 { Err("0で割ることはできません".to_string()) }
                    else { Ok(Value::Number(l / r)) }
                }
                BinaryOperator::Mod => {
                    if r == 0.0 { Err("0で割ることはできません".to_string()) }
                    else { Ok(Value::Number(l % r)) }
                }
                BinaryOperator::Equal => Ok(Value::Boolean((l - r).abs() < f64::EPSILON)),
                BinaryOperator::NotEqual => Ok(Value::Boolean((l - r).abs() >= f64::EPSILON)),
                BinaryOperator::Less => Ok(Value::Boolean(l < r)),
                BinaryOperator::LessEqual => Ok(Value::Boolean(l <= r)),
                BinaryOperator::Greater => Ok(Value::Boolean(l > r)),
                BinaryOperator::GreaterEqual => Ok(Value::Boolean(l >= r)),
                BinaryOperator::And => Ok(Value::Boolean(l != 0.0 && r != 0.0)),
                BinaryOperator::Or => Ok(Value::Boolean(l != 0.0 || r != 0.0)),
            },
            (Value::String(l), Value::String(r)) => match op {
                BinaryOperator::Add => Ok(Value::String(format!("{}{}", l, r))),
                BinaryOperator::Equal => Ok(Value::Boolean(l == r)),
                BinaryOperator::NotEqual => Ok(Value::Boolean(l != r)),
                _ => Err(format!("文字列に対して{}演算は使用できません", op_to_string(op))),
            },
            (l, r) => match op {
                BinaryOperator::Equal => Ok(Value::Boolean(self.value_eq(&l, &r))),
                BinaryOperator::NotEqual => Ok(Value::Boolean(!self.value_eq(&l, &r))),
                _ => Err(format!("型の不一致: {} と {} の間で {} 演算は使用できません", l.type_name(), r.type_name(), op_to_string(op))),
            },
        }
    }

    fn eval_unary_op(&self, op: UnaryOperator, value: Value) -> Result<Value, String> {
        match op {
            UnaryOperator::Neg => {
                if let Value::Number(n) = value {
                    Ok(Value::Number(-n))
                } else {
                    Err(format!("「{}」から符号を反転することはできません。数値を使ってください。", self.value_to_string(&value)))
                }
            }
            UnaryOperator::Not => Ok(Value::Boolean(!self.is_truthy(&value))),
        }
    }

    fn eval_property_access(&self, obj: Value, prop: &Property) -> Result<Value, String> {
        match prop {
            Property::Length => match obj {
                Value::String(s) => Ok(Value::Number(s.chars().count() as f64)),
                Value::Array(arr) => Ok(Value::Number(arr.len() as f64)),
                _ => Err(format!("{}には文字数や要素数を取得できません", obj.type_name())),
            },
            Property::Index(n) => match obj {
                Value::Array(arr) => {
                    let idx = *n as i64;
                    if idx < 0 || idx >= arr.len() as i64 {
                        Err("配列の範囲外アクセスです".to_string())
                    } else {
                        Ok(arr[idx as usize].clone())
                    }
                }
                Value::String(s) => {
                    let idx = *n as i64;
                    if idx < 0 || idx >= s.chars().count() as i64 {
                        Err("文字列の範囲外アクセスです".to_string())
                    } else {
                        Ok(Value::String(s.chars().nth(idx as usize).unwrap().to_string()))
                    }
                }
                _ => Err(format!("{}からインデックスでアクセスできません", obj.type_name())),
            },
        }
    }

    fn eval_index(&self, arr: Value, idx: Value) -> Result<Value, String> {
        match (arr, idx) {
            (Value::Array(a), Value::Number(n)) => {
                let i = n as i64;
                if i < 0 || i >= a.len() as i64 {
                    Err("配列の範囲外アクセスです".to_string())
                } else {
                    Ok(a[i as usize].clone())
                }
            }
            (Value::String(s), Value::Number(n)) => {
                let i = n as i64;
                if i < 0 || i >= s.chars().count() as i64 {
                    Err("文字列の範囲外アクセスです".to_string())
                } else {
                    Ok(Value::String(s.chars().nth(i as usize).unwrap().to_string()))
                }
            }
            _ => Err("配列インデックスは数値である必要があります".to_string()),
        }
    }

    fn call_function(&mut self, name: String, func: Value, args: Vec<Value>) -> Result<Value, String> {
        match func {
            Value::Function { params, body, closure } => {
                if name == "表示" {
                    for arg in args {
                        print!("{}", self.value_to_string(&arg));
                    }
                    println!();
                    Ok(Value::Null)
                } else if name == "聞く" {
                    let prompt = args.get(0).map(|a| self.value_to_string(a)).unwrap_or_default();
                    print!("{}", prompt);
                    std::io::Write::flush(&mut std::io::stdout()).unwrap();
                    let mut input = String::new();
                    std::io::stdin().read_line(&mut input).unwrap();
                    Ok(Value::String(input.trim().to_string()))
                } else if name == "文字数" {
                    if let Value::String(s) = &args[0] {
                        Ok(Value::Number(s.chars().count() as f64))
                    } else {
                        Err("文字数には文字列を渡してください".to_string())
                    }
                } else if name == "要素数" {
                    if let Value::Array(arr) = &args[0] {
                        Ok(Value::Number(arr.len() as f64))
                    } else {
                        Err("要素数には配列を渡してください".to_string())
                    }
                } else if name == "数にする" {
                    match &args[0] {
                        Value::String(s) => s.parse::<f64>().map(Value::Number)
                            .map_err(|_| format!("「{}」を数値に変換できません", s)),
                        Value::Number(n) => Ok(Value::Number(*n)),
                        Value::Boolean(b) => Ok(Value::Number(if *b { 1.0 } else { 0.0 })),
                        _ => Err(format!("{}を数値に変換できません", args[0].type_name())),
                    }
                } else if name == "文字にする" {
                    Ok(Value::String(self.value_to_string(&args[0])))
                } else if name == "繋ぐ" {
                    let result = args.iter().map(|a| self.value_to_string(a)).collect::<String>();
                    Ok(Value::String(result))
                } else if name == "絶対値" {
                    if let Value::Number(n) = args[0] {
                        Ok(Value::Number(n.abs()))
                    } else {
                        Err("絶対値には数値を渡してください".to_string())
                    }
                } else if name == "四捨五入" {
                    if let Value::Number(n) = args[0] {
                        Ok(Value::Number(n.round()))
                    } else {
                        Err("四捨五入には数値を渡してください".to_string())
                    }
                } else if name == "切り上げ" {
                    if let Value::Number(n) = args[0] {
                        Ok(Value::Number(n.ceil()))
                    } else {
                        Err("切り上げには数値を渡してください".to_string())
                    }
                } else if name == "切り下げ" {
                    if let Value::Number(n) = args[0] {
                        Ok(Value::Number(n.floor()))
                    } else {
                        Err("切り下げには数値を渡してください".to_string())
                    }
                } else if name == "乱数" {
                    let min = if let Value::Number(n) = args[0] { n as i64 } else { return Err("最小値は数値である必要があります".to_string()) };
                    let max = if let Value::Number(n) = args[1] { n as i64 } else { return Err("最大値は数値である必要があります".to_string()) };
                    use std::time::{SystemTime, UNIX_EPOCH};
                    let seed = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as u64;
                    let mut rng = seed.wrapping_mul(1103515245).wrapping_add(12345);
                    let random = ((rng % ((max - min + 1) as u64)) as i64) + min;
                    Ok(Value::Number(random as f64))
                } else if name == "追加" {
                    // 追加は変数を直接変更する必要があるため、第1引数の変数名も受け取る
                    // 使い方: 追加(配列, 値) → 配列に値を追加して返す
                    if let Value::Array(mut a) = args[0].clone() {
                        a.push(args[1].clone());
                        // 呼び出し元で代入が必要: 配列 = 追加(配列, 値)
                        // または自動代入をサポート
                        Ok(Value::Array(a))
                    } else {
                        Err("追加の第1引数は配列である必要があります".to_string())
                    }
                } else {
                    if args.len() != params.len() {
                        return Err(format!("{}には{}個の引数が必要ですが{}個渡されました", name, params.len(), args.len()));
                    }
                    self.locals.push(closure);
                    for (param, arg) in params.iter().zip(args.iter()) {
                        self.locals.last_mut().unwrap().insert(param.clone(), arg.clone());
                    }
                    let mut result = Value::Null;
                    for stmt in &body {
                        if let Stmt::Return(expr) = stmt {
                            result = self.eval_expr(expr)?;
                            break;
                        }
                        result = self.execute_statement(stmt)?;
                    }
                    self.locals.pop();
                    Ok(result)
                }
            }
            _ => Err(format!("{}は関数ではありません", name)),
        }
    }

    fn get_variable(&self, name: &str) -> Result<Value, String> {
        for scope in self.locals.iter().rev() {
            if let Some(val) = scope.get(name) {
                return Ok(val.clone());
            }
        }
        if let Some(val) = self.globals.get(name) {
            return Ok(val.clone());
        }
        Err(format!("「{}」という変数はまだ作られていません。「{}」の入力ミスかもしれません。", name, name))
    }

    fn set_variable(&mut self, name: &str, value: Value) {
        for scope in self.locals.iter_mut().rev() {
            if scope.contains_key(name) {
                scope.insert(name.to_string(), value);
                return;
            }
        }
        self.globals.insert(name.to_string(), value);
    }

    fn is_truthy(&self, value: &Value) -> bool {
        match value {
            Value::Boolean(b) => *b,
            Value::Number(n) => *n != 0.0,
            Value::String(s) => !s.is_empty(),
            Value::Array(arr) => !arr.is_empty(),
            Value::Null => false,
            Value::Function { .. } => true,
        }
    }

    fn value_eq(&self, a: &Value, b: &Value) -> bool {
        match (a, b) {
            (Value::Number(l), Value::Number(r)) => (l - r).abs() < f64::EPSILON,
            (Value::String(l), Value::String(r)) => l == r,
            (Value::Boolean(l), Value::Boolean(r)) => l == r,
            (Value::Null, Value::Null) => true,
            _ => false,
        }
    }

    fn value_to_string(&self, value: &Value) -> String {
        match value {
            Value::Number(n) => {
                if n.fract() == 0.0 { format!("{}", *n as i64) } else { format!("{}", n) }
            }
            Value::String(s) => s.clone(),
            Value::Boolean(b) => if *b { "真".to_string() } else { "偽".to_string() },
            Value::Array(arr) => {
                let items: Vec<String> = arr.iter().map(|v| self.value_to_string(v)).collect();
                format!("[{}]", items.join(", "))
            }
            Value::Null => "なし".to_string(),
            Value::Function { .. } => "<関数>".to_string(),
        }
    }
}

fn op_to_string(op: BinaryOperator) -> String {
    match op {
        BinaryOperator::Add => "+".to_string(),
        BinaryOperator::Sub => "-".to_string(),
        BinaryOperator::Mul => "*".to_string(),
        BinaryOperator::Div => "/".to_string(),
        BinaryOperator::Mod => "%".to_string(),
        BinaryOperator::Equal => "==".to_string(),
        BinaryOperator::NotEqual => "!=".to_string(),
        BinaryOperator::Less => "<".to_string(),
        BinaryOperator::LessEqual => "<=".to_string(),
        BinaryOperator::Greater => ">".to_string(),
        BinaryOperator::GreaterEqual => ">=".to_string(),
        BinaryOperator::And => "そして".to_string(),
        BinaryOperator::Or => "または".to_string(),
    }
}
