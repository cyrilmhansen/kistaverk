use crate::state::{AppState, MathHistoryEntry};
use crate::ui::{
    maybe_push_back, Button as UiButton, Column as UiColumn, Text as UiText,
    TextInput as UiTextInput,
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
                .action_on_submit("math_calculate"),
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
        for entry in state.math_tool.history.iter() {
            let line = format!("{} = {}", entry.expression, entry.result);
            children.push(serde_json::to_value(UiText::new(&line).size(12.0)).unwrap());
        }
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
            state.math_tool.error = None;
        }
        _ => {}
    }
}

pub fn evaluate_expression(expr: &str) -> Result<f64, String> {
    let tokens = tokenize(expr)?;
    let rpn = shunting_yard(&tokens)?;
    eval_rpn(&rpn)
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
    Operator(Operator),
    Function(String),
    LeftParen,
    RightParen,
}

#[derive(Debug, Clone)]
enum RpnToken {
    Number(f64),
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
                } else {
                    tokens.push(Token::Function(lowered));
                    prev_is_value = false;
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
        assert!(err.starts_with("unknown_function"));
    }
}
