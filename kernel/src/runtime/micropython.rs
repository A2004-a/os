//! NovaOS Apex — MicroPython Embedded Scripting Engine (M14)
//! Provides a lightweight `#![no_std]` compliant Python bytecode and AST evaluation engine
//! for sandboxed script execution, variable scopes, and built-ins.

extern crate alloc;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

#[derive(Clone, Debug, PartialEq)]
pub enum PyValue {
    None,
    Bool(bool),
    Int(i64),
    Str(String),
    List(Vec<PyValue>),
}

impl PyValue {
    pub fn is_truthy(&self) -> bool {
        match self {
            PyValue::None => false,
            PyValue::Bool(b) => *b,
            PyValue::Int(i) => *i != 0,
            PyValue::Str(s) => !s.is_empty(),
            PyValue::List(l) => !l.is_empty(),
        }
    }

    pub fn to_string_repr(&self) -> String {
        match self {
            PyValue::None => String::from("None"),
            PyValue::Bool(b) => String::from(if *b { "True" } else { "False" }),
            PyValue::Int(i) => {
                let mut s = String::new();
                if *i < 0 {
                    s.push('-');
                }
                let mut n = i.unsigned_abs();
                if n == 0 {
                    s.push('0');
                } else {
                    let mut buf = [0u8; 24];
                    let mut idx = 0;
                    while n > 0 && idx < buf.len() {
                        buf[idx] = b'0' + (n % 10) as u8;
                        n /= 10;
                        idx += 1;
                    }
                    for j in (0..idx).rev() {
                        s.push(buf[j] as char);
                    }
                }
                s
            }
            PyValue::Str(s) => s.clone(),
            PyValue::List(l) => {
                let mut s = String::from("[");
                for (idx, item) in l.iter().enumerate() {
                    if idx > 0 {
                        s.push_str(", ");
                    }
                    s.push_str(&item.to_string_repr());
                }
                s.push(']');
                s
            }
        }
    }
}

pub struct MicroPythonVm {
    globals: BTreeMap<String, PyValue>,
    pub output: String,
}

impl MicroPythonVm {
    pub fn new() -> Self {
        Self {
            globals: BTreeMap::new(),
            output: String::new(),
        }
    }

    pub fn get_var(&self, name: &str) -> Option<&PyValue> {
        self.globals.get(name)
    }

    pub fn set_var(&mut self, name: &str, val: PyValue) {
        self.globals.insert(String::from(name), val);
    }

    pub fn eval_expr(&mut self, expr: &str) -> Result<PyValue, &'static str> {
        let trimmed = expr.trim();
        if trimmed == "True" {
            return Ok(PyValue::Bool(true));
        }
        if trimmed == "False" {
            return Ok(PyValue::Bool(false));
        }
        if trimmed == "None" {
            return Ok(PyValue::None);
        }

        // String literal: "..." or '...'
        if is_single_string_literal(trimmed) {
            return Ok(PyValue::Str(String::from(&trimmed[1..trimmed.len() - 1])));
        }

        // Built-in: len(...)
        if trimmed.starts_with("len(") && trimmed.ends_with(')') {
            let inner = &trimmed[4..trimmed.len() - 1];
            let val = self.eval_expr(inner)?;
            let length = match val {
                PyValue::Str(s) => s.len(),
                PyValue::List(l) => l.len(),
                _ => return Err("TypeError: object of this type has no len()"),
            };
            return Ok(PyValue::Int(length as i64));
        }

        // Binary comparisons: ==, !=, <=, >=, <, >
        if let Some(pos) = trimmed.find("==") {
            let left = self.eval_expr(&trimmed[..pos])?;
            let right = self.eval_expr(&trimmed[pos + 2..])?;
            return Ok(PyValue::Bool(left == right));
        }
        if let Some(pos) = trimmed.find("!=") {
            let left = self.eval_expr(&trimmed[..pos])?;
            let right = self.eval_expr(&trimmed[pos + 2..])?;
            return Ok(PyValue::Bool(left != right));
        }
        if let Some(pos) = trimmed.find("<=") {
            let left = self.eval_expr(&trimmed[..pos])?;
            let right = self.eval_expr(&trimmed[pos + 2..])?;
            match (left, right) {
                (PyValue::Int(a), PyValue::Int(b)) => return Ok(PyValue::Bool(a <= b)),
                _ => return Err("TypeError: unsupported operand types for <="),
            }
        }
        if let Some(pos) = trimmed.find(">=") {
            let left = self.eval_expr(&trimmed[..pos])?;
            let right = self.eval_expr(&trimmed[pos + 2..])?;
            match (left, right) {
                (PyValue::Int(a), PyValue::Int(b)) => return Ok(PyValue::Bool(a >= b)),
                _ => return Err("TypeError: unsupported operand types for >="),
            }
        }
        if let Some(pos) = trimmed.find('<') {
            let left = self.eval_expr(&trimmed[..pos])?;
            let right = self.eval_expr(&trimmed[pos + 1..])?;
            match (left, right) {
                (PyValue::Int(a), PyValue::Int(b)) => return Ok(PyValue::Bool(a < b)),
                _ => return Err("TypeError: unsupported operand types for <"),
            }
        }
        if let Some(pos) = trimmed.find('>') {
            let left = self.eval_expr(&trimmed[..pos])?;
            let right = self.eval_expr(&trimmed[pos + 1..])?;
            match (left, right) {
                (PyValue::Int(a), PyValue::Int(b)) => return Ok(PyValue::Bool(a > b)),
                _ => return Err("TypeError: unsupported operand types for >"),
            }
        }

        // Arithmetic operations: +, -, *, /
        // Look for addition/subtraction outside quotes
        if let Some(pos) = find_op_outside_quotes(trimmed, '+') {
            let left = self.eval_expr(&trimmed[..pos])?;
            let right = self.eval_expr(&trimmed[pos + 1..])?;
            match (left, right) {
                (PyValue::Int(a), PyValue::Int(b)) => return Ok(PyValue::Int(a + b)),
                (PyValue::Str(mut a), PyValue::Str(b)) => {
                    a.push_str(&b);
                    return Ok(PyValue::Str(a));
                }
                _ => return Err("TypeError: unsupported operand types for +"),
            }
        }
        if let Some(pos) = find_op_outside_quotes(trimmed, '-') {
            if pos > 0 {
                let left = self.eval_expr(&trimmed[..pos])?;
                let right = self.eval_expr(&trimmed[pos + 1..])?;
                match (left, right) {
                    (PyValue::Int(a), PyValue::Int(b)) => return Ok(PyValue::Int(a - b)),
                    _ => return Err("TypeError: unsupported operand types for -"),
                }
            }
        }
        if let Some(pos) = find_op_outside_quotes(trimmed, '*') {
            let left = self.eval_expr(&trimmed[..pos])?;
            let right = self.eval_expr(&trimmed[pos + 1..])?;
            match (left, right) {
                (PyValue::Int(a), PyValue::Int(b)) => return Ok(PyValue::Int(a * b)),
                _ => return Err("TypeError: unsupported operand types for *"),
            }
        }
        if let Some(pos) = find_op_outside_quotes(trimmed, '/') {
            let left = self.eval_expr(&trimmed[..pos])?;
            let right = self.eval_expr(&trimmed[pos + 1..])?;
            match (left, right) {
                (PyValue::Int(a), PyValue::Int(b)) => {
                    if b == 0 {
                        return Err("ZeroDivisionError: division by zero");
                    }
                    return Ok(PyValue::Int(a / b));
                }
                _ => return Err("TypeError: unsupported operand types for /"),
            }
        }

        // Integer literal
        if let Ok(val) = trimmed.parse::<i64>() {
            return Ok(PyValue::Int(val));
        }

        // Variable lookup
        if let Some(val) = self.globals.get(trimmed) {
            return Ok(val.clone());
        }

        Err("NameError: name is not defined")
    }

    pub fn run_script(&mut self, script: &str) -> Result<PyValue, &'static str> {
        let mut last_val = PyValue::None;
        let lines: Vec<&str> = script.lines().collect();
        let mut idx = 0;

        while idx < lines.len() {
            let line = lines[idx].trim();
            idx += 1;

            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // print(...)
            if line.starts_with("print(") && line.ends_with(')') {
                let inner = &line[6..line.len() - 1];
                let val = self.eval_expr(inner)?;
                let text = val.to_string_repr();
                self.output.push_str(&text);
                self.output.push('\n');
                last_val = val;
                continue;
            }

            // while loop: `while <cond>:`
            if line.starts_with("while ") && line.ends_with(':') {
                let cond_str = &line[6..line.len() - 1].trim();
                let body_start = idx;
                let mut body_end = idx;
                while body_end < lines.len() && (lines[body_end].starts_with("    ") || lines[body_end].starts_with('\t')) {
                    body_end += 1;
                }

                let mut loop_count = 0;
                while self.eval_expr(cond_str)?.is_truthy() {
                    loop_count += 1;
                    if loop_count > 1000 {
                        return Err("RuntimeError: maximum loop iteration limit exceeded");
                    }
                    for b_idx in body_start..body_end {
                        let stmt = lines[b_idx].trim();
                        if !stmt.is_empty() {
                            self.execute_statement(stmt)?;
                        }
                    }
                }
                idx = body_end;
                continue;
            }

            // if / else statement
            if line.starts_with("if ") && line.ends_with(':') {
                let cond_str = &line[3..line.len() - 1].trim();
                let cond_val = self.eval_expr(cond_str)?.is_truthy();
                let mut if_lines = Vec::new();
                while idx < lines.len() && (lines[idx].starts_with("    ") || lines[idx].starts_with('\t')) {
                    if_lines.push(lines[idx].trim());
                    idx += 1;
                }

                let mut else_lines = Vec::new();
                if idx < lines.len() && lines[idx].trim() == "else:" {
                    idx += 1;
                    while idx < lines.len() && (lines[idx].starts_with("    ") || lines[idx].starts_with('\t')) {
                        else_lines.push(lines[idx].trim());
                        idx += 1;
                    }
                }

                if cond_val {
                    for stmt in if_lines {
                        self.execute_statement(stmt)?;
                    }
                } else {
                    for stmt in else_lines {
                        self.execute_statement(stmt)?;
                    }
                }
                continue;
            }

            // Simple assignment or expression
            last_val = self.execute_statement(line)?;
        }

        Ok(last_val)
    }

    fn execute_statement(&mut self, stmt: &str) -> Result<PyValue, &'static str> {
        if let Some(eq_pos) = stmt.find('=') {
            if eq_pos > 0 && !stmt[..eq_pos].ends_with('!') && !stmt[..eq_pos].ends_with('<') && !stmt[..eq_pos].ends_with('>') {
                let var_name = stmt[..eq_pos].trim();
                let expr_str = stmt[eq_pos + 1..].trim();
                let val = self.eval_expr(expr_str)?;
                self.globals.insert(String::from(var_name), val.clone());
                return Ok(val);
            }
        }

        self.eval_expr(stmt)
    }
}

fn find_op_outside_quotes(s: &str, target: char) -> Option<usize> {
    let mut in_quote = false;
    let mut quote_char = ' ';

    for (i, c) in s.char_indices() {
        if c == '"' || c == '\'' {
            if in_quote && c == quote_char {
                in_quote = false;
            } else if !in_quote {
                in_quote = true;
                quote_char = c;
            }
        } else if !in_quote && c == target {
            return Some(i);
        }
    }
    None
}

fn is_single_string_literal(s: &str) -> bool {
    if s.len() < 2 {
        return false;
    }
    let first = match s.chars().next() {
        Some(c) => c,
        None => return false,
    };
    if first != '"' && first != '\'' {
        return false;
    }
    let mut chars = s.char_indices();
    chars.next(); // skip first quote
    for (i, c) in chars {
        if c == first {
            return i == s.len() - 1;
        }
    }
    false
}
