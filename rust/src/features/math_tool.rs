use crate::state::{AppState, MathHistoryEntry};
use crate::ui::{
    maybe_push_back, Button as UiButton, Column as UiColumn, Text as UiText,
    TextInput as UiTextInput, VirtualList as UiVirtualList,
};
use serde_json::Value;
use std::collections::HashMap;
use std::f64::consts::{E, PI};

pub fn render_math_tool_screen(state: &AppState) -> Value {
    let mut children = vec![
        serde_json::to_value(UiText::new("Math Expression Evaluator").size(20.0)).unwrap(),
        serde_json::to_value(
            UiText::new("Evaluate expressions with +, -, *, /, ^, parentheses, and functions: sin, cos, sqrt, log (base e).")
                .size(14.0),
        )
        .unwrap(),
        serde_json::to_value(
            UiTextInput::new("math_expr")
                .hint("e.g., sin(pi/2) + 3^2")
                .text(&state.math_tool.expression)
                .single_line(true)
                .debounce_ms(150),
        )
        .unwrap(),
        serde_json::to_value(UiButton::new("Calculate", "math_calculate")).unwrap(),
        serde_json::to_value(UiButton::new("Clear history", "math_clear_history")).unwrap(),
    ];

    if let Some(err) = &state.math_tool.error {
        children.push(
            serde_json::to_value(UiText::new(&format!("Error: {err}")).size(12.0)).unwrap(),
        );
    }

    if !state.math_tool.history.is_empty() {
        children.push(serde_json::to_value(UiText::new("History").size(16.0)).unwrap());
        let items: Vec<Value> = state
            .math_tool
            .history
            .iter()
            .map(|entry| {
                serde_json::to_value(UiText::new(&format!("{} = {}", entry.expression, entry.result)).size(12.0))
                    .unwrap()
            })
            .collect();
        children.push(serde_json::to_value(UiVirtualList::new(items).id("math_history")).unwrap());
    }

    maybe_push_back(&mut children, state);
    serde_json::to_value(UiColumn::new(children).padding(20)).unwrap()
}

pub fn handle_math_action(
    state: &mut AppState,
    action: &str,
    bindings: &HashMap<String, String>,
) {
    match action {
        "math_calculate" => {
            if let Some(input) = bindings.get("math_expr") {
                state.math_tool.expression = input.clone();
            }
            let expr = state.math_tool.expression.trim();
            if expr.is_empty() {
                state.math_tool.error = Some("expression_empty".into());
                return;
            }
            match evaluate_expression(expr) {
                Ok(value) => {
                    let result = format_result(value);
                    state.math_tool.error = None;
                    state.math_tool
                        .history
                        .insert(0, MathHistoryEntry { expression: expr.to_string(), result });
                    if state.math_tool.history.len() > 20 {
                        state.math_tool.history.truncate(20);
                    }
                }
                Err(e) => {
                    state.math_tool.error = Some(e);
                }
            }
        }
        "math_clear_history" => {
            state.math_tool.clear_history();
            state.math_tool.expression.clear();
            state.math_tool.error = None;
        }
        _ => {}
    }
}

pub fn evaluate_expression(expr: &str) -> Result<f64, String> {
    if let Some((inner, var)) = extract_integ_call(expr) {
        let ast = parse_symbolic(&inner)?;
        let integral = integrate(&ast, &var);
        let simplified = simplify(&integral);
        let rendered = render_symbol(&simplified);
        return Err(format!("symbolic_result:{rendered}"));
    }
    if let Some(inner) = extract_deriv_call(expr) {
        let ast = parse_symbolic(&inner)?;
        let deriv = differentiate(&ast, "x");
        let simplified = simplify(&deriv);
        let rendered = render_symbol(&simplified);
        // Represent derivative as NaN in numeric context; caller (UI) will display string.
        return Err(format!("symbolic_result:{rendered}"));
    }

    let tokens = tokenize(expr)?;
    let rpn = shunting_yard(&tokens)?;
    eval_rpn(&rpn)
}

fn extract_deriv_call(expr: &str) -> Option<String> {
    let trimmed = expr.trim();
    let lower = trimmed.to_ascii_lowercase();
    if !lower.starts_with("deriv") {
        return None;
    }
    let open = trimmed.find('(')?;
    let mut depth = 0i32;
    let mut end = None;
    for (idx, ch) in trimmed.char_indices().skip(open) {
        match ch {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    end = Some(idx);
                    break;
                }
            }
            _ => {}
        }
    }
    let end_idx = end?;
    let inner = trimmed.get(open + 1..end_idx)?;
    Some(inner.trim().to_string())
}

fn extract_integ_call(expr: &str) -> Option<(String, String)> {
    let trimmed = expr.trim();
    let lower = trimmed.to_ascii_lowercase();
    if !lower.starts_with("integ") {
        return None;
    }
    let open = trimmed.find('(')?;
    let mut depth = 0i32;
    let mut end = None;
    for (idx, ch) in trimmed.char_indices().skip(open) {
        match ch {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    end = Some(idx);
                    break;
                }
            }
            _ => {}
        }
    }
    let end_idx = end?;
    let inner = trimmed.get(open + 1..end_idx)?;
    let mut parts = inner.splitn(2, ',').map(|s| s.trim()).collect::<Vec<_>>();
    let var = if parts.len() == 2 && !parts[1].is_empty() {
        parts.pop().unwrap().to_string()
    } else {
        "x".to_string()
    };
    let expr_body = parts.first().cloned().unwrap_or("").to_string();
    if expr_body.is_empty() {
        None
    } else {
        Some((expr_body, var))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Symbol {
    Number(f64),
    Var(String),
    Add(Box<Symbol>, Box<Symbol>),
    Sub(Box<Symbol>, Box<Symbol>),
    Mul(Box<Symbol>, Box<Symbol>),
    Div(Box<Symbol>, Box<Symbol>),
    Pow(Box<Symbol>, Box<Symbol>),
    Neg(Box<Symbol>),
    Sin(Box<Symbol>),
    Cos(Box<Symbol>),
    Tan(Box<Symbol>),
    Exp(Box<Symbol>),
    Atan(Box<Symbol>),
    Sqrt(Box<Symbol>),
    Log(Box<Symbol>),
}

fn parse_symbolic(expr: &str) -> Result<Symbol, String> {
    let tokens = tokenize(expr)?;
    let rpn = shunting_yard(&tokens)?;
    rpn_to_symbol(&rpn)
}

fn rpn_to_symbol(tokens: &[RpnToken]) -> Result<Symbol, String> {
    let mut stack: Vec<Symbol> = Vec::new();
    for token in tokens {
        match token {
            RpnToken::Number(n) => stack.push(Symbol::Number(*n)),
            RpnToken::Variable(name) => stack.push(Symbol::Var(name.clone())),
            RpnToken::Operator(op) => {
                let sym = match op {
                    Operator::Add => {
                        let (b, a) = pop_two_symbol(&mut stack)?;
                        Symbol::Add(Box::new(a), Box::new(b))
                    }
                    Operator::Sub => {
                        let (b, a) = pop_two_symbol(&mut stack)?;
                        Symbol::Sub(Box::new(a), Box::new(b))
                    }
                    Operator::Mul => {
                        let (b, a) = pop_two_symbol(&mut stack)?;
                        Symbol::Mul(Box::new(a), Box::new(b))
                    }
                    Operator::Div => {
                        let (b, a) = pop_two_symbol(&mut stack)?;
                        Symbol::Div(Box::new(a), Box::new(b))
                    }
                    Operator::Pow => {
                        let (b, a) = pop_two_symbol(&mut stack)?;
                        Symbol::Pow(Box::new(a), Box::new(b))
                    }
                    Operator::Neg => {
                        let a = stack.pop().ok_or_else(|| "missing_operand".to_string())?;
                        Symbol::Neg(Box::new(a))
                    }
                };
                stack.push(sym);
            }
            RpnToken::Function(name) => {
                let arg = stack.pop().ok_or_else(|| "missing_operand".to_string())?;
                let sym = match name.as_str() {
                    "sin" => Symbol::Sin(Box::new(arg)),
                    "cos" => Symbol::Cos(Box::new(arg)),
                    "tan" => Symbol::Tan(Box::new(arg)),
                    "exp" => Symbol::Exp(Box::new(arg)),
                    "atan" => Symbol::Atan(Box::new(arg)),
                    "sqrt" => Symbol::Sqrt(Box::new(arg)),
                    "log" => Symbol::Log(Box::new(arg)),
                    other => return Err(format!("unknown_function:{other}")),
                };
                stack.push(sym);
            }
        }
    }
    if stack.len() == 1 {
        Ok(stack.pop().unwrap())
    } else {
        Err("invalid_expression".into())
    }
}

fn pop_two_symbol(stack: &mut Vec<Symbol>) -> Result<(Symbol, Symbol), String> {
    let b = stack.pop().ok_or_else(|| "missing_operand".to_string())?;
    let a = stack.pop().ok_or_else(|| "missing_operand".to_string())?;
    Ok((b, a))
}

fn differentiate(expr: &Symbol, var: &str) -> Symbol {
    use Symbol::*;
    match expr {
        Number(_) => Number(0.0),
        Var(name) => {
            if name == var {
                Number(1.0)
            } else {
                Number(0.0)
            }
        }
        Add(a, b) => Add(Box::new(differentiate(a, var)), Box::new(differentiate(b, var))),
        Sub(a, b) => Sub(Box::new(differentiate(a, var)), Box::new(differentiate(b, var))),
        Mul(a, b) => Add(
            Box::new(Mul(Box::new(differentiate(a, var)), b.clone())),
            Box::new(Mul(a.clone(), Box::new(differentiate(b, var)))),
        ),
        Div(a, b) => Div(
            Box::new(Sub(
                Box::new(Mul(Box::new(differentiate(a, var)), b.clone())),
                Box::new(Mul(a.clone(), Box::new(differentiate(b, var)))),
            )),
            Box::new(Pow(b.clone(), Box::new(Number(2.0)))),
        ),
        Pow(a, b) => {
            match (a.as_ref(), b.as_ref()) {
                (_, Number(n)) => Mul(
                    Box::new(Mul(Box::new(Number(*n)), Box::new(Pow(a.clone(), Box::new(Number(n - 1.0)))))),
                    Box::new(differentiate(a, var)),
                ),
                (Number(_), _) => {
                    // d/dx c^g = c^g * ln(c) * g'
                    Mul(
                        Box::new(Mul(
                            Box::new(Pow(a.clone(), b.clone())),
                            Box::new(Log(a.clone())),
                        )),
                        Box::new(differentiate(b, var)),
                    )
                }
                _ => {
                    // General case: d(a^b) = a^b * (b' * ln a + b * a'/a)
                    let a_prime = differentiate(a, var);
                    let b_prime = differentiate(b, var);
                    let term1 = Mul(Box::new(b_prime), Box::new(Log(a.clone())));
                    let term2 = Mul(
                        b.clone(),
                        Box::new(Div(Box::new(a_prime), a.clone())),
                    );
                    Mul(Box::new(Pow(a.clone(), b.clone())), Box::new(Add(Box::new(term1), Box::new(term2))))
                }
            }
        }
        Neg(a) => Neg(Box::new(differentiate(a, var))),
        Sin(a) => Mul(Box::new(Cos(a.clone())), Box::new(differentiate(a, var))),
        Cos(a) => Neg(Box::new(Mul(Box::new(Sin(a.clone())), Box::new(differentiate(a, var))))),
        Tan(a) => Mul(
            Box::new(
                Div(
                    Box::new(Number(1.0)),
                    Box::new(Pow(Box::new(Cos(a.clone())), Box::new(Number(2.0)))),
                ),
            ),
            Box::new(differentiate(a, var)),
        ),
        Exp(a) => Mul(Box::new(Exp(a.clone())), Box::new(differentiate(a, var))),
        Atan(a) => Div(
            Box::new(differentiate(a, var)),
            Box::new(Add(
                Box::new(Number(1.0)),
                Box::new(Pow(a.clone(), Box::new(Number(2.0)))),
            )),
        ),
        Sqrt(a) => Div(
            Box::new(differentiate(a, var)),
            Box::new(Mul(Box::new(Number(2.0)), Box::new(Sqrt(a.clone())))),
        ),
        Log(a) => Div(Box::new(differentiate(a, var)), a.clone()),
    }
}

fn integrate(expr: &Symbol, var: &str) -> Symbol {
    use Symbol::*;
    match expr {
        Number(c) => Mul(Box::new(Number(*c)), Box::new(Var(var.to_string()))),
        Var(name) => {
            if name == var {
                Div(
                    Box::new(Pow(
                        Box::new(Var(name.clone())),
                        Box::new(Number(2.0)),
                    )),
                    Box::new(Number(2.0)),
                )
            } else {
                Mul(Box::new(Var(name.clone())), Box::new(Var(var.to_string())))
            }
        }
        Add(a, b) => Add(Box::new(integrate(a, var)), Box::new(integrate(b, var))),
        Sub(a, b) => Sub(Box::new(integrate(a, var)), Box::new(integrate(b, var))),
        Mul(a, b) => {
            match (&**a, &**b) {
                (Number(c), rhs) => Mul(Box::new(Number(*c)), Box::new(integrate(rhs, var))),
                (lhs, Number(c)) => Mul(Box::new(Number(*c)), Box::new(integrate(lhs, var))),
                _ => Var("∫unsupported".into()),
            }
        }
        Pow(base, exp) => match (&**base, &**exp) {
            (Var(name), Number(n)) if name == var => {
                if (*n - -1.0).abs() < f64::EPSILON {
                    Log(Box::new(Var(name.clone())))
                } else {
                    let new_exp = *n + 1.0;
                    Div(
                        Box::new(Pow(
                            Box::new(Var(name.clone())),
                            Box::new(Number(new_exp)),
                        )),
                        Box::new(Number(new_exp)),
                    )
                }
            }
            (Var(name), Neg(inner)) if name == var => {
                if let Symbol::Number(n) = inner.as_ref() {
                    let exp_val = -*n;
                    if (exp_val - -1.0).abs() < f64::EPSILON {
                        Log(Box::new(Var(name.clone())))
                    } else {
                        let new_exp = exp_val + 1.0;
                        Div(
                            Box::new(Pow(
                                Box::new(Var(name.clone())),
                                Box::new(Number(new_exp)),
                            )),
                            Box::new(Number(new_exp)),
                        )
                    }
                } else {
                    Var("∫unsupported".into())
                }
            }
            _ => Var("∫unsupported".into()),
        },
        Sin(a) => {
            if matches!(&**a, Var(name) if name == var) {
                Neg(Box::new(Cos(a.clone())))
            } else {
                Var("∫unsupported".into())
            }
        }
        Cos(a) => {
            if matches!(&**a, Var(name) if name == var) {
                Sin(a.clone())
            } else {
                Var("∫unsupported".into())
            }
        }
        Tan(a) => {
            if matches!(&**a, Var(name) if name == var) {
                Neg(Box::new(Log(Box::new(Cos(a.clone())))))
            } else {
                Var("∫unsupported".into())
            }
        }
        Exp(a) => match &**a {
            Var(name) if name == var => Exp(a.clone()),
            Mul(left, right) => {
                // ∫ exp(c*x) dx = exp(c*x)/c for constant c
                match (&**left, &**right) {
                    (Number(c), Var(name)) if name == var && c.abs() > f64::EPSILON => {
                        Div(Box::new(Exp(a.clone())), Box::new(Number(*c)))
                    }
                    (Var(name), Number(c)) if name == var && c.abs() > f64::EPSILON => {
                        Div(Box::new(Exp(a.clone())), Box::new(Number(*c)))
                    }
                    _ => Var("∫unsupported".into()),
                }
            }
            _ => Var("∫unsupported".into()),
        },
        Sqrt(a) => {
            // ∫ sqrt(x) dx = 2/3 x^(3/2)
            if matches!(&**a, Var(name) if name == var) {
                Div(
                    Box::new(Mul(
                        Box::new(Number(2.0)),
                        Box::new(Pow(
                            Box::new(Var(var.to_string())),
                            Box::new(Number(1.5)),
                        )),
                    )),
                    Box::new(Number(3.0)),
                )
            } else {
                Var("∫unsupported".into())
            }
        }
        Log(a) => {
            if matches!(&**a, Var(name) if name == var) {
                Sub(
                    Box::new(Mul(
                        Box::new(Var(var.to_string())),
                        Box::new(Log(Box::new(Var(var.to_string())))),
                    )),
                    Box::new(Var(var.to_string())),
                )
            } else {
                Var("∫unsupported".into())
            }
        }
        Atan(_) => Var("∫unsupported".into()),
        Div(num, den) => {
            // ∫ c/x dx = c*log(x)
            if let (Number(c), Var(name)) = (&**num, &**den) {
                if name == var {
                    return Mul(Box::new(Number(*c)), Box::new(Log(Box::new(Var(var.to_string())))));
                }
            }
            // ∫ c/(1+x^2) dx = c*atan(x)
            let c = if let Number(k) = &**num { *k } else { 1.0 };
            if matches!(&**num, Number(_)) {
                if let Some(inner_var) = match_arctan_denominator(den, var) {
                    return Mul(Box::new(Number(c)), Box::new(Atan(Box::new(inner_var))));
                }
            }
            Var("∫unsupported".into())
        }
        Neg(_) => Var("∫unsupported".into()),
    }
}

fn match_arctan_denominator(den: &Symbol, var: &str) -> Option<Symbol> {
    use Symbol::*;
    let (left, right) = match den {
        Add(a, b) => (a.as_ref(), b.as_ref()),
        _ => return None,
    };
    let (one, other) = if is_number(left, 1.0) {
        (left, right)
    } else if is_number(right, 1.0) {
        (right, left)
    } else {
        return None;
    };
    let _ = one;
    match other {
        Pow(base, exp) => match (&**base, &**exp) {
            (Var(name), Number(_)) if name == var && is_number(exp, 2.0) => {
                Some(Var(var.to_string()))
            }
            _ => None,
        },
        _ => None,
    }
}

fn is_number(sym: &Symbol, target: f64) -> bool {
    matches!(sym, Symbol::Number(n) if (n - target).abs() < 1e-12)
}

fn simplify(expr: &Symbol) -> Symbol {
    use Symbol::*;
    match expr {
        Add(a, b) => {
            let sa = simplify(a);
            let sb = simplify(b);
            match (&sa, &sb) {
                (Number(x), Number(y)) => Number(x + y),
                (Number(0.0), other) => other.clone(),
                (other, Number(0.0)) => other.clone(),
                _ => Add(Box::new(sa), Box::new(sb)),
            }
        }
        Sub(a, b) => {
            let sa = simplify(a);
            let sb = simplify(b);
            match (&sa, &sb) {
                (Number(x), Number(y)) => Number(x - y),
                (other, Number(0.0)) => other.clone(),
                _ => Sub(Box::new(sa), Box::new(sb)),
            }
        }
        Mul(a, b) => {
            let sa = simplify(a);
            let sb = simplify(b);
            match (&sa, &sb) {
                (Number(x), Number(y)) => Number(x * y),
                (Number(0.0), _) | (_, Number(0.0)) => Number(0.0),
                (Number(1.0), other) => other.clone(),
                (other, Number(1.0)) => other.clone(),
                _ => Mul(Box::new(sa), Box::new(sb)),
            }
        }
        Div(a, b) => {
            let sa = simplify(a);
            let sb = simplify(b);
            match (&sa, &sb) {
                (Number(x), Number(y)) if *y != 0.0 => Number(x / y),
                (other, Number(1.0)) => other.clone(),
                _ => Div(Box::new(sa), Box::new(sb)),
            }
        }
        Pow(a, b) => {
            let sa = simplify(a);
            let sb = simplify(b);
            match (&sa, &sb) {
                (Number(x), Number(y)) => Number(x.powf(*y)),
                (_, Number(0.0)) => Number(1.0),
                (other, Number(1.0)) => other.clone(),
                _ => Pow(Box::new(sa), Box::new(sb)),
            }
        }
        Neg(a) => {
            let sa = simplify(a);
            match sa {
                Number(v) => Number(-v),
                _ => Neg(Box::new(sa)),
            }
        }
        Sin(a) => Sin(Box::new(simplify(a))),
        Cos(a) => Cos(Box::new(simplify(a))),
        Tan(a) => Tan(Box::new(simplify(a))),
        Exp(a) => Exp(Box::new(simplify(a))),
        Atan(a) => Atan(Box::new(simplify(a))),
        Sqrt(a) => Sqrt(Box::new(simplify(a))),
        Log(a) => Log(Box::new(simplify(a))),
        Number(n) => Number(*n),
        Var(v) => Var(v.clone()),
    }
}

fn render_symbol(expr: &Symbol) -> String {
    use Symbol::*;
    match expr {
        Number(n) => {
            let mut out = format!("{:.10}", n);
            while out.contains('.') && out.ends_with('0') {
                out.pop();
            }
            if out.ends_with('.') {
                out.pop();
            }
            out
        }
        Var(v) => v.clone(),
        Add(a, b) => format!("{}+{}", wrap(a, 1), wrap(b, 1)),
        Sub(a, b) => format!("{}-{}", wrap(a, 1), wrap(b, 1)),
        Mul(a, b) => format!("{}*{}", wrap(a, 2), wrap(b, 2)),
        Div(a, b) => format!("{}/{}", wrap(a, 2), wrap(b, 2)),
        Pow(a, b) => format!("{}^{}", wrap(a, 3), wrap(b, 3)),
        Neg(a) => format!("-{}", wrap(a, 4)),
        Sin(a) => format!("sin({})", render_symbol(a)),
        Cos(a) => format!("cos({})", render_symbol(a)),
        Tan(a) => format!("tan({})", render_symbol(a)),
        Exp(a) => format!("exp({})", render_symbol(a)),
        Atan(a) => format!("atan({})", render_symbol(a)),
        Sqrt(a) => format!("sqrt({})", render_symbol(a)),
        Log(a) => format!("log({})", render_symbol(a)),
    }
}

fn precedence(sym: &Symbol) -> u8 {
    use Symbol::*;
    match sym {
        Add(_, _) | Sub(_, _) => 1,
        Mul(_, _) | Div(_, _) => 2,
        Pow(_, _) => 3,
        Neg(_) => 4,
        _ => 5,
    }
}

fn wrap(sym: &Symbol, parent_prec: u8) -> String {
    let child = render_symbol(sym);
    let prec = precedence(sym);
    if prec < parent_prec {
        format!("({child})")
    } else {
        child
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Assoc {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Operator {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
    Neg,
}

impl Operator {
    fn precedence(self) -> u8 {
        match self {
            Operator::Add | Operator::Sub => 1,
            Operator::Mul | Operator::Div => 2,
            Operator::Pow => 3,
            Operator::Neg => 4,
        }
    }

    fn assoc(self) -> Assoc {
        match self {
            Operator::Pow => Assoc::Right,
            Operator::Neg => Assoc::Right,
            _ => Assoc::Left,
        }
    }

    fn arity(self) -> usize {
        match self {
            Operator::Neg => 1,
            _ => 2,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Number(f64),
    Variable(String),
    Operator(Operator),
    Function(String),
    LeftParen,
    RightParen,
}

#[derive(Debug, Clone)]
enum RpnToken {
    Number(f64),
    Variable(String),
    Operator(Operator),
    Function(String),
}

fn tokenize(expr: &str) -> Result<Vec<Token>, String> {
    let mut chars = expr.chars().peekable();
    let mut tokens = Vec::new();
    let mut prev_is_value = false;

    while let Some(&ch) = chars.peek() {
        match ch {
            c if c.is_whitespace() => {
                chars.next();
            }
            c if c.is_ascii_digit() || c == '.' => {
                let number = parse_number(&mut chars)?;
                tokens.push(Token::Number(number));
                prev_is_value = true;
            }
            c if c.is_ascii_alphabetic() => {
                let ident = parse_identifier(&mut chars);
                let lowered = ident.to_lowercase();
                if lowered == "pi" {
                    tokens.push(Token::Number(PI));
                    prev_is_value = true;
                } else if lowered == "e" {
                    tokens.push(Token::Number(E));
                    prev_is_value = true;
                } else if matches!(lowered.as_str(), "sin" | "cos" | "tan" | "exp" | "atan" | "sqrt" | "log" | "deriv") {
                    tokens.push(Token::Function(lowered));
                    prev_is_value = false;
                } else {
                    tokens.push(Token::Variable(lowered));
                    prev_is_value = true;
                }
            }
            '+' => {
                chars.next();
                if prev_is_value {
                    tokens.push(Token::Operator(Operator::Add));
                }
                prev_is_value = false;
            }
            '-' => {
                chars.next();
                let op = if prev_is_value {
                    Operator::Sub
                } else {
                    Operator::Neg
                };
                tokens.push(Token::Operator(op));
                prev_is_value = false;
            }
            '*' => {
                chars.next();
                tokens.push(Token::Operator(Operator::Mul));
                prev_is_value = false;
            }
            '/' => {
                chars.next();
                tokens.push(Token::Operator(Operator::Div));
                prev_is_value = false;
            }
            '^' => {
                chars.next();
                tokens.push(Token::Operator(Operator::Pow));
                prev_is_value = false;
            }
            '(' => {
                chars.next();
                tokens.push(Token::LeftParen);
                prev_is_value = false;
            }
            ')' => {
                chars.next();
                tokens.push(Token::RightParen);
                prev_is_value = true;
            }
            other => {
                return Err(format!("unexpected_char:{other}"));
            }
        }
    }

    Ok(tokens)
}

fn parse_number(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> Result<f64, String> {
    let mut buf = String::new();
    let mut has_exp = false;

    while let Some(&c) = chars.peek() {
        if c.is_ascii_digit() || c == '.' {
            buf.push(c);
            chars.next();
        } else if (c == 'e' || c == 'E') && !has_exp {
            has_exp = true;
            buf.push(c);
            chars.next();
            if let Some(&sign) = chars.peek() {
                if sign == '+' || sign == '-' {
                    buf.push(sign);
                    chars.next();
                }
            }
        } else {
            break;
        }
    }

    buf.parse::<f64>()
        .map_err(|_| format!("invalid_number:{buf}"))
}

fn parse_identifier(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> String {
    let mut buf = String::new();
    while let Some(&c) = chars.peek() {
        if c.is_ascii_alphabetic() {
            buf.push(c);
            chars.next();
        } else {
            break;
        }
    }
    buf
}

fn shunting_yard(tokens: &[Token]) -> Result<Vec<RpnToken>, String> {
    let mut output: Vec<RpnToken> = Vec::new();
    let mut stack: Vec<Token> = Vec::new();

    for token in tokens {
        match token {
            Token::Number(n) => output.push(RpnToken::Number(*n)),
            Token::Variable(name) => output.push(RpnToken::Variable(name.clone())),
            Token::Function(name) => stack.push(Token::Function(name.clone())),
            Token::Operator(op) => {
                while let Some(top) = stack.last() {
                    match top {
                        Token::Operator(top_op)
                            if (top_op.precedence() > op.precedence())
                                || (top_op.precedence() == op.precedence()
                                    && op.assoc() == Assoc::Left) =>
                        {
                            let popped = stack.pop().unwrap();
                            if let Token::Operator(o) = popped {
                                output.push(RpnToken::Operator(o));
                            }
                        }
                        Token::Function(_) => {
                            let func = stack.pop().unwrap();
                            if let Token::Function(name) = func {
                                output.push(RpnToken::Function(name));
                            }
                        }
                        _ => break,
                    }
                }
                stack.push(Token::Operator(*op));
            }
            Token::LeftParen => stack.push(Token::LeftParen),
            Token::RightParen => {
                while let Some(top) = stack.pop() {
                    if matches!(top, Token::LeftParen) {
                        break;
                    }
                    match top {
                        Token::Operator(o) => output.push(RpnToken::Operator(o)),
                        Token::Function(name) => output.push(RpnToken::Function(name)),
                        _ => {}
                    }
                }
                if let Some(Token::Function(_)) = stack.last() {
                    if let Some(Token::Function(name)) = stack.pop() {
                        output.push(RpnToken::Function(name));
                    }
                }
            }
        }
    }

    while let Some(top) = stack.pop() {
        match top {
            Token::LeftParen | Token::RightParen => return Err("mismatched_parentheses".into()),
            Token::Operator(o) => output.push(RpnToken::Operator(o)),
            Token::Function(name) => output.push(RpnToken::Function(name)),
            _ => return Err("invalid_expression".into()),
        }
    }

    Ok(output)
}

fn eval_rpn(tokens: &[RpnToken]) -> Result<f64, String> {
    let mut stack: Vec<f64> = Vec::new();
    for token in tokens {
        match token {
            RpnToken::Number(n) => stack.push(*n),
            RpnToken::Variable(name) => {
                return Err(format!("unknown_variable:{name}"));
            }
            RpnToken::Operator(op) => {
                let arity = op.arity();
                if stack.len() < arity {
                    return Err("missing_operand".into());
                }
                let result = match op {
                    Operator::Add => {
                        let (b, a) = pop_two(&mut stack);
                        a + b
                    }
                    Operator::Sub => {
                        let (b, a) = pop_two(&mut stack);
                        a - b
                    }
                    Operator::Mul => {
                        let (b, a) = pop_two(&mut stack);
                        a * b
                    }
                    Operator::Div => {
                        let (b, a) = pop_two(&mut stack);
                        if b.abs() < f64::EPSILON {
                            return Err("division_by_zero".into());
                        }
                        a / b
                    }
                    Operator::Pow => {
                        let (b, a) = pop_two(&mut stack);
                        a.powf(b)
                    }
                    Operator::Neg => {
                        let a = stack.pop().unwrap();
                        -a
                    }
                };
                if !result.is_finite() {
                    return Err("non_finite_result".into());
                }
                stack.push(result);
            }
            RpnToken::Function(name) => {
                let Some(arg) = stack.pop() else {
                    return Err("missing_operand".into());
                };
                let res = match name.as_str() {
                    "sin" => arg.sin(),
                    "cos" => arg.cos(),
                    "tan" => arg.tan(),
                    "exp" => arg.exp(),
                    "atan" => arg.atan(),
                    "sqrt" => {
                        if arg < 0.0 {
                            return Err("sqrt_of_negative".into());
                        }
                        arg.sqrt()
                    }
                    "log" => {
                        if arg <= 0.0 {
                            return Err("log_non_positive".into());
                        }
                        arg.ln()
                    }
                    other => return Err(format!("unknown_function:{other}")),
                };
                if !res.is_finite() {
                    return Err("non_finite_result".into());
                }
                stack.push(res);
            }
        }
    }

    if stack.len() == 1 {
        Ok(stack[0])
    } else {
        Err("invalid_expression".into())
    }
}

fn pop_two(stack: &mut Vec<f64>) -> (f64, f64) {
    let b = stack.pop().unwrap();
    let a = stack.pop().unwrap();
    (b, a)
}

fn format_result(value: f64) -> String {
    let mut out = format!("{:.10}", value);
    while out.contains('.') && out.ends_with('0') {
        out.pop();
    }
    if out.ends_with('.') {
        out.pop();
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    fn approx_eq(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn respects_operator_precedence() {
        let res = evaluate_expression("2 + 3 * 4").unwrap();
        assert!(approx_eq(res, 14.0));
    }

    #[test]
    fn handles_parentheses_and_exponent() {
        let res = evaluate_expression("(2 + 3) * 4 ^ 2").unwrap();
        assert!(approx_eq(res, 80.0));
    }

    #[test]
    fn derivative_of_polynomial() {
        let ast = parse_symbolic("x^3 + 2*x").unwrap();
        let deriv = differentiate(&ast, "x");
        let simplified = simplify(&deriv);
        assert_eq!(render_symbol(&simplified), "3*x^2+2");
    }

    #[test]
    fn derivative_of_trig_and_log() {
        let ast = parse_symbolic("sin(x) + log(x)").unwrap();
        let deriv = simplify(&differentiate(&ast, "x"));
        assert_eq!(render_symbol(&deriv), "cos(x)+1/x");
    }

    #[test]
    fn derivative_chain_with_power() {
        let ast = parse_symbolic("(x^2 + 1)^3").unwrap();
        let deriv = simplify(&differentiate(&ast, "x"));
        assert_eq!(render_symbol(&deriv), "3*(x^2+1)^2*2*x");
    }

    #[test]
    fn supports_unary_minus_and_functions() {
        let res = evaluate_expression("-cos(0) + sqrt(9)").unwrap();
        assert!(approx_eq(res, 2.0));
    }

    #[test]
    fn exponent_is_right_associative() {
        let res = evaluate_expression("2^3^2").unwrap();
        assert!(approx_eq(res, 512.0));
    }

    #[test]
    fn detects_division_by_zero() {
        let err = evaluate_expression("1/0").unwrap_err();
        assert_eq!(err, "division_by_zero");
    }

    #[test]
    fn errors_on_unknown_function() {
        let err = evaluate_expression("foo(2)").unwrap_err();
        assert!(err.contains("unknown"));
    }

    #[test]
    fn numeric_and_symbolic_paths_coexist() {
        // Numeric evaluation stays numeric.
        let num = evaluate_expression("2+2").unwrap();
        assert!(approx_eq(num, 4.0));

        // Symbolic deriv triggers symbolic_result string payload.
        let sym_err = evaluate_expression("deriv(x^2)").unwrap_err();
        assert!(sym_err.starts_with("symbolic_result:"));
        assert!(sym_err.contains("2*x"));
    }

    #[test]
    fn integrate_power_rule() {
        let ast = parse_symbolic("x^2").unwrap();
        let integ = simplify(&integrate(&ast, "x"));
        assert_eq!(render_symbol(&integ), "x^3/3");
    }

    #[test]
    fn integrate_trig_and_constants() {
        let ast = parse_symbolic("2*cos(x)").unwrap();
        let integ = simplify(&integrate(&ast, "x"));
        assert_eq!(render_symbol(&integ), "2*sin(x)");
    }

    #[test]
    fn integrate_inverse_x_to_log() {
        let ast = parse_symbolic("x^-1").unwrap();
        let integ = simplify(&integrate(&ast, "x"));
        assert_eq!(render_symbol(&integ), "log(x)");
    }

    #[test]
    fn integrate_exp_of_x() {
        let ast = parse_symbolic("exp(x)").unwrap();
        let integ = simplify(&integrate(&ast, "x"));
        assert_eq!(render_symbol(&integ), "exp(x)");
    }

    #[test]
    fn integrate_tan_of_x() {
        let ast = parse_symbolic("tan(x)").unwrap();
        let integ = simplify(&integrate(&ast, "x"));
        assert_eq!(render_symbol(&integ), "-log(cos(x))");
    }

    #[test]
    fn integrate_one_over_one_plus_x_squared_to_atan() {
        let ast = parse_symbolic("1/(1+x^2)").unwrap();
        let integ = simplify(&integrate(&ast, "x"));
        assert_eq!(render_symbol(&integ), "atan(x)");
    }

    #[test]
    fn integrate_constant_over_x_to_log() {
        let ast = parse_symbolic("2/x").unwrap();
        let integ = simplify(&integrate(&ast, "x"));
        assert_eq!(render_symbol(&integ), "2*log(x)");
    }

    #[test]
    fn integrate_dispatches_in_eval() {
        let res = evaluate_expression("integ(x^3)").unwrap_err();
        assert!(res.starts_with("symbolic_result:"));
        assert!(res.contains("x^4/4"));
    }

    #[test]
    fn history_renders_as_virtual_list() {
        let mut state = AppState::new();
        state.math_tool.history.push(MathHistoryEntry {
            expression: "1+1".into(),
            result: "2".into(),
        });
        state.math_tool.history.push(MathHistoryEntry {
            expression: "2+2".into(),
            result: "4".into(),
        });

        let ui = render_math_tool_screen(&state);
        assert_eq!(ui.get("type").and_then(Value::as_str), Some("Column"));
        let children = ui.get("children").and_then(Value::as_array).unwrap();
        let has_virtual = children.iter().any(|c| {
            c.get("type")
                .and_then(Value::as_str)
                .map(|t| t == "VirtualList")
                .unwrap_or(false)
        });
        assert!(has_virtual, "expected history to use VirtualList");
    }

    #[test]
    fn virtual_list_serializes_estimated_height() {
        let items = vec![serde_json::to_value(UiText::new("a")).unwrap()];
        let list = UiVirtualList::new(items).estimated_item_height(24);
        let val = serde_json::to_value(list).unwrap();
        assert_eq!(val.get("type").and_then(Value::as_str), Some("VirtualList"));
        assert_eq!(val.get("estimated_item_height").and_then(Value::as_u64), Some(24));
    }

    #[test]
    fn clear_history_resets_entries_and_expression() {
        let mut state = AppState::new();
        handle_math_action(
            &mut state,
            "math_calculate",
            &HashMap::from([("math_expr".into(), "1+1".into())]),
        );
        assert_eq!(state.math_tool.history.len(), 1);
        assert_eq!(state.math_tool.expression, "1+1");

        handle_math_action(&mut state, "math_clear_history", &HashMap::new());
        assert!(state.math_tool.history.is_empty());
        assert!(state.math_tool.expression.is_empty());
    }
}
