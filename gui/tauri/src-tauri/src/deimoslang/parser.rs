//! DeimosLang Parser — Abstract Syntax Tree and Parser.
//!
//! Faithfully ported from `deimos-reference/src/deimoslang/parser.py`.
#![allow(dead_code, non_camel_case_types, unused_mut, unused_variables)]

use crate::deimoslang::tokenizer::{Token, TokenKind, LineInfo, render_tokens, TokenValue};
use crate::deimoslang::types::*;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParserError {
    #[error("{0}")]
    Error(String),
}

pub struct Parser {
    pub tokens: Vec<Token>,
    pub i: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, i: 0 }
    }

    fn fetch_line_tokens(&self, line: usize) -> Vec<Token> {
        let mut result = Vec::new();
        for tok in &self.tokens {
            if tok.line_info.line == line {
                result.push(tok.clone());
            }
        }
        result
    }

    fn err_manual(&self, line_info: &LineInfo, msg: &str) -> ParserError {
        let line_toks = self.fetch_line_tokens(line_info.line);
        let mut err_msg = msg.to_string();
        err_msg.push('\n');
        err_msg.push_str(&render_tokens(&line_toks));
        let arrow_indent = if line_info.column > 1 {
            " ".repeat(line_info.column - 1)
        } else {
            "".to_string()
        };
        err_msg.push('\n');
        err_msg.push_str(&arrow_indent);
        err_msg.push('^');
        ParserError::Error(format!("{}\nLine: {}", err_msg, line_info.line))
    }

    fn err(&self, token: &Token, msg: &str) -> ParserError {
        self.err_manual(&token.line_info, msg)
    }

    fn skip_any(&mut self, kinds: &[TokenKind]) {
        if self.i < self.tokens.len() && kinds.contains(&self.tokens[self.i].kind) {
            self.i += 1;
        }
    }

    fn skip_comma(&mut self) {
        self.skip_any(&[TokenKind::comma]);
    }

    fn expect_consume_any(&mut self, kinds: &[TokenKind]) -> Result<Token, ParserError> {
        if self.i >= self.tokens.len() {
            let last_token = self.tokens.last().ok_or_else(|| ParserError::Error("No tokens".to_string()))?;
            return Err(self.err(last_token, &format!("Premature end of file, expected {:?} before the end", kinds)));
        }
        let result = self.tokens[self.i].clone();
        if !kinds.contains(&result.kind) {
            return Err(self.err(&result, &format!("Expected token kinds {:?} but got {:?}", kinds, result.kind)));
        }
        self.i += 1;
        Ok(result)
    }

    fn expect_consume(&mut self, kind: TokenKind) -> Result<Token, ParserError> {
        self.expect_consume_any(&[kind])
    }

    fn consume_any_optional(&mut self, kinds: &[TokenKind]) -> Option<Token> {
        if self.i >= self.tokens.len() {
            return None;
        }
        let result = self.tokens[self.i].clone();
        if !kinds.contains(&result.kind) {
            return None;
        }
        self.i += 1;
        Some(result)
    }

    fn consume_optional(&mut self, kind: TokenKind) -> Option<Token> {
        self.consume_any_optional(&[kind])
    }

    pub fn gen_greater_expression(&self, left: Expression, right: Expression, player_selector: PlayerSelector) -> Expression {
        Expression::SelectorGroup(player_selector, Box::new(Expression::Greater(Box::new(left), Box::new(right))))
    }

    pub fn gen_equivalent_expression(&self, left: Expression, right: Expression, player_selector: PlayerSelector) -> Expression {
        Expression::SelectorGroup(player_selector, Box::new(Expression::Equivalent(Box::new(left), Box::new(right))))
    }

    pub fn parse_numeric_comparison(&mut self, evaluated: Expression, player_selector: PlayerSelector) -> Result<Expression, ParserError> {
        if self.i < self.tokens.len() && [TokenKind::greater, TokenKind::less, TokenKind::equals].contains(&self.tokens[self.i].kind) {
            let operator = self.tokens[self.i].clone();
            self.i += 1;

            let target = self.parse_expression()?;

            if operator.kind == TokenKind::greater {
                Ok(self.gen_greater_expression(evaluated, target, player_selector))
            } else if operator.kind == TokenKind::less {
                Ok(self.gen_greater_expression(target, evaluated, player_selector))
            } else {
                Ok(self.gen_equivalent_expression(evaluated, target, player_selector))
            }
        } else if self.i < self.tokens.len() && self.tokens[self.i].kind == TokenKind::keyword_isbetween {
            self.i += 1;

            if self.i < self.tokens.len() && self.tokens[self.i].kind == TokenKind::identifier {
                let range_ident = self.tokens[self.i].literal.clone();
                self.i += 1;

                let range_expr = Expression::Ident(range_ident);

                let min_expr = self.gen_greater_expression(
                    evaluated.clone(),
                    Expression::IndexAccess(Box::new(range_expr.clone()), Box::new(Expression::Number(0.0))),
                    player_selector.clone(),
                );
                let max_expr = self.gen_greater_expression(
                    Expression::IndexAccess(Box::new(range_expr), Box::new(Expression::Number(1.0))),
                    evaluated,
                    player_selector,
                );

                Ok(Expression::And(vec![Box::new(min_expr), Box::new(max_expr)]))
            } else {
                let range_token = self.expect_consume(TokenKind::string)?;
                let range_str = match range_token.value {
                    TokenValue::String(ref s) => s,
                    _ => return Err(self.err(&range_token, "Expected string")),
                };

                let parts: Vec<&str> = range_str.split('-').collect();
                if parts.len() == 2 {
                    let min_val = parts[0].parse::<f64>().map_err(|_| self.err(&range_token, "Invalid min value"))?;
                    let max_val = parts[1].parse::<f64>().map_err(|_| self.err(&range_token, "Invalid max value"))?;

                    let min_expr = self.gen_greater_expression(evaluated.clone(), Expression::Number(min_val), player_selector.clone());
                    let max_expr = self.gen_greater_expression(Expression::Number(max_val), evaluated, player_selector);

                    Ok(Expression::And(vec![Box::new(min_expr), Box::new(max_expr)]))
                } else {
                    Err(self.err(&range_token, &format!("Invalid range format: {}. Expected format like '1-100'", range_str)))
                }
            }
        } else {
            Ok(Expression::SelectorGroup(player_selector, Box::new(evaluated)))
        }
    }

    pub fn parse_indexed_numeric_comparison(&mut self, evaluated: Expression, player_selector: PlayerSelector) -> Result<Expression, ParserError> {
        if self.i < self.tokens.len() && self.tokens[self.i].kind == TokenKind::square_open {
            self.i += 1;

            let mut expressions = Vec::new();
            let mut index = 0;

            while self.i < self.tokens.len() && self.tokens[self.i].kind != TokenKind::square_close {
                if self.tokens[self.i].kind == TokenKind::comma {
                    self.i += 1;
                    continue;
                }

                let indexed_eval = Expression::IndexAccess(Box::new(evaluated.clone()), Box::new(Expression::Number(index as f64)));

                if self.i < self.tokens.len() && [TokenKind::greater, TokenKind::less, TokenKind::equals].contains(&self.tokens[self.i].kind) {
                    let operator = self.tokens[self.i].clone();
                    self.i += 1;
                    let target = self.parse_expression()?;

                    if operator.kind == TokenKind::greater {
                        expressions.push(Box::new(self.gen_greater_expression(indexed_eval, target, player_selector.clone())));
                    } else if operator.kind == TokenKind::less {
                        expressions.push(Box::new(self.gen_greater_expression(target, indexed_eval, player_selector.clone())));
                    } else {
                        expressions.push(Box::new(self.gen_equivalent_expression(indexed_eval, target, player_selector.clone())));
                    }
                } else if self.i < self.tokens.len() && self.tokens[self.i].kind == TokenKind::keyword_isbetween {
                    self.i += 1;

                    if self.i < self.tokens.len() && self.tokens[self.i].kind == TokenKind::identifier {
                        let range_ident = self.tokens[self.i].literal.clone();
                        self.i += 1;

                        let range_expr = Expression::Ident(range_ident);

                        let min_expr = self.gen_greater_expression(indexed_eval.clone(), Expression::RangeMin(Box::new(range_expr.clone())), player_selector.clone());
                        let max_expr = self.gen_greater_expression(Expression::RangeMax(Box::new(range_expr)), indexed_eval, player_selector.clone());

                        expressions.push(Box::new(Expression::And(vec![Box::new(min_expr), Box::new(max_expr)])));
                    } else {
                        let range_token = self.expect_consume(TokenKind::string)?;
                        let range_str = match range_token.value {
                            TokenValue::String(ref s) => s,
                            _ => return Err(self.err(&range_token, "Expected string")),
                        };

                        let parts: Vec<&str> = range_str.split('-').collect();
                        if parts.len() == 2 {
                            let min_val = parts[0].parse::<f64>().map_err(|_| self.err(&range_token, "Invalid min value"))?;
                            let max_val = parts[1].parse::<f64>().map_err(|_| self.err(&range_token, "Invalid max value"))?;

                            let min_expr = self.gen_greater_expression(indexed_eval.clone(), Expression::Number(min_val), player_selector.clone());
                            let max_expr = self.gen_greater_expression(Expression::Number(max_val), indexed_eval, player_selector.clone());

                            expressions.push(Box::new(Expression::And(vec![Box::new(min_expr), Box::new(max_expr)])));
                        } else {
                            return Err(self.err(&range_token, &format!("Invalid range format: {}. Expected format like '1-100'", range_str)));
                        }
                    }
                } else {
                    let target = self.parse_expression()?;
                    expressions.push(Box::new(self.gen_equivalent_expression(indexed_eval, target, player_selector.clone())));
                }

                index += 1;

                if self.i < self.tokens.len() && self.tokens[self.i].kind == TokenKind::comma {
                    self.i += 1;
                }
            }

            self.expect_consume(TokenKind::square_close)?;

            if expressions.len() == 1 {
                Ok(*expressions.remove(0))
            } else {
                Ok(Expression::And(expressions))
            }
        } else {
            self.parse_numeric_comparison(Expression::IndexAccess(Box::new(evaluated), Box::new(Expression::Number(0.0))), player_selector)
        }
    }

    pub fn gen_range_check_expression(&self, value: Expression, range_ident: Expression, player_selector: PlayerSelector) -> Expression {
        Expression::SelectorGroup(player_selector, Box::new(Expression::ContainsString(Box::new(range_ident), Box::new(value))))
    }

    pub fn get_stat_eval_expression(&mut self, token_kind: TokenKind, is_percent: bool) -> Result<Expression, ParserError> {
        match token_kind {
            TokenKind::command_expr_health | TokenKind::command_expr_health_above | TokenKind::command_expr_health_below => {
                if is_percent {
                    Ok(Expression::Divide(Box::new(Expression::Eval(EvalKind::health, vec![])), Box::new(Expression::Eval(EvalKind::max_health, vec![]))))
                } else {
                    Ok(Expression::Eval(EvalKind::health, vec![]))
                }
            }
            TokenKind::command_expr_mana | TokenKind::command_expr_mana_above | TokenKind::command_expr_mana_below => {
                if is_percent {
                    Ok(Expression::Divide(Box::new(Expression::Eval(EvalKind::mana, vec![])), Box::new(Expression::Eval(EvalKind::max_mana, vec![]))))
                } else {
                    Ok(Expression::Eval(EvalKind::mana, vec![]))
                }
            }
            TokenKind::command_expr_energy | TokenKind::command_expr_energy_above | TokenKind::command_expr_energy_below => {
                if is_percent {
                    Ok(Expression::Divide(Box::new(Expression::Eval(EvalKind::energy, vec![])), Box::new(Expression::Eval(EvalKind::max_energy, vec![]))))
                } else {
                    Ok(Expression::Eval(EvalKind::energy, vec![]))
                }
            }
            TokenKind::command_expr_bagcount | TokenKind::command_expr_bagcount_above | TokenKind::command_expr_bagcount_below => {
                if is_percent {
                    Ok(Expression::Divide(Box::new(Expression::Eval(EvalKind::bagcount, vec![])), Box::new(Expression::Eval(EvalKind::max_bagcount, vec![]))))
                } else {
                    Ok(Expression::Eval(EvalKind::bagcount, vec![]))
                }
            }
            TokenKind::command_expr_gold | TokenKind::command_expr_gold_above | TokenKind::command_expr_gold_below => {
                if is_percent {
                    Ok(Expression::Divide(Box::new(Expression::Eval(EvalKind::gold, vec![])), Box::new(Expression::Eval(EvalKind::max_gold, vec![]))))
                } else {
                    Ok(Expression::Eval(EvalKind::gold, vec![]))
                }
            }
            TokenKind::command_expr_account_level => Ok(Expression::Eval(EvalKind::account_level, vec![])),
            TokenKind::command_expr_potion_count | TokenKind::command_expr_potion_countbelow | TokenKind::command_expr_potion_countabove => {
                if is_percent {
                    Ok(Expression::Divide(Box::new(Expression::Eval(EvalKind::potioncount, vec![])), Box::new(Expression::Eval(EvalKind::max_potioncount, vec![]))))
                } else {
                    Ok(Expression::Eval(EvalKind::potioncount, vec![]))
                }
            }
            TokenKind::command_expr_playercount => Ok(Expression::Eval(EvalKind::playercount, vec![])),
            TokenKind::command_expr_window_text => {
                let val = self.parse_value(Some(&["window_path"]))?;
                Ok(Expression::Eval(EvalKind::windowtext, vec![Box::new(val)]))
            }
            TokenKind::command_expr_window_num => {
                let val = self.parse_value(Some(&["window_path"]))?;
                Ok(Expression::Eval(EvalKind::windownum, vec![Box::new(val)]))
            }
            TokenKind::command_expr_duel_round => Ok(Expression::Eval(EvalKind::duel_round, vec![])),
            _ => {
                if self.i > 0 {
                    let prev_tok = &self.tokens[self.i - 1];
                    Err(self.err(prev_tok, &format!("Unexpected token kind: {:?}", token_kind)))
                } else {
                    Err(ParserError::Error(format!("Unexpected token kind at start: {:?}", token_kind)))
                }
            }
        }
    }

    pub fn parse_atom(&mut self) -> Result<Expression, ParserError> {
        if self.i < self.tokens.len() && self.tokens[self.i].kind == TokenKind::identifier && self.tokens[self.i].literal.starts_with('$') {
            let constant_name = self.tokens[self.i].literal[1..].to_string();
            self.i += 1;
            return Ok(Expression::ConstantReference(constant_name));
        }

        if self.i < self.tokens.len() && self.tokens[self.i].kind == TokenKind::boolean_true {
            let token = self.tokens[self.i].clone();
            self.i += 1;
            return Ok(Expression::Constant(token.literal, Box::new(Expression::String("true".to_string()))));
        }

        if self.i < self.tokens.len() && self.tokens[self.i].kind == TokenKind::boolean_false {
            let token = self.tokens[self.i].clone();
            self.i += 1;
            return Ok(Expression::Constant(token.literal, Box::new(Expression::String("false".to_string()))));
        }

        if self.i < self.tokens.len() && self.tokens[self.i].kind == TokenKind::square_open {
            return self.parse_list();
        }

        if self.i < self.tokens.len() && self.tokens[self.i].kind == TokenKind::path {
            return self.parse_zone_path_expression();
        }

        if self.i < self.tokens.len() && self.tokens[self.i].kind == TokenKind::keyword_xyz {
            return self.parse_xyz();
        }

        if self.i < self.tokens.len() && self.tokens[self.i].kind == TokenKind::identifier {
            let tok = self.tokens[self.i].clone();
            self.i += 1;
            return Ok(Expression::Ident(tok.literal));
        }

        let tok = self.expect_consume_any(&[TokenKind::number, TokenKind::string, TokenKind::percent])?;
        match tok.kind {
            TokenKind::number => {
                if let TokenValue::Number(n) = tok.value {
                    Ok(Expression::Number(n))
                } else {
                    Err(self.err(&tok, "Expected number value"))
                }
            }
            TokenKind::percent => {
                if let TokenValue::Percent(p) = tok.value {
                    Ok(Expression::Number(p))
                } else {
                    Err(self.err(&tok, "Expected percent value"))
                }
            }
            TokenKind::string => {
                if let TokenValue::String(ref s) = tok.value {
                    Ok(Expression::String(s.clone()))
                } else {
                    Err(self.err(&tok, "Expected string value"))
                }
            }
            _ => Err(self.err(&tok, &format!("Invalid atom kind: {:?} in {}", tok.kind, tok))),
        }
    }

    pub fn parse_unary_expression(&mut self) -> Result<Expression, ParserError> {
        let kinds = [TokenKind::minus];
        if self.i < self.tokens.len() && kinds.contains(&self.tokens[self.i].kind) {
            let operator = self.expect_consume_any(&kinds)?;
            Ok(Expression::Unary(operator, Box::new(self.parse_unary_expression()?)))
        } else {
            self.parse_atom()
        }
    }

    pub fn parse_value(&mut self, expected_types: Option<&[&str]>) -> Result<Expression, ParserError> {
        let expected = expected_types.unwrap_or(&["number", "string", "percent", "identifier"]);

        if expected.contains(&"identifier") || expected.contains(&"window_path") {
            if self.i < self.tokens.len() && self.tokens[self.i].kind == TokenKind::identifier {
                let ident = self.tokens[self.i].literal.clone();
                self.i += 1;
                return Ok(Expression::Ident(ident));
            }
        }

        if expected.contains(&"window_path") && self.i < self.tokens.len() && self.tokens[self.i].kind == TokenKind::square_open {
            return self.parse_list();
        }

        if expected.contains(&"path") {
            if self.i < self.tokens.len() {
                if self.tokens[self.i].kind == TokenKind::path {
                    return self.parse_zone_path_expression();
                } else if self.tokens[self.i].kind == TokenKind::identifier {
                    let ident = self.tokens[self.i].literal.clone();
                    self.i += 1;
                    return Ok(Expression::String(ident));
                }
            }
        }

        if self.i < self.tokens.len() && self.tokens[self.i].kind == TokenKind::keyword_xyz {
            return self.parse_xyz();
        }

        let mut valid_token_kinds = Vec::new();
        if expected.contains(&"number") { valid_token_kinds.push(TokenKind::number); }
        if expected.contains(&"string") { valid_token_kinds.push(TokenKind::string); }
        if expected.contains(&"percent") { valid_token_kinds.push(TokenKind::percent); }

        if valid_token_kinds.is_empty() {
            let current_tok = if self.i < self.tokens.len() { &self.tokens[self.i] } else if !self.tokens.is_empty() { &self.tokens[self.tokens.len() - 1] } else { return Err(ParserError::Error("No tokens available".to_string())); };
            return Err(self.err(current_tok, &format!("Expected one of {:?} but none are basic token types", expected)));
        }

        let tok = self.expect_consume_any(&valid_token_kinds)?;
        match tok.kind {
            TokenKind::number => {
                if let TokenValue::Number(n) = tok.value { Ok(Expression::Number(n)) } else { Err(self.err(&tok, "Expected number")) }
            }
            TokenKind::percent => {
                if let TokenValue::Percent(p) = tok.value { Ok(Expression::Number(p)) } else { Err(self.err(&tok, "Expected percent")) }
            }
            TokenKind::string => {
                if let TokenValue::String(ref s) = tok.value { Ok(Expression::String(s.clone())) } else { Err(self.err(&tok, "Expected string")) }
            }
            _ => Err(self.err(&tok, &format!("Invalid value kind: {:?} in {}", tok.kind, tok))),
        }
    }

    pub fn parse_numeric_stat_expression(&mut self, token_kind: TokenKind, player_selector: PlayerSelector) -> Result<Expression, ParserError> {
        self.i += 1;

        if self.i < self.tokens.len() && self.tokens[self.i].kind == TokenKind::keyword_isbetween {
            self.i += 1;

            let min_value = self.parse_value(Some(&["number", "percent", "identifier"]))?;
            let max_value = self.parse_value(Some(&["number", "percent", "identifier"]))?;

            let mut is_percent = false;
            if self.i >= 2 {
                if self.tokens[self.i - 2].kind == TokenKind::percent { is_percent = true; }
            }
            if self.i >= 1 {
                if self.tokens[self.i - 1].kind == TokenKind::percent { is_percent = true; }
            }

            let evaluated = self.get_stat_eval_expression(token_kind, is_percent)?;

            let min_expr = self.gen_greater_expression(evaluated.clone(), min_value, player_selector.clone());
            let max_expr = self.gen_greater_expression(max_value, evaluated, player_selector);

            Ok(Expression::And(vec![Box::new(min_expr), Box::new(max_expr)]))
        } else if self.i < self.tokens.len() && [TokenKind::greater, TokenKind::less, TokenKind::equals].contains(&self.tokens[self.i].kind) {
            let operator = self.tokens[self.i].clone();
            self.i += 1;

            let target = self.parse_value(Some(&["number", "percent", "identifier"]))?;
            let evaluated = self.get_stat_eval_expression(token_kind, false)?;

            if operator.kind == TokenKind::greater {
                Ok(self.gen_greater_expression(evaluated, target, player_selector))
            } else if operator.kind == TokenKind::less {
                Ok(self.gen_greater_expression(target, evaluated, player_selector))
            } else {
                Ok(self.gen_equivalent_expression(evaluated, target, player_selector))
            }
        } else {
            let value_expr = self.parse_value(Some(&["number", "percent"]))?;
            let is_percent = self.i > 0 && self.tokens[self.i - 1].kind == TokenKind::percent;
            let evaluated = self.get_stat_eval_expression(token_kind, is_percent)?;

            let above_tokens = [
                TokenKind::command_expr_health_above,
                TokenKind::command_expr_mana_above,
                TokenKind::command_expr_energy_above,
                TokenKind::command_expr_bagcount_above,
                TokenKind::command_expr_gold_above,
                TokenKind::command_expr_potion_countabove,
                TokenKind::command_expr_playercountabove,
            ];

            let below_tokens = [
                TokenKind::command_expr_health_below,
                TokenKind::command_expr_mana_below,
                TokenKind::command_expr_energy_below,
                TokenKind::command_expr_bagcount_below,
                TokenKind::command_expr_gold_below,
                TokenKind::command_expr_potion_countbelow,
                TokenKind::command_expr_playercountbelow,
            ];

            if above_tokens.contains(&token_kind) {
                Ok(self.gen_greater_expression(evaluated, value_expr, player_selector))
            } else if below_tokens.contains(&token_kind) {
                Ok(self.gen_greater_expression(value_expr, evaluated, player_selector))
            } else {
                Ok(self.gen_equivalent_expression(evaluated, value_expr, player_selector))
            }
        }
    }

    pub fn parse_command_expression(&mut self) -> Result<Expression, ParserError> {
        let player_selector = self.parse_player_selector()?;

        if self.i < self.tokens.len() && self.tokens[self.i].kind == TokenKind::identifier {
            let ident = self.tokens[self.i].literal.clone();
            self.i += 1;

            if self.i < self.tokens.len() && self.tokens[self.i].kind == TokenKind::equals {
                self.i += 1;

                if self.i < self.tokens.len() {
                    if self.tokens[self.i].kind == TokenKind::boolean_true {
                        let token = self.tokens[self.i].clone();
                        self.i += 1;
                        return Ok(Expression::ConstantCheck(ident, Box::new(Expression::Constant(token.literal, Box::new(Expression::String("true".to_string()))))));
                    } else if self.tokens[self.i].kind == TokenKind::boolean_false {
                        let token = self.tokens[self.i].clone();
                        self.i += 1;
                        return Ok(Expression::ConstantCheck(ident, Box::new(Expression::Constant(token.literal, Box::new(Expression::String("false".to_string()))))));
                    }
                }

                let value = self.parse_expression()?;
                return Ok(Expression::ConstantCheck(ident, Box::new(value)));
            } else {
                self.i -= 1;
            }
        }

        if self.i >= self.tokens.len() {
            return self.parse_unary_expression();
        }

        match self.tokens[self.i].kind {
            TokenKind::command_expr_account_level => self.parse_numeric_stat_expression(TokenKind::command_expr_account_level, player_selector),
            TokenKind::command_expr_zone_changed => {
                self.i += 1;
                let mut data = vec![Box::new(Expression::ExprKind(ExprKind::zone_changed))];
                if self.i < self.tokens.len() && self.tokens[self.i].kind == TokenKind::logical_to {
                    self.i += 1;
                    let text = self.parse_value(Some(&["path", "identifier"]))?;
                    match text {
                        Expression::String(s) => data.push(Box::new(Expression::String(s.to_lowercase()))),
                        _ => data.push(Box::new(text)),
                    }
                }
                Ok(Expression::CommandExpr(Command { kind: CommandKind::expr, data, player_selector: Some(player_selector) }))
            }
            TokenKind::command_expr_goal_changed => {
                self.i += 1;
                let mut data = vec![Box::new(Expression::ExprKind(ExprKind::goal_changed))];
                if self.i < self.tokens.len() && self.tokens[self.i].kind == TokenKind::logical_to {
                    self.i += 1;
                    let text = self.parse_value(Some(&["string", "identifier"]))?;
                    match text {
                        Expression::String(s) => data.push(Box::new(Expression::String(s.to_lowercase()))),
                        _ => data.push(Box::new(text)),
                    }
                }
                Ok(Expression::CommandExpr(Command { kind: CommandKind::expr, data, player_selector: Some(player_selector) }))
            }
            TokenKind::command_expr_quest_changed => {
                self.i += 1;
                let mut data = vec![Box::new(Expression::ExprKind(ExprKind::quest_changed))];
                if self.i < self.tokens.len() && self.tokens[self.i].kind == TokenKind::logical_to {
                    self.i += 1;
                    let text = self.parse_value(Some(&["string", "identifier"]))?;
                    match text {
                        Expression::String(s) => data.push(Box::new(Expression::String(s.to_lowercase()))),
                        _ => data.push(Box::new(text)),
                    }
                }
                Ok(Expression::CommandExpr(Command { kind: CommandKind::expr, data, player_selector: Some(player_selector) }))
            }
            TokenKind::command_expr_duel_round => self.parse_numeric_stat_expression(TokenKind::command_expr_duel_round, player_selector),
            TokenKind::command_expr_item_dropped => {
                self.i += 1;
                let mut data = vec![Box::new(Expression::ExprKind(ExprKind::items_dropped))];
                if self.i < self.tokens.len() && self.tokens[self.i].kind == TokenKind::square_open {
                    let item_list_expr = self.parse_list()?;
                    if let Expression::List(ref items) = item_list_expr {
                        let mut item_names = Vec::new();
                        for item_expr in items {
                            if let Expression::String(ref s) = **item_expr {
                                item_names.push(Box::new(Expression::String(s.to_lowercase())));
                            } else {
                                return Err(self.err(&self.tokens[self.i-1], "Expected string in item list"));
                            }
                        }
                        data.push(Box::new(Expression::List(item_names)));
                    }
                } else {
                    let item = self.parse_value(Some(&["string", "identifier"]))?;
                    match item {
                        Expression::String(s) => data.push(Box::new(Expression::String(s.to_lowercase()))),
                        _ => data.push(Box::new(item)),
                    }
                }
                Ok(Expression::CommandExpr(Command { kind: CommandKind::expr, data, player_selector: Some(player_selector) }))
            }
            TokenKind::command_expr_window_visible => {
                self.i += 1;
                let window_path = self.parse_value(Some(&["window_path"]))?;
                let data = vec![Box::new(Expression::ExprKind(ExprKind::window_visible)), Box::new(window_path)];
                Ok(Expression::CommandExpr(Command { kind: CommandKind::expr, data, player_selector: Some(player_selector) }))
            }
            TokenKind::command_expr_in_zone => {
                self.i += 1;
                let zone = self.parse_value(Some(&["path", "identifier"]))?;
                let data = vec![Box::new(Expression::ExprKind(ExprKind::in_zone)), Box::new(zone)];
                Ok(Expression::CommandExpr(Command { kind: CommandKind::expr, data, player_selector: Some(player_selector) }))
            }
            TokenKind::command_expr_same_zone => {
                self.i += 1;
                let data = vec![Box::new(Expression::ExprKind(ExprKind::same_zone))];
                Ok(Expression::CommandExpr(Command { kind: CommandKind::expr, data, player_selector: Some(player_selector) }))
            }
            TokenKind::command_expr_same_quest => {
                self.i += 1;
                let data = vec![Box::new(Expression::ExprKind(ExprKind::same_quest))];
                Ok(Expression::CommandExpr(Command { kind: CommandKind::expr, data, player_selector: Some(player_selector) }))
            }
            TokenKind::command_expr_same_xyz => {
                self.i += 1;
                let data = vec![Box::new(Expression::ExprKind(ExprKind::same_xyz))];
                Ok(Expression::CommandExpr(Command { kind: CommandKind::expr, data, player_selector: Some(player_selector) }))
            }
            TokenKind::command_expr_same_yaw => {
                self.i += 1;
                let data = vec![Box::new(Expression::ExprKind(ExprKind::same_yaw))];
                Ok(Expression::CommandExpr(Command { kind: CommandKind::expr, data, player_selector: Some(player_selector) }))
            }
            TokenKind::command_expr_in_combat => {
                self.i += 1;
                let data = vec![Box::new(Expression::ExprKind(ExprKind::in_combat))];
                Ok(Expression::CommandExpr(Command { kind: CommandKind::expr, data, player_selector: Some(player_selector) }))
            }
            TokenKind::command_expr_has_quest => {
                self.i += 1;
                let text = self.parse_value(Some(&["string", "identifier"]))?;
                let mut data = vec![Box::new(Expression::ExprKind(ExprKind::has_quest))];
                match text {
                    Expression::String(s) => data.push(Box::new(Expression::String(s.to_lowercase()))),
                    _ => data.push(Box::new(text)),
                }
                Ok(Expression::CommandExpr(Command { kind: CommandKind::expr, data, player_selector: Some(player_selector) }))
            }
            TokenKind::command_expr_has_dialogue => {
                self.i += 1;
                let data = vec![Box::new(Expression::ExprKind(ExprKind::has_dialogue))];
                Ok(Expression::CommandExpr(Command { kind: CommandKind::expr, data, player_selector: Some(player_selector) }))
            }
            TokenKind::command_expr_loading => {
                self.i += 1;
                let data = vec![Box::new(Expression::ExprKind(ExprKind::loading))];
                Ok(Expression::CommandExpr(Command { kind: CommandKind::expr, data, player_selector: Some(player_selector) }))
            }
            TokenKind::command_expr_has_xyz => {
                self.i += 1;
                let xyz = self.parse_value(Some(&["xyz", "identifier"]))?;
                let data = vec![Box::new(Expression::ExprKind(ExprKind::has_xyz)), Box::new(xyz)];
                Ok(Expression::CommandExpr(Command { kind: CommandKind::expr, data, player_selector: Some(player_selector) }))
            }
            TokenKind::command_expr_has_yaw => {
                self.i += 1;
                let yaw = self.parse_value(Some(&["number", "identifier"]))?;
                let data = vec![Box::new(Expression::ExprKind(ExprKind::has_yaw)), Box::new(yaw)];
                Ok(Expression::CommandExpr(Command { kind: CommandKind::expr, data, player_selector: Some(player_selector) }))
            }
            TokenKind::command_expr_health_above => self.parse_numeric_stat_expression(TokenKind::command_expr_health_above, player_selector),
            TokenKind::command_expr_health_below => self.parse_numeric_stat_expression(TokenKind::command_expr_health_below, player_selector),
            TokenKind::command_expr_health => self.parse_numeric_stat_expression(TokenKind::command_expr_health, player_selector),
            TokenKind::command_expr_mana_above => self.parse_numeric_stat_expression(TokenKind::command_expr_mana_above, player_selector),
            TokenKind::command_expr_mana_below => self.parse_numeric_stat_expression(TokenKind::command_expr_mana_below, player_selector),
            TokenKind::command_expr_mana => self.parse_numeric_stat_expression(TokenKind::command_expr_mana, player_selector),
            TokenKind::command_expr_energy_above => self.parse_numeric_stat_expression(TokenKind::command_expr_energy_above, player_selector),
            TokenKind::command_expr_energy_below => self.parse_numeric_stat_expression(TokenKind::command_expr_energy_below, player_selector),
            TokenKind::command_expr_energy => self.parse_numeric_stat_expression(TokenKind::command_expr_energy, player_selector),
            TokenKind::command_expr_bagcount_above => self.parse_numeric_stat_expression(TokenKind::command_expr_bagcount_above, player_selector),
            TokenKind::command_expr_bagcount_below => self.parse_numeric_stat_expression(TokenKind::command_expr_bagcount_below, player_selector),
            TokenKind::command_expr_bagcount => self.parse_numeric_stat_expression(TokenKind::command_expr_bagcount, player_selector),
            TokenKind::command_expr_gold_above => self.parse_numeric_stat_expression(TokenKind::command_expr_gold_above, player_selector),
            TokenKind::command_expr_gold_below => self.parse_numeric_stat_expression(TokenKind::command_expr_gold_below, player_selector),
            TokenKind::command_expr_gold => self.parse_numeric_stat_expression(TokenKind::command_expr_gold, player_selector),
            TokenKind::command_expr_window_text => {
                self.i += 1;
                let window_path = self.parse_window_path()?;
                let contains = self.consume_optional(TokenKind::contains).is_some();

                if self.i < self.tokens.len() && self.tokens[self.i].kind == TokenKind::square_open {
                    let string_list_expr = self.parse_list()?;
                    if contains {
                        Ok(Expression::SelectorGroup(player_selector, Box::new(Expression::ContainsString(
                            Box::new(Expression::Eval(EvalKind::windowtext, vec![Box::new(window_path)])),
                            Box::new(string_list_expr),
                        ))))
                    } else {
                        let mut or_expressions = Vec::new();
                        let window_text_eval = Expression::Eval(EvalKind::windowtext, vec![Box::new(window_path)]);
                        if let Expression::List(items) = string_list_expr {
                            for string_expr in items {
                                match *string_expr {
                                    Expression::String(ref s) => {
                                        or_expressions.push(Box::new(Expression::Equivalent(
                                            Box::new(window_text_eval.clone()),
                                            Box::new(Expression::String(s.to_lowercase())),
                                        )));
                                    }
                                    Expression::Ident(ref id) => {
                                        or_expressions.push(Box::new(Expression::Equivalent(
                                            Box::new(window_text_eval.clone()),
                                            Box::new(Expression::Ident(id.clone())),
                                        )));
                                    }
                                    _ => {}
                                }
                            }
                        }
                        if or_expressions.len() == 1 {
                            Ok(Expression::SelectorGroup(player_selector, or_expressions.remove(0)))
                        } else {
                            Ok(Expression::SelectorGroup(player_selector, Box::new(Expression::Or(or_expressions))))
                        }
                    }
                } else {
                    let target_expr = self.parse_value(Some(&["string", "identifier"]))?;
                    let window_text_eval = Expression::Eval(EvalKind::windowtext, vec![Box::new(window_path)]);
                    if contains {
                        match target_expr {
                            Expression::String(s) => Ok(Expression::SelectorGroup(player_selector, Box::new(Expression::ContainsString(Box::new(window_text_eval), Box::new(Expression::String(s.to_lowercase())))))),
                            _ => Ok(Expression::SelectorGroup(player_selector, Box::new(Expression::ContainsString(Box::new(window_text_eval), Box::new(target_expr))))),
                        }
                    } else {
                        match target_expr {
                            Expression::String(s) => Ok(Expression::SelectorGroup(player_selector, Box::new(Expression::Equivalent(Box::new(window_text_eval), Box::new(Expression::String(s.to_lowercase())))))),
                            _ => Ok(Expression::SelectorGroup(player_selector, Box::new(Expression::Equivalent(Box::new(window_text_eval), Box::new(target_expr))))),
                        }
                    }
                }
            }
            TokenKind::command_expr_window_num => {
                self.i += 1;
                let window_path = self.parse_window_path()?;
                let evaluated = Expression::Eval(EvalKind::windownum, vec![Box::new(window_path)]);
                self.parse_indexed_numeric_comparison(evaluated, player_selector)
            }
            TokenKind::command_expr_playercount => self.parse_numeric_stat_expression(TokenKind::command_expr_playercount, player_selector),
            TokenKind::command_expr_playercountabove => self.parse_numeric_stat_expression(TokenKind::command_expr_playercountabove, player_selector),
            TokenKind::command_expr_playercountbelow => self.parse_numeric_stat_expression(TokenKind::command_expr_playercountbelow, player_selector),
            TokenKind::command_expr_window_disabled => {
                self.i += 1;
                let val = self.parse_value(Some(&["window_path"]))?;
                let data = vec![Box::new(Expression::ExprKind(ExprKind::window_disabled)), Box::new(val)];
                Ok(Expression::CommandExpr(Command { kind: CommandKind::expr, data, player_selector: Some(player_selector) }))
            }
            TokenKind::command_expr_in_range => {
                self.i += 1;
                let text = self.parse_value(Some(&["string", "identifier"]))?;
                let mut data = vec![Box::new(Expression::ExprKind(ExprKind::in_range))];
                match text {
                    Expression::String(s) => data.push(Box::new(Expression::String(s.to_lowercase()))),
                    _ => data.push(Box::new(text)),
                }
                Ok(Expression::CommandExpr(Command { kind: CommandKind::expr, data, player_selector: Some(player_selector) }))
            }
            TokenKind::command_expr_same_place => {
                self.i += 1;
                let data = vec![Box::new(Expression::ExprKind(ExprKind::same_place))];
                Ok(Expression::CommandExpr(Command { kind: CommandKind::expr, data, player_selector: Some(player_selector) }))
            }
            TokenKind::command_expr_tracking_quest => {
                self.i += 1;
                let text = self.parse_value(Some(&["string"]))?;
                let mut data = vec![Box::new(Expression::ExprKind(ExprKind::tracking_quest))];
                match text {
                    Expression::String(s) => data.push(Box::new(Expression::String(s.to_lowercase()))),
                    _ => data.push(Box::new(text)),
                }
                Ok(Expression::CommandExpr(Command { kind: CommandKind::expr, data, player_selector: Some(player_selector) }))
            }
            TokenKind::command_expr_tracking_goal => {
                self.i += 1;
                let text = self.parse_value(Some(&["string"]))?;
                let mut data = vec![Box::new(Expression::ExprKind(ExprKind::tracking_goal))];
                match text {
                    Expression::String(s) => data.push(Box::new(Expression::String(s.to_lowercase()))),
                    _ => data.push(Box::new(text)),
                }
                Ok(Expression::CommandExpr(Command { kind: CommandKind::expr, data, player_selector: Some(player_selector) }))
            }
            TokenKind::command_expr_potion_count => self.parse_numeric_stat_expression(TokenKind::command_expr_potion_count, player_selector),
            TokenKind::command_expr_potion_countabove => self.parse_numeric_stat_expression(TokenKind::command_expr_potion_countabove, player_selector),
            TokenKind::command_expr_potion_countbelow => self.parse_numeric_stat_expression(TokenKind::command_expr_potion_countbelow, player_selector),
            _ => self.parse_unary_expression(),
        }
    }

    pub fn parse_negation_expression(&mut self) -> Result<Expression, ParserError> {
        let kinds = [TokenKind::keyword_not];
        if self.i < self.tokens.len() && kinds.contains(&self.tokens[self.i].kind) {
            let operator = self.expect_consume_any(&kinds)?;
            Ok(Expression::Unary(operator, Box::new(self.parse_command_expression()?)))
        } else {
            self.parse_command_expression()
        }
    }

    pub fn parse_logical_expression(&mut self) -> Result<Expression, ParserError> {
        let mut expr = self.parse_negation_expression()?;

        while self.i < self.tokens.len() && [TokenKind::keyword_and, TokenKind::keyword_or].contains(&self.tokens[self.i].kind) {
            let operator = self.tokens[self.i].clone();
            self.i += 1;
            let right = self.parse_negation_expression()?;

            if operator.kind == TokenKind::keyword_and {
                expr = Expression::And(vec![Box::new(expr), Box::new(right)]);
            } else {
                expr = Expression::Or(vec![Box::new(expr), Box::new(right)]);
            }
        }
        Ok(expr)
    }

    pub fn parse_expression(&mut self) -> Result<Expression, ParserError> {
        if self.i < self.tokens.len() && self.tokens[self.i].kind == TokenKind::logical_and {
            return Err(self.err(&self.tokens[self.i], "Expected an expression before &&"));
        }
        self.parse_logical_expression()
    }

    pub fn parse_player_selector(&mut self) -> Result<PlayerSelector, ParserError> {
        let mut result = PlayerSelector::default();
        let valid_toks = [TokenKind::keyword_same_any, TokenKind::keyword_any_player, TokenKind::keyword_mass, TokenKind::keyword_except, TokenKind::player_num, TokenKind::player_wildcard, TokenKind::colon];
        let mut expected_toks = vec![TokenKind::keyword_same_any, TokenKind::keyword_any_player, TokenKind::keyword_mass, TokenKind::keyword_except, TokenKind::player_num, TokenKind::player_wildcard];

        while self.i < self.tokens.len() && valid_toks.contains(&self.tokens[self.i].kind) {
            if !expected_toks.contains(&self.tokens[self.i].kind) {
                return Err(self.err(&self.tokens[self.i], &format!("Invalid player selector encountered: {:?}", self.tokens[self.i])));
            }
            match self.tokens[self.i].kind {
                TokenKind::keyword_same_any => {
                    result.same_any = true;
                    expected_toks.clear();
                    self.i += 1;
                }
                TokenKind::keyword_any_player => {
                    result.any_player = true;
                    expected_toks.clear();
                    self.i += 1;
                }
                TokenKind::keyword_mass => {
                    result.mass = true;
                    expected_toks.clear();
                    self.i += 1;
                }
                TokenKind::keyword_except => {
                    result.inverted = true;
                    expected_toks = vec![TokenKind::player_num];
                    self.i += 1;
                }
                TokenKind::player_num => {
                    if let TokenValue::Int(n) = self.tokens[self.i].value {
                        result.player_nums.push(n);
                    }
                    expected_toks = vec![TokenKind::colon];
                    self.i += 1;
                }
                TokenKind::player_wildcard => {
                    result.wildcard = true;
                    expected_toks.clear();
                    self.i += 1;
                }
                TokenKind::colon => {
                    expected_toks = vec![TokenKind::player_num];
                    self.i += 1;
                }
                _ => unreachable!(),
            }
        }
        result.validate();
        if result.player_nums.is_empty() && !result.wildcard && !result.any_player && !result.same_any {
            result.mass = true;
        }
        result.validate();
        Ok(result)
    }

    pub fn parse_key(&mut self) -> Result<Expression, ParserError> {
        let tok = self.expect_consume_any(&[TokenKind::identifier, TokenKind::command_kill])?;
        Ok(Expression::Key(tok.literal))
    }

    pub fn parse_xyz(&mut self) -> Result<Expression, ParserError> {
        let start_tok = self.expect_consume(TokenKind::keyword_xyz)?;
        let mut vals = Vec::new();
        let valid_toks = [TokenKind::paren_open, TokenKind::paren_close, TokenKind::comma, TokenKind::number, TokenKind::minus];
        let mut expected_toks = vec![TokenKind::paren_open];
        let mut found_closing = false;

        while self.i < self.tokens.len() && valid_toks.contains(&self.tokens[self.i].kind) {
            if !expected_toks.contains(&self.tokens[self.i].kind) {
                return Err(self.err(&self.tokens[self.i], "Invalid xyz encountered"));
            }
            match self.tokens[self.i].kind {
                TokenKind::paren_open => {
                    self.i += 1;
                    expected_toks = vec![TokenKind::comma, TokenKind::number, TokenKind::paren_close, TokenKind::minus];
                }
                TokenKind::paren_close => {
                    self.i += 1;
                    expected_toks.clear();
                    found_closing = true;
                }
                TokenKind::comma | TokenKind::number | TokenKind::minus => {
                    if self.tokens[self.i].kind == TokenKind::comma {
                        vals.push(Box::new(Expression::Number(0.0)));
                        self.i += 1;
                    } else {
                        vals.push(Box::new(self.parse_expression()?));
                        if self.i < self.tokens.len() && self.tokens[self.i].kind == TokenKind::comma {
                            self.i += 1;
                        }
                    }
                    expected_toks = vec![TokenKind::comma, TokenKind::paren_close, TokenKind::number, TokenKind::minus];
                }
                _ => unreachable!(),
            }
        }
        if !found_closing {
            return Err(self.err(&start_tok, "Encountered unclosed XYZ"));
        }
        if vals.len() != 3 {
            return Err(self.err(&start_tok, "Encountered invalid XYZ"));
        }
        let z = vals.pop().unwrap();
        let y = vals.pop().unwrap();
        let x = vals.pop().unwrap();
        Ok(Expression::XYZ(x, y, z))
    }

    pub fn parse_completion_optional(&mut self) -> bool {
        if self.i < self.tokens.len() && self.tokens[self.i].kind == TokenKind::keyword_completion {
            self.i += 1;
            return true;
        }
        false
    }

    pub fn parse_zone_path_optional(&mut self) -> Option<Vec<String>> {
        if self.i < self.tokens.len() && self.tokens[self.i].kind == TokenKind::path {
            let tok = self.tokens[self.i].clone();
            self.i += 1;
            if let TokenValue::Path(p) = tok.value {
                return Some(p);
            }
        }
        None
    }

    pub fn parse_zone_path(&mut self) -> Result<Vec<String>, ParserError> {
        let res = self.parse_zone_path_optional();
        match res {
            Some(p) => Ok(p),
            None => {
                let tok = if self.i < self.tokens.len() { &self.tokens[self.i] } else if !self.tokens.is_empty() { &self.tokens[self.tokens.len() - 1] } else { return Err(ParserError::Error("No tokens available".to_string())); };
                Err(self.err(tok, "Failed to parse zone path"))
            }
        }
    }

    pub fn parse_zone_path_expression(&mut self) -> Result<Expression, ParserError> {
        let tok = self.expect_consume(TokenKind::path)?;
        Ok(Expression::String(tok.literal))
    }

    pub fn parse_list(&mut self) -> Result<Expression, ParserError> {
        self.expect_consume(TokenKind::square_open)?;
        let mut items = Vec::new();
        while self.i < self.tokens.len() && self.tokens[self.i].kind != TokenKind::square_close {
            if self.tokens[self.i].kind == TokenKind::comma {
                self.i += 1;
                continue;
            }
            items.push(Box::new(self.parse_expression()?));
            if self.i < self.tokens.len() && self.tokens[self.i].kind != TokenKind::square_close {
                self.expect_consume(TokenKind::comma)?;
            }
        }
        self.expect_consume(TokenKind::square_close)?;
        Ok(Expression::List(items))
    }

    pub fn parse_window_path(&mut self) -> Result<Expression, ParserError> {
        if self.i < self.tokens.len() && self.tokens[self.i].kind == TokenKind::identifier && self.tokens[self.i].literal.starts_with('$') {
            let ident = self.tokens[self.i].literal.clone();
            self.i += 1;
            let const_name = ident[1..].to_string();
            return Ok(Expression::Ident(const_name));
        }

        let list_expr = self.parse_list()?;
        Ok(list_expr)
    }

    pub fn end_line(&mut self) -> Result<(), ParserError> {
        if self.i < self.tokens.len() && self.tokens[self.i].kind == TokenKind::logical_and {
            return Ok(());
        }
        self.expect_consume(TokenKind::END_LINE)?;
        Ok(())
    }

    pub fn end_line_optional(&mut self) {
        if self.i < self.tokens.len() && self.tokens[self.i].kind == TokenKind::END_LINE {
            self.i += 1;
        }
    }

    pub fn parse_command(&mut self) -> Result<Stmt, ParserError> {
        let mut commands = Vec::new();
        commands.push(self._parse_simple_command()?);

        while self.i < self.tokens.len() && self.tokens[self.i].kind == TokenKind::logical_and {
            self.i += 1;
            commands.push(self._parse_simple_command()?);
        }

        if commands.len() == 1 {
            Ok(Stmt::Command(commands.remove(0)))
        } else {
            Ok(Stmt::ParallelCommand(commands))
        }
    }

    fn _parse_simple_command(&mut self) -> Result<Command, ParserError> {
        let mut result = Command { kind: CommandKind::invalid, data: Vec::new(), player_selector: None };
        result.player_selector = Some(self.parse_player_selector()?);

        if self.i >= self.tokens.len() {
             return Err(ParserError::Error("Premature end of file while parsing command".to_string()));
        }

        match self.tokens[self.i].kind {
            TokenKind::command_restart_bot => {
                result.kind = CommandKind::restart_bot;
                self.i += 1;
                self.end_line()?;
            }
            TokenKind::command_toggle_combat => {
                result.kind = CommandKind::toggle_combat;
                self.i += 1;
                if self.i < self.tokens.len() && [TokenKind::logical_on, TokenKind::logical_off].contains(&self.tokens[self.i].kind) {
                    let tok = self.expect_consume_any(&[TokenKind::logical_on, TokenKind::logical_off])?;
                    result.data.push(Box::new(Expression::String(tok.literal)));
                }
                self.end_line()?;
            }
            TokenKind::command_set_zone => {
                result.kind = CommandKind::set_zone;
                self.i += 1;
                self.end_line()?;
            }
            TokenKind::command_set_goal => {
                result.kind = CommandKind::set_goal;
                self.i += 1;
                self.end_line()?;
            }
            TokenKind::command_set_quest => {
                result.kind = CommandKind::set_quest;
                self.i += 1;
                self.end_line()?;
            }
            TokenKind::command_autopet => {
                result.kind = CommandKind::autopet;
                self.i += 1;
                self.end_line()?;
            }
            TokenKind::command_kill => {
                result.kind = CommandKind::kill;
                self.i += 1;
                self.end_line()?;
            }
            TokenKind::command_log => {
                self.i += 1;
                if self.i >= self.tokens.len() { return Err(ParserError::Error("Premature end of file while parsing log command".to_string())); }
                let kind_tok = self.tokens[self.i].clone();
                result.kind = CommandKind::log;
                
                let mut handle_log = |p: &mut Parser| -> Result<(), ParserError> {
                    match kind_tok.kind {
                        TokenKind::identifier => {
                            if kind_tok.literal == "window" {
                                p.i += 1;
                                let window_path = p.parse_window_path()?;
                                result.data = vec![Box::new(Expression::LogKind(LogKind::multi)), Box::new(Expression::StrFormat("windowtext: %s".to_string(), vec![Box::new(Expression::Eval(EvalKind::windowtext, vec![Box::new(window_path)]))]))];
                            } else if kind_tok.literal.starts_with('$') {
                                p.i += 1;
                                let const_name = kind_tok.literal[1..].to_string();
                                result.data = vec![Box::new(Expression::LogKind(LogKind::single)), Box::new(Expression::Ident(const_name))];
                            } else {
                                let mut final_str = String::new();
                                while p.i < p.tokens.len() && p.tokens[p.i].kind != TokenKind::END_LINE {
                                    let tok = &p.tokens[p.i];
                                    match tok.kind {
                                        TokenKind::string => {
                                            if let TokenValue::String(ref s) = tok.value {
                                                final_str.push_str(s);
                                                final_str.push(' ');
                                            }
                                        }
                                        _ => {
                                            final_str.push_str(&tok.literal);
                                            final_str.push(' ');
                                        }
                                    }
                                    p.i += 1;
                                }
                                result.data = vec![Box::new(Expression::LogKind(LogKind::single)), Box::new(Expression::String(final_str))];
                            }
                        }
                        TokenKind::command_expr_bagcount => {
                            p.i += 1;
                            result.data = vec![Box::new(Expression::LogKind(LogKind::multi)), Box::new(Expression::StrFormat("bagcount: %d/%d".to_string(), vec![Box::new(Expression::Eval(EvalKind::bagcount, vec![])), Box::new(Expression::Eval(EvalKind::max_bagcount, vec![]))]))];
                        }
                        TokenKind::command_expr_mana => {
                            p.i += 1;
                            result.data = vec![Box::new(Expression::LogKind(LogKind::multi)), Box::new(Expression::StrFormat("mana: %d/%d".to_string(), vec![Box::new(Expression::Eval(EvalKind::mana, vec![])), Box::new(Expression::Eval(EvalKind::max_mana, vec![]))]))];
                        }
                        TokenKind::command_expr_energy => {
                            p.i += 1;
                            result.data = vec![Box::new(Expression::LogKind(LogKind::multi)), Box::new(Expression::StrFormat("energy: %d/%d".to_string(), vec![Box::new(Expression::Eval(EvalKind::energy, vec![])), Box::new(Expression::Eval(EvalKind::max_energy, vec![]))]))];
                        }
                        TokenKind::command_expr_health => {
                            p.i += 1;
                            result.data = vec![Box::new(Expression::LogKind(LogKind::multi)), Box::new(Expression::StrFormat("health: %d/%d".to_string(), vec![Box::new(Expression::Eval(EvalKind::health, vec![])), Box::new(Expression::Eval(EvalKind::max_health, vec![]))]))];
                        }
                        TokenKind::command_expr_gold => {
                            p.i += 1;
                            result.data = vec![Box::new(Expression::LogKind(LogKind::multi)), Box::new(Expression::StrFormat("gold: %d/%d".to_string(), vec![Box::new(Expression::Eval(EvalKind::gold, vec![])), Box::new(Expression::Eval(EvalKind::max_gold, vec![]))]))];
                        }
                        TokenKind::command_expr_potion_count => {
                            p.i += 1;
                            result.data = vec![Box::new(Expression::LogKind(LogKind::multi)), Box::new(Expression::StrFormat("potioncount: %d/%d".to_string(), vec![Box::new(Expression::Eval(EvalKind::potioncount, vec![])), Box::new(Expression::Eval(EvalKind::max_potioncount, vec![]))]))];
                        }
                        TokenKind::command_expr_playercount => {
                            p.i += 1;
                            result.data = vec![Box::new(Expression::LogKind(LogKind::single)), Box::new(Expression::StrFormat("playercount: %d".to_string(), vec![Box::new(Expression::Eval(EvalKind::playercount, vec![]))]))];
                        }
                        TokenKind::command_expr_window_text => {
                            p.i += 1;
                            let window_path = p.parse_window_path()?;
                            result.data = vec![Box::new(Expression::LogKind(LogKind::multi)), Box::new(Expression::StrFormat("windowtext: %s".to_string(), vec![Box::new(Expression::Eval(EvalKind::windowtext, vec![Box::new(window_path)]))]))];
                        }
                        TokenKind::command_expr_any_player_list => {
                            p.i += 1;
                            result.data = vec![Box::new(Expression::LogKind(LogKind::single)), Box::new(Expression::StrFormat("clients using anyplayer: %s".to_string(), vec![Box::new(Expression::Eval(EvalKind::any_player_list, vec![]))]))];
                        }
                        _ => {
                            let mut final_str = String::new();
                            while p.i < p.tokens.len() && p.tokens[p.i].kind != TokenKind::END_LINE {
                                let tok = &p.tokens[p.i];
                                match tok.kind {
                                    TokenKind::string => {
                                        if let TokenValue::String(ref s) = tok.value {
                                            final_str.push_str(s);
                                            final_str.push(' ');
                                        }
                                    }
                                    _ => {
                                        final_str.push_str(&tok.literal);
                                        final_str.push(' ');
                                    }
                                }
                                p.i += 1;
                            }
                            result.data = vec![Box::new(Expression::LogKind(LogKind::single)), Box::new(Expression::String(final_str))];
                        }
                    }
                    Ok(())
                };
                handle_log(self)?;
                self.end_line()?;
            }
            TokenKind::command_teleport => {
                result.kind = CommandKind::teleport;
                self.i += 1;
                if self.consume_optional(TokenKind::keyword_mob).is_some() {
                    result.data.push(Box::new(Expression::TeleportKind(TeleportKind::mob)));
                } else if self.consume_optional(TokenKind::keyword_quest).is_some() {
                    result.data.push(Box::new(Expression::TeleportKind(TeleportKind::quest)));
                } else if let Some(num_tok) = self.consume_optional(TokenKind::player_num) {
                    if let TokenValue::Int(n) = num_tok.value {
                        result.data.push(Box::new(Expression::TeleportKind(TeleportKind::client_num)));
                        result.data.push(Box::new(Expression::Number(n as f64)));
                    }
                } else {
                    if self.i < self.tokens.len() && self.tokens[self.i].kind == TokenKind::string {
                        result.data.push(Box::new(Expression::TeleportKind(TeleportKind::position)));
                        // BUG: (from Python original) parse_xyz() expects keyword_xyz, but this branch is taken on TokenKind::string
                        result.data.push(Box::new(self.parse_xyz()?));
                    } else {
                        result.data.push(Box::new(Expression::TeleportKind(TeleportKind::position)));
                        result.data.push(Box::new(self.parse_expression()?));
                    }
                }
                self.end_line()?;
            }
            TokenKind::command_plus_teleport => {
                result.kind = CommandKind::teleport;
                self.i += 1;
                result.data.push(Box::new(Expression::TeleportKind(TeleportKind::plusteleport)));
                if self.i < self.tokens.len() && self.tokens[self.i].kind == TokenKind::string {
                    // BUG: (from Python original) parse_xyz() expects keyword_xyz, but this branch is taken on TokenKind::string
                    result.data.push(Box::new(self.parse_xyz()?));
                } else {
                    result.data.push(Box::new(self.parse_expression()?));
                }
                self.end_line()?;
            }
            TokenKind::command_minus_teleport => {
                result.kind = CommandKind::teleport;
                self.i += 1;
                result.data.push(Box::new(Expression::TeleportKind(TeleportKind::minusteleport)));
                if self.i < self.tokens.len() && self.tokens[self.i].kind == TokenKind::string {
                    // BUG: (from Python original) parse_xyz() expects keyword_xyz, but this branch is taken on TokenKind::string
                    result.data.push(Box::new(self.parse_xyz()?));
                } else {
                    result.data.push(Box::new(self.parse_expression()?));
                }
                self.end_line()?;
            }
            TokenKind::command_sleep => {
                result.kind = CommandKind::sleep;
                self.i += 1;
                result.data.push(Box::new(self.parse_expression()?));
                self.end_line()?;
            }
            TokenKind::command_sendkey => {
                result.kind = CommandKind::sendkey;
                self.i += 1;
                result.data.push(Box::new(self.parse_key()?));
                self.skip_comma();
                if self.i < self.tokens.len() && self.tokens[self.i].kind != TokenKind::END_LINE {
                    result.data.push(Box::new(self.parse_expression()?));
                }
                self.end_line()?;
            }
            TokenKind::command_waitfor_zonechange => {
                result.kind = CommandKind::waitfor;
                self.i += 1;
                result.data.push(Box::new(Expression::WaitforKind(WaitforKind::zonechange)));
                result.data.push(Box::new(Expression::Boolean(self.parse_completion_optional())));
                self.end_line()?;
            }
            TokenKind::command_waitfor_battle => {
                result.kind = CommandKind::waitfor;
                self.i += 1;
                result.data.push(Box::new(Expression::WaitforKind(WaitforKind::battle)));
                result.data.push(Box::new(Expression::Boolean(self.parse_completion_optional())));
                self.end_line()?;
            }
            TokenKind::command_waitfor_window => {
                result.kind = CommandKind::waitfor;
                self.i += 1;
                result.data.push(Box::new(Expression::WaitforKind(WaitforKind::window)));
                result.data.push(Box::new(self.parse_window_path()?));
                result.data.push(Box::new(Expression::Boolean(self.parse_completion_optional())));
                self.end_line()?;
            }
            TokenKind::command_waitfor_free => {
                result.kind = CommandKind::waitfor;
                self.i += 1;
                result.data.push(Box::new(Expression::WaitforKind(WaitforKind::free)));
                result.data.push(Box::new(Expression::Boolean(self.parse_completion_optional())));
                self.end_line()?;
            }
            TokenKind::command_waitfor_dialog => {
                result.kind = CommandKind::waitfor;
                self.i += 1;
                result.data.push(Box::new(Expression::WaitforKind(WaitforKind::dialog)));
                result.data.push(Box::new(Expression::Boolean(self.parse_completion_optional())));
                self.end_line()?;
            }
            TokenKind::command_goto => {
                result.kind = CommandKind::goto;
                self.i += 1;
                if self.i < self.tokens.len() && self.tokens[self.i].kind == TokenKind::string {
                    result.data.push(Box::new(self.parse_xyz()?));
                } else {
                    result.data.push(Box::new(self.parse_expression()?));
                }
                self.end_line()?;
            }
            TokenKind::command_move_cursor_window => {
                result.kind = CommandKind::cursor;
                self.i += 1;
                result.data.push(Box::new(Expression::CursorKind(CursorKind::window)));
                if self.i < self.tokens.len() && self.tokens[self.i].kind == TokenKind::string {
                    result.data.push(Box::new(self.parse_window_path()?));
                } else {
                    result.data.push(Box::new(self.parse_expression()?));
                }
                self.end_line()?;
            }
            TokenKind::command_clickwindow => {
                result.kind = CommandKind::click;
                self.i += 1;
                result.data.push(Box::new(Expression::ClickKind(ClickKind::window)));
                if self.i < self.tokens.len() && self.tokens[self.i].kind == TokenKind::string {
                    result.data.push(Box::new(self.parse_window_path()?));
                } else {
                    result.data.push(Box::new(self.parse_expression()?));
                }
                self.end_line()?;
            }
            TokenKind::command_usepotion => {
                result.kind = CommandKind::usepotion;
                self.i += 1;
                let mut health_expr = None;
                if self.i < self.tokens.len() && self.tokens[self.i].kind == TokenKind::number {
                    if let TokenValue::Number(n) = self.expect_consume(TokenKind::number)?.value {
                        health_expr = Some(Box::new(Expression::Number(n)));
                    }
                } else if self.i < self.tokens.len() && self.tokens[self.i].kind == TokenKind::identifier {
                    health_expr = Some(Box::new(Expression::Ident(self.expect_consume(TokenKind::identifier)?.literal)));
                }

                if let Some(h) = health_expr {
                    result.data.push(h);
                    self.skip_comma();
                    if self.i < self.tokens.len() && self.tokens[self.i].kind == TokenKind::number {
                        if let TokenValue::Number(n) = self.expect_consume(TokenKind::number)?.value {
                            result.data.push(Box::new(Expression::Number(n)));
                        }
                    } else if self.i < self.tokens.len() && self.tokens[self.i].kind == TokenKind::identifier {
                        result.data.push(Box::new(Expression::Ident(self.expect_consume(TokenKind::identifier)?.literal)));
                    }
                }
                self.end_line()?;
            }
            TokenKind::command_buypotions => {
                result.kind = CommandKind::buypotions;
                self.i += 1;
                let if_needed = self.consume_optional(TokenKind::keyword_ifneeded).is_some();
                result.data.push(Box::new(Expression::Boolean(if_needed)));
                self.end_line()?;
            }
            TokenKind::command_relog => {
                result.kind = CommandKind::relog;
                self.i += 1;
                self.end_line()?;
            }
            TokenKind::command_move_cursor => {
                result.kind = CommandKind::cursor;
                self.i += 1;
                result.data.push(Box::new(Expression::CursorKind(CursorKind::position)));
                let mut x_expr = None;
                if self.i < self.tokens.len() && self.tokens[self.i].kind == TokenKind::number {
                    if let TokenValue::Number(n) = self.expect_consume(TokenKind::number)?.value {
                        x_expr = Some(Box::new(Expression::Number(n)));
                    }
                } else if self.i < self.tokens.len() && self.tokens[self.i].kind == TokenKind::identifier {
                    x_expr = Some(Box::new(Expression::Ident(self.expect_consume(TokenKind::identifier)?.literal)));
                }

                if let Some(x) = x_expr {
                    result.data.push(x);
                    self.skip_comma();
                    if self.i < self.tokens.len() && self.tokens[self.i].kind == TokenKind::number {
                        if let TokenValue::Number(n) = self.expect_consume(TokenKind::number)?.value {
                            result.data.push(Box::new(Expression::Number(n)));
                        }
                    } else if self.i < self.tokens.len() && self.tokens[self.i].kind == TokenKind::identifier {
                        result.data.push(Box::new(Expression::Ident(self.expect_consume(TokenKind::identifier)?.literal)));
                    }
                }
                self.end_line()?;
            }
            TokenKind::command_click => {
                result.kind = CommandKind::click;
                self.i += 1;
                result.data.push(Box::new(Expression::ClickKind(ClickKind::position)));
                let mut x_expr = None;
                if self.i < self.tokens.len() && self.tokens[self.i].kind == TokenKind::number {
                    if let TokenValue::Number(n) = self.expect_consume(TokenKind::number)?.value {
                        x_expr = Some(Box::new(Expression::Number(n)));
                    }
                } else if self.i < self.tokens.len() && self.tokens[self.i].kind == TokenKind::identifier {
                    x_expr = Some(Box::new(Expression::Ident(self.expect_consume(TokenKind::identifier)?.literal)));
                }

                if let Some(x) = x_expr {
                    result.data.push(x);
                    self.skip_comma();
                    if self.i < self.tokens.len() && self.tokens[self.i].kind == TokenKind::number {
                        if let TokenValue::Number(n) = self.expect_consume(TokenKind::number)?.value {
                            result.data.push(Box::new(Expression::Number(n)));
                        }
                    } else if self.i < self.tokens.len() && self.tokens[self.i].kind == TokenKind::identifier {
                        result.data.push(Box::new(Expression::Ident(self.expect_consume(TokenKind::identifier)?.literal)));
                    }
                }
                self.end_line()?;
            }
            TokenKind::command_friendtp => {
                result.kind = CommandKind::teleport;
                self.i += 1;
                let x = self.expect_consume_any(&[TokenKind::keyword_icon, TokenKind::identifier])?;
                if x.kind == TokenKind::keyword_icon {
                    result.data.push(Box::new(Expression::TeleportKind(TeleportKind::friend_icon)));
                } else if self.i < self.tokens.len() && self.tokens[self.i].kind == TokenKind::END_LINE {
                    result.data.push(Box::new(Expression::TeleportKind(TeleportKind::friend_name)));
                    result.data.push(Box::new(Expression::Ident(x.literal)));
                } else {
                    let mut name_parts = vec![x.literal];
                    while self.i < self.tokens.len() && self.tokens[self.i].kind != TokenKind::END_LINE {
                        name_parts.push(self.tokens[self.i].literal.clone());
                        self.i += 1;
                    }
                    result.data.push(Box::new(Expression::TeleportKind(TeleportKind::friend_name)));
                    result.data.push(Box::new(Expression::String(name_parts.join(" "))));
                }
                self.end_line()?;
            }
            TokenKind::command_entitytp => {
                result.kind = CommandKind::teleport;
                self.i += 1;
                let mut nav_mode = false;
                if self.i < self.tokens.len() && self.tokens[self.i].kind == TokenKind::command_nav {
                    self.i += 1;
                    nav_mode = true;
                }

                let arg = self.consume_optional(TokenKind::string);
                if let Some(a) = arg {
                    if let TokenValue::String(ref s) = a.value {
                        result.data.push(Box::new(Expression::TeleportKind(TeleportKind::entity_literal)));
                        if nav_mode { result.data.push(Box::new(Expression::TeleportKind(TeleportKind::nav))); }
                        result.data.push(Box::new(Expression::String(s.clone())));
                    }
                } else if self.i < self.tokens.len() && self.tokens[self.i].kind == TokenKind::identifier {
                    let ident = self.expect_consume(TokenKind::identifier)?;
                    result.data.push(Box::new(Expression::TeleportKind(TeleportKind::entity_vague)));
                    if nav_mode { result.data.push(Box::new(Expression::TeleportKind(TeleportKind::nav))); }
                    result.data.push(Box::new(Expression::String(ident.literal)));
                } else if self.i < self.tokens.len() {
                    let token = self.tokens[self.i].clone();
                    self.i += 1;
                    result.data.push(Box::new(Expression::TeleportKind(TeleportKind::entity_vague)));
                    if nav_mode { result.data.push(Box::new(Expression::TeleportKind(TeleportKind::nav))); }
                    result.data.push(Box::new(Expression::String(token.literal)));
                }
                self.end_line()?;
            }
            TokenKind::command_tozone => {
                result.kind = CommandKind::tozone;
                self.i += 1;
                if self.i < self.tokens.len() && self.tokens[self.i].kind == TokenKind::path {
                    let path = self.parse_zone_path()?;
                    let mut list_items = Vec::new();
                    for p in path { list_items.push(Box::new(Expression::String(p))); }
                    result.data.push(Box::new(Expression::List(list_items)));
                } else if self.i < self.tokens.len() && self.tokens[self.i].kind == TokenKind::identifier {
                    result.data.push(Box::new(Expression::Ident(self.expect_consume(TokenKind::identifier)?.literal)));
                } else {
                    result.data.push(Box::new(self.parse_expression()?));
                }
                self.end_line()?;
            }
            TokenKind::command_load_playstyle => {
                result.kind = CommandKind::load_playstyle;
                self.i += 1;
                if self.i < self.tokens.len() && self.tokens[self.i].kind == TokenKind::string {
                    if let TokenValue::String(ref s) = self.expect_consume(TokenKind::string)?.value {
                        result.data.push(Box::new(Expression::String(s.clone())));
                    }
                } else if self.i < self.tokens.len() && self.tokens[self.i].kind == TokenKind::identifier {
                    result.data.push(Box::new(Expression::Ident(self.expect_consume(TokenKind::identifier)?.literal)));
                } else {
                    result.data.push(Box::new(self.parse_expression()?));
                }
                self.end_line()?;
            }
            TokenKind::command_set_yaw => {
                result.kind = CommandKind::set_yaw;
                self.i += 1;
                if self.i < self.tokens.len() && self.tokens[self.i].kind == TokenKind::number {
                    if let TokenValue::Number(n) = self.expect_consume(TokenKind::number)?.value {
                        result.data.push(Box::new(Expression::Number(n)));
                    }
                } else if self.i < self.tokens.len() && self.tokens[self.i].kind == TokenKind::identifier {
                    result.data.push(Box::new(Expression::Ident(self.expect_consume(TokenKind::identifier)?.literal)));
                } else {
                    result.data.push(Box::new(self.parse_expression()?));
                }
                self.end_line()?;
            }
            TokenKind::command_select_friend => {
                result.kind = CommandKind::select_friend;
                self.i += 1;
                if self.i < self.tokens.len() && self.tokens[self.i].kind == TokenKind::identifier && self.i + 1 < self.tokens.len() && self.tokens[self.i + 1].kind == TokenKind::END_LINE {
                    result.data.push(Box::new(Expression::String(self.expect_consume(TokenKind::identifier)?.literal)));
                } else {
                    let mut name_parts = Vec::new();
                    while self.i < self.tokens.len() && self.tokens[self.i].kind != TokenKind::END_LINE {
                        name_parts.push(self.tokens[self.i].literal.clone());
                        self.i += 1;
                    }
                    result.data.push(Box::new(Expression::String(name_parts.join(" "))));
                }
                self.end_line()?;
            }
            TokenKind::command_setdeck => {
                result.kind = CommandKind::setdeck;
                self.i += 1;
                let tok = self.expect_consume(TokenKind::string)?;
                if let TokenValue::String(ref s) = tok.value {
                    result.data.push(Box::new(Expression::String(s.clone())));
                }
                self.end_line()?;
            }
            TokenKind::command_getdeck => {
                result.kind = CommandKind::getdeck;
                self.i += 1;
                self.end_line()?;
            }
            _ => {
                let current_tok = if self.i < self.tokens.len() { &self.tokens[self.i] } else if !self.tokens.is_empty() { &self.tokens[self.tokens.len() - 1] } else { return Err(ParserError::Error("No tokens available".to_string())); };
                return Err(self.err(current_tok, "Unhandled command token"));
            }
        }
        Ok(result)
    }

    pub fn parse_block(&mut self) -> Result<Stmt, ParserError> {
        let mut inner = Vec::new();
        self.expect_consume(TokenKind::curly_open)?;
        self.end_line_optional();
        while self.i < self.tokens.len() && self.tokens[self.i].kind != TokenKind::curly_close {
            inner.push(self.parse_stmt()?);
        }
        self.expect_consume(TokenKind::curly_close)?;
        self.end_line_optional();
        Ok(Stmt::StmtList(inner))
    }

    fn consume_any_ident(&mut self) -> Result<Expression, ParserError> {
        if self.i >= self.tokens.len() {
             return Err(ParserError::Error("Premature end of file while looking for identifier".to_string()));
        }
        let result = self.tokens[self.i].clone();
        if result.kind != TokenKind::identifier && !format!("{:?}", result.kind).contains("keyword") && !format!("{:?}", result.kind).contains("command") {
            return Err(self.err(&result, "Unable to consume an identifier"));
        }
        self.i += 1;
        Ok(Expression::Ident(result.literal))
    }

    pub fn parse_stmt(&mut self) -> Result<Stmt, ParserError> {
        if self.i >= self.tokens.len() {
            return Err(ParserError::Error("Premature end of file while parsing statement".to_string()));
        }
        match self.tokens[self.i].kind {
            TokenKind::keyword_con => {
                self.i += 1;
                let var_name = self.expect_consume(TokenKind::identifier)?.literal;
                self.expect_consume(TokenKind::equals)?;
                let expr = self.parse_expression()?;
                self.end_line()?;
                Ok(Stmt::ConstantDecl(var_name, Box::new(expr)))
            }
            TokenKind::keyword_settimer => {
                self.i += 1;
                let timer_name = self.consume_any_ident()?;
                let name = match timer_name {
                    Expression::Ident(s) => s,
                    _ => unreachable!(),
                };
                self.end_line()?;
                Ok(Stmt::Timer(TimerAction::start, name))
            }
            TokenKind::keyword_endtimer => {
                self.i += 1;
                let timer_name = self.consume_any_ident()?;
                let name = match timer_name {
                    Expression::Ident(s) => s,
                    _ => unreachable!(),
                };
                self.end_line()?;
                Ok(Stmt::Timer(TimerAction::end, name))
            }
            TokenKind::keyword_block => {
                self.i += 1;
                let ident = self.consume_any_ident()?;
                let body = self.parse_block()?;
                Ok(Stmt::BlockDef(Box::new(ident), Box::new(body), Vec::new()))
            }
            TokenKind::keyword_call => {
                self.i += 1;
                let ident = self.consume_any_ident()?;
                self.end_line()?;
                Ok(Stmt::Call(Box::new(ident)))
            }
            TokenKind::keyword_loop => {
                self.i += 1;
                let body = self.parse_block()?;
                Ok(Stmt::Loop(Box::new(body)))
            }
            TokenKind::keyword_while => {
                self.i += 1;
                let expr = self.parse_expression()?;
                let body = self.parse_block()?;
                Ok(Stmt::While(Box::new(expr), Box::new(body)))
            }
            TokenKind::keyword_until => {
                self.i += 1;
                let expr = self.parse_expression()?;
                let body = self.parse_block()?;
                Ok(Stmt::Until(Box::new(expr), Box::new(body)))
            }
            TokenKind::keyword_times => {
                self.i += 1;
                let tok = self.expect_consume(TokenKind::number)?;
                let count = if let TokenValue::Number(n) = tok.value { n as i32 } else { 0 };
                let body = self.parse_block()?;
                Ok(Stmt::Times(count, Box::new(body)))
            }
            TokenKind::keyword_if => {
                self.i += 1;
                let expr = self.parse_expression()?;
                let true_body = self.parse_block()?;
                let mut elif_body_stack: Vec<Stmt> = Vec::new();
                let mut else_body = Stmt::StmtList(Vec::new());

                while self.i < self.tokens.len() && [TokenKind::keyword_else, TokenKind::keyword_elif].contains(&self.tokens[self.i].kind) {
                    if self.tokens[self.i].kind == TokenKind::keyword_else {
                        self.i += 1;
                        else_body = self.parse_block()?;
                        break;
                    } else if self.tokens[self.i].kind == TokenKind::keyword_elif {
                        self.i += 1;
                        let elif_expr = self.parse_expression()?;
                        let elif_body = self.parse_block()?;
                        let elif_stmt = Stmt::If(Box::new(elif_expr), Box::new(elif_body), Box::new(Stmt::StmtList(Vec::new())));
                        elif_body_stack.push(elif_stmt);
                    }
                }

                let mut final_else = else_body;
                while let Some(mut elif) = elif_body_stack.pop() {
                    if let Stmt::If(_, _, ref mut branch_false) = elif {
                        **branch_false = final_else;
                    }
                    final_else = elif;
                }

                Ok(Stmt::If(Box::new(expr), Box::new(true_body), Box::new(final_else)))
            }
            TokenKind::keyword_break => {
                self.i += 1;
                self.end_line()?;
                Ok(Stmt::Break)
            }
            TokenKind::keyword_return => {
                self.i += 1;
                self.end_line()?;
                Ok(Stmt::Return)
            }
            TokenKind::keyword_mixin => {
                self.i += 1;
                let ident = self.consume_any_ident()?;
                let name = match ident {
                    Expression::Ident(s) => s,
                    _ => unreachable!(),
                };
                self.end_line()?;
                Ok(Stmt::Mixin(name))
            }
            _ => self.parse_command(),
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Stmt>, ParserError> {
        let mut result = Vec::new();
        while self.i < self.tokens.len() {
            result.push(self.parse_stmt()?);
        }
        Ok(result)
    }
}
