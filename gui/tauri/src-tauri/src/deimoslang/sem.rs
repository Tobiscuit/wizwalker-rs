//! DeimosLang Semantic Analyzer — Scope management and validation.
//!
//! Faithfully ported from `deimos-reference/src/deimoslang/sem.py`.
#![allow(dead_code, non_camel_case_types, unused_mut, unused_variables)]

use std::collections::{HashMap, HashSet};
use crate::deimoslang::types::*;
use crate::deimoslang::tokenizer::{Token, TokenKind, TokenValue, LineInfo};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SemError {
    #[error("Semantic error: {0}")]
    Generic(String),
}

type Result<T> = std::result::Result<T, SemError>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct MixinKey {
    source_sym_id: usize,
    mixed_sym_ids: Vec<usize>,
}

pub struct Scope {
    pub parent: Option<Box<Scope>>,
    pub syms: Vec<Symbol>,
    pub mixins: HashSet<String>,
    pub unique_player_selectors: HashSet<PlayerSelector>,
    pub active_vars: Vec<Symbol>,
    pub cleaned_vars: HashSet<Symbol>,
    pub is_block: bool,
}

impl Scope {
    pub fn new(parent: Option<Box<Scope>>, is_block: bool) -> Self {
        Self {
            parent,
            syms: Vec::new(),
            mixins: HashSet::new(),
            unique_player_selectors: HashSet::new(),
            active_vars: Vec::new(),
            cleaned_vars: HashSet::new(),
            is_block,
        }
    }

    pub fn lookup_block_by_name(&self, literal: &str) -> Option<Symbol> {
        for sym in self.syms.iter().rev() {
            if sym.kind != SymbolKind::block {
                continue;
            }
            if sym.literal == literal {
                return Some(sym.clone());
            }
        }
        if let Some(ref parent) = self.parent {
            return parent.lookup_block_by_name(literal);
        }
        None
    }

    pub fn lookup_var_by_name(&self, literal: &str) -> Option<Symbol> {
        let transformed = format!(":{}:", literal);
        for sym in self.active_vars.iter().rev() {
            if sym.literal == literal || sym.literal == transformed {
                return Some(sym.clone());
            }
        }
        if let Some(ref parent) = self.parent {
            return parent.lookup_var_by_name(literal);
        }
        None
    }

    pub fn is_mixin(&self, literal: &str) -> bool {
        if self.mixins.contains(literal) {
            return true;
        }
        if let Some(ref parent) = self.parent {
            return parent.is_mixin(literal);
        }
        false
    }

    pub fn is_block_local_var(&self, sym: &Symbol) -> bool {
        let mut cur = self;
        loop {
            if cur.syms.contains(sym) {
                return true;
            } else if cur.is_block {
                break;
            }
            if let Some(ref parent) = cur.parent {
                cur = parent;
            } else {
                break;
            }
        }
        false
    }

    pub fn put_sym(&mut self, sym: Symbol) -> Symbol {
        self.syms.push(sym.clone());
        sym
    }

    pub fn activate_var(&mut self, sym: Symbol) -> Result<()> {
        if self.active_vars.contains(&sym) {
            return Err(SemError::Generic(format!("Attempted to activate an already active variable: {:?}", sym)));
        }
        self.active_vars.push(sym);
        Ok(())
    }

    pub fn kill_var(&mut self, sym: Symbol) -> Result<()> {
        if !self.is_block_local_var(&sym) {
            return Err(SemError::Generic("Attempted to kill a variable that isn't local to the current block".to_string()));
        }
        if !self.active_vars.contains(&sym) {
            return Err(SemError::Generic(format!("Attempted to kill an inactive variable: {:?}", sym)));
        }
        self.cleaned_vars.insert(sym.clone());
        self.active_vars.retain(|v| v != &sym);
        Ok(())
    }
}

pub struct Analyzer {
    pub scope: Scope,
    pub next_sym_id: usize,
    pub block_defs: Vec<Stmt>,
    pub stmts: Vec<Stmt>,
    pub mixin_cache: HashMap<MixinKey, Symbol>,
    pub defnode_map: HashMap<usize, Stmt>,
    pub constants: HashMap<String, Expression>,
    pub block_nesting_level: i32,
    pub loop_nesting_level: i32,
    pub loop_nesting_stack: Vec<i32>,
}

impl Analyzer {
    pub fn new(stmts: Vec<Stmt>) -> Self {
        Self {
            scope: Scope::new(None, false),
            next_sym_id: 0,
            block_defs: Vec::new(),
            stmts,
            mixin_cache: HashMap::new(),
            defnode_map: HashMap::new(),
            constants: HashMap::new(),
            block_nesting_level: 0,
            loop_nesting_level: 0,
            loop_nesting_stack: Vec::new(),
        }
    }

    pub fn open_block(&mut self) {
        let parent = std::mem::replace(&mut self.scope, Scope::new(None, true));
        self.scope = Scope::new(Some(Box::new(parent)), true);
        self.loop_nesting_stack.push(self.loop_nesting_level);
        self.loop_nesting_level = 0;
        self.block_nesting_level += 1;
    }

    pub fn close_block(&mut self) {
        if let Some(parent) = self.scope.parent.take() {
            self.scope = *parent;
        }
        self.loop_nesting_level = self.loop_nesting_stack.pop().unwrap_or(0);
        self.block_nesting_level -= 1;
    }

    pub fn open_loop(&mut self) {
        let parent = std::mem::replace(&mut self.scope, Scope::new(None, false));
        let active_vars = parent.active_vars.clone();
        let mut new_scope = Scope::new(Some(Box::new(parent)), false);
        new_scope.active_vars = active_vars;
        self.scope = new_scope;
        self.loop_nesting_level += 1;
    }

    pub fn close_loop(&mut self) -> Result<()> {
        if !self.scope.mixins.is_empty() {
            return Err(SemError::Generic("Mixins are only allowed at the top level of a block".to_string()));
        }
        if let Some(parent) = self.scope.parent.take() {
            self.scope = *parent;
        }
        self.loop_nesting_level -= 1;
        Ok(())
    }

    pub fn with_branch<F, T>(&mut self, f: F) -> Result<T> 
    where F: FnOnce(&mut Analyzer) -> Result<T> {
        let parent = std::mem::replace(&mut self.scope, Scope::new(None, false));
        let active_vars = parent.active_vars.clone();
        let mut new_scope = Scope::new(Some(Box::new(parent)), false);
        new_scope.active_vars = active_vars;
        self.scope = new_scope;

        let result = f(self);

        if let Some(parent) = self.scope.parent.take() {
            self.scope = *parent;
        }
        result
    }

    pub fn gen_sym_id(&mut self) -> usize {
        let result = self.next_sym_id;
        self.next_sym_id += 1;
        result
    }

    pub fn gen_block_sym(&mut self, name: &str) -> Symbol {
        let id = self.gen_sym_id();
        self.scope.put_sym(Symbol { literal: name.to_string(), id, kind: SymbolKind::block })
    }

    pub fn gen_var_sym(&mut self, name: &str) -> Symbol {
        let id = self.gen_sym_id();
        self.scope.put_sym(Symbol { literal: format!(":{}:", name), id, kind: SymbolKind::variable })
    }

    pub fn gen_label_sym(&mut self, name: &str) -> Symbol {
        let id = self.gen_sym_id();
        self.scope.put_sym(Symbol { literal: format!(":{}", name), id, kind: SymbolKind::label })
    }

    pub fn def_var(&mut self) -> Result<Symbol> {
        let var_sym = self.gen_var_sym("anonymous");
        self.scope.activate_var(var_sym.clone())?;
        Ok(var_sym)
    }

    pub fn mark_var_dead(&mut self, sym: Symbol) -> Result<()> {
        self.scope.kill_var(sym)
    }

    pub fn gen_cleanup_all_vars(&mut self) -> Result<Stmt> {
        let mut res = Vec::new();
        let vars: Vec<Symbol> = self.scope.active_vars.iter().cloned().rev().collect();
        for var in vars {
            self.mark_var_dead(var.clone())?;
            res.push(Stmt::KillVar(var));
        }
        Ok(Stmt::StmtList(res))
    }

    pub fn sem_command(&mut self, mut cmd: Command) -> Command {
        cmd.data = cmd.data.into_iter().map(|e| Box::new(self.sem_expr(*e))).collect();
        if let Some(ref sel) = cmd.player_selector {
            self.scope.unique_player_selectors.insert(sel.clone());
        }
        cmd
    }

    pub fn sem_expr(&mut self, expr: Expression) -> Expression {
        match expr {
            Expression::Ident(ref ident) => {
                if let Some(sym) = self.scope.lookup_var_by_name(ident) {
                    Expression::ReadVar(Box::new(Expression::Sym(sym)))
                } else if self.lookup_constant(ident).is_some() {
                    Expression::ConstantReference(ident.clone())
                } else {
                    expr
                }
            }
            Expression::Unary(op, inner) => Expression::Unary(op, Box::new(self.sem_expr(*inner))),
            Expression::Sub(lhs, rhs) => Expression::Sub(Box::new(self.sem_expr(*lhs)), Box::new(self.sem_expr(*rhs))),
            Expression::Divide(lhs, rhs) => Expression::Divide(Box::new(self.sem_expr(*lhs)), Box::new(self.sem_expr(*rhs))),
            Expression::Equivalent(lhs, rhs) => Expression::Equivalent(Box::new(self.sem_expr(*lhs)), Box::new(self.sem_expr(*rhs))),
            Expression::ContainsString(lhs, rhs) => Expression::ContainsString(Box::new(self.sem_expr(*lhs)), Box::new(self.sem_expr(*rhs))),
            Expression::Greater(lhs, rhs) => Expression::Greater(Box::new(self.sem_expr(*lhs)), Box::new(self.sem_expr(*rhs))),
            Expression::And(exprs) => Expression::And(exprs.into_iter().map(|e| Box::new(self.sem_expr(*e))).collect()),
            Expression::Or(exprs) => Expression::Or(exprs.into_iter().map(|e| Box::new(self.sem_expr(*e))).collect()),
            Expression::List(items) => Expression::List(items.into_iter().map(|e| Box::new(self.sem_expr(*e))).collect()),
            Expression::XYZ(x, y, z) => Expression::XYZ(Box::new(self.sem_expr(*x)), Box::new(self.sem_expr(*y)), Box::new(self.sem_expr(*z))),
            Expression::ReadVar(loc) => Expression::ReadVar(Box::new(self.sem_expr(*loc))),
            Expression::Eval(kind, args) => Expression::Eval(kind, args.into_iter().map(|e| Box::new(self.sem_expr(*e))).collect()),
            Expression::IndexAccess(expr, idx) => Expression::IndexAccess(Box::new(self.sem_expr(*expr)), Box::new(self.sem_expr(*idx))),
            Expression::SelectorGroup(sel, inner) => Expression::SelectorGroup(sel, Box::new(self.sem_expr(*inner))),
            Expression::StrFormat(fmt, args) => Expression::StrFormat(fmt, args.into_iter().map(|e| Box::new(self.sem_expr(*e))).collect()),
            Expression::Constant(name, val) => Expression::Constant(name, Box::new(self.sem_expr(*val))),
            Expression::ConstantCheck(name, val) => Expression::ConstantCheck(name, Box::new(self.sem_expr(*val))),
            Expression::RangeMin(inner) => Expression::RangeMin(Box::new(self.sem_expr(*inner))),
            Expression::RangeMax(inner) => Expression::RangeMax(Box::new(self.sem_expr(*inner))),
            Expression::CommandExpr(cmd) => Expression::CommandExpr(self.sem_command(cmd)),
            _ => expr,
        }
    }

    fn _mix_stmt(&self, stmt: &mut Stmt, mixins: &HashSet<String>) -> Result<()> {
        match stmt {
            Stmt::StmtList(stmts) => {
                for inner in stmts {
                    self._mix_stmt(inner, mixins)?;
                }
            }
            Stmt::Call(name_expr) => {
                match &mut **name_expr {
                    Expression::Ident(ident) => {
                        if mixins.contains(ident) {
                            let sym = self.scope.lookup_block_by_name(ident)
                                .ok_or_else(|| SemError::Generic(format!("Unable to find symbol in scope: {}", ident)))?;
                            
                            if !self.defnode_map.contains_key(&sym.id) {
                                *name_expr = Box::new(Expression::Sym(sym));
                            } else {
                                let defnode = self.defnode_map.get(&sym.id).unwrap();
                                if let Stmt::BlockDef(_, _, def_mixins) = defnode {
                                    if !def_mixins.is_empty() {
                                        return Err(SemError::Generic("Recursive mixins aren't allowed".to_string()));
                                    }
                                }
                                *name_expr = Box::new(Expression::Sym(sym));
                            }
                        } else {
                            return Err(SemError::Generic(format!("Undeclared identifier during mixin stage: {}", ident)));
                        }
                    }
                    Expression::Sym(sym) => {
                        let defnode = self.defnode_map.get(&sym.id)
                            .ok_or_else(|| SemError::Generic("Sym has no defnode".to_string()))?;
                        if let Stmt::BlockDef(_, _, def_mixins) = defnode {
                            if !def_mixins.is_empty() {
                                return Err(SemError::Generic("Recursive mixins aren't allowed".to_string()));
                            }
                        }
                    }
                    _ => return Err(SemError::Generic(format!("Invalid call target: {:?}", name_expr))),
                }
            }
            Stmt::While(_, body) => self._mix_stmt(body, mixins)?,
            Stmt::If(_, true_branch, false_branch) => {
                self._mix_stmt(true_branch, mixins)?;
                self._mix_stmt(false_branch, mixins)?;
            }
            Stmt::Loop(body) => self._mix_stmt(body, mixins)?,
            Stmt::UntilRegion(_, body) => self._mix_stmt(body, mixins)?,
            _ => {}
        }
        Ok(())
    }

    pub fn mix_block(&mut self, mut stmt: Stmt, source_sym: Symbol) -> Result<Stmt> {
        let (mut body, mixins, name_sym) = match stmt {
            Stmt::BlockDef(ref name_expr, ref body, ref mixins) => {
                let sym = match &**name_expr {
                    Expression::Sym(s) => s.clone(),
                    _ => return Err(SemError::Generic("Expected SymExpression in mix_block".to_string())),
                };
                (body.clone(), mixins.clone(), sym)
            }
            _ => return Err(SemError::Generic("Expected BlockDef in mix_block".to_string())),
        };

        let mut mixed_sym_ids: Vec<usize> = Vec::new();
        for m in &mixins {
            let ms = self.scope.lookup_block_by_name(m)
                .ok_or_else(|| SemError::Generic(format!("Unable to resolve mixin: {}", m)))?;
            mixed_sym_ids.push(ms.id);
        }
        mixed_sym_ids.sort();

        let key = MixinKey {
            source_sym_id: source_sym.id,
            mixed_sym_ids,
        };
        self.mixin_cache.insert(key, name_sym);

        let mixin_set: HashSet<String> = mixins.into_iter().collect();
        self._mix_stmt(&mut body, &mixin_set)?;

        if let Stmt::BlockDef(_, ref mut b, ref mut m) = stmt {
            *b = body;
            m.clear();
        }

        Ok(stmt)
    }

    pub fn lookup_constant(&self, name: &str) -> Option<Expression> {
        self.constants.get(name).cloned()
    }

    pub fn sem_stmt(&mut self, stmt: Stmt) -> Result<Option<Stmt>> {
        match stmt {
            Stmt::Timer(action, name) => Ok(Some(Stmt::Timer(action, name))),
            Stmt::ConstantDecl(name, mut value) => {
                let sem_value = self.sem_expr(*value);
                self.constants.insert(name.clone(), sem_value.clone());
                Ok(Some(Stmt::ConstantDecl(name, Box::new(sem_value))))
            }
            Stmt::BlockDef(name_expr, mut body, _) => {
                let ident = match &*name_expr {
                    Expression::Ident(s) => s.clone(),
                    _ => return Err(SemError::Generic("Only IdentExpression is allowed during block declaration".to_string())),
                };
                let sym = self.gen_block_sym(&ident);

                if let Stmt::StmtList(ref mut stmts) = *body {
                    stmts.push(Stmt::Return);
                }

                self.open_block();
                let sem_body = self.sem_stmt(*body)?.unwrap_or(Stmt::StmtList(vec![]));
                let scope_mixins: Vec<String> = self.scope.mixins.iter().cloned().collect();
                self.close_block();

                let sym_expr = Box::new(Expression::Sym(sym.clone()));
                let block_def = Stmt::BlockDef(sym_expr, Box::new(sem_body), scope_mixins);

                self.defnode_map.insert(sym.id, block_def.clone());

                if let Stmt::BlockDef(_, _, ms) = &block_def {
                    if ms.is_empty() {
                        self.block_defs.push(block_def.clone());
                    }
                }

                Ok(None)
            }
            Stmt::StmtList(stmts) => {
                let mut res = Vec::new();
                for inner in stmts {
                    if let Some(semmed) = self.sem_stmt(inner)? {
                        res.push(semmed);
                    }
                }
                Ok(Some(Stmt::StmtList(res)))
            }
            Stmt::Call(name_expr) => {
                let ident = match &*name_expr {
                    Expression::Ident(s) => Some(s.clone()),
                    _ => None,
                };

                let sym_res = if let Some(ref id) = ident {
                    if self.scope.is_mixin(id) {
                        return Ok(Some(Stmt::Call(name_expr)));
                    }
                    self.scope.lookup_block_by_name(id)
                } else if let Expression::Sym(ref s) = *name_expr {
                    Some(s.clone())
                } else {
                    return Err(SemError::Generic(format!("Malformed call: {:?}", name_expr)));
                };

                let mut sym = sym_res.ok_or_else(|| SemError::Generic(format!("Unable to find symbol in scope: {:?}", ident)))?;

                let defnode_opt = self.defnode_map.get(&sym.id).cloned();
                if let Some(defnode) = defnode_opt {
                    if let Stmt::BlockDef(_, _, ms) = &defnode {
                        if !ms.is_empty() {
                            let mut mixed_sym_ids = Vec::new();
                            for m in ms {
                                let ms_sym = self.scope.lookup_block_by_name(m)
                                    .ok_or_else(|| SemError::Generic(format!("Unable to resolve mixin: {}", m)))?;
                                mixed_sym_ids.push(ms_sym.id);
                            }
                            mixed_sym_ids.sort();

                            let key = MixinKey {
                                source_sym_id: sym.id,
                                mixed_sym_ids,
                            };

                            if let Some(cached_sym) = self.mixin_cache.get(&key) {
                                sym = cached_sym.clone();
                            } else {
                                let new_name = format!(":mixed_{}", sym.literal);
                                let mixed_sym = self.gen_block_sym(&new_name);

                                let mut new_defnode = defnode.clone();
                                if let Stmt::BlockDef(ref mut n, _, _) = new_defnode {
                                    *n = Box::new(Expression::Sym(mixed_sym.clone()));
                                }

                                let mixed_defnode = self.mix_block(new_defnode, sym.clone())?;
                                sym = mixed_sym;
                                self.defnode_map.insert(sym.id, mixed_defnode.clone());
                                self.block_defs.push(mixed_defnode);
                            }
                        }
                    }
                }
                Ok(Some(Stmt::Call(Box::new(Expression::Sym(sym)))))
            }
            Stmt::Command(cmd) => {
                let sem_cmd = self.sem_command(cmd);
                Ok(Some(Stmt::Command(sem_cmd)))
            }
            Stmt::ParallelCommand(cmds) => {
                let sem_cmds = cmds.into_iter().map(|c| self.sem_command(c)).collect();
                Ok(Some(Stmt::ParallelCommand(sem_cmds)))
            }
            Stmt::If(expr, true_branch, false_branch) => {
                let sem_expr = self.sem_expr(*expr);

                let sem_true = self.with_branch(|a| Ok(a.sem_stmt(*true_branch)?.unwrap_or(Stmt::StmtList(vec![]))))?;
                let sem_false = self.with_branch(|a| Ok(a.sem_stmt(*false_branch)?.unwrap_or(Stmt::StmtList(vec![]))))?;

                Ok(Some(Stmt::If(Box::new(sem_expr), Box::new(sem_true), Box::new(sem_false))))
            }
            Stmt::Loop(body) => {
                self.open_loop();
                let sem_body = self.sem_stmt(*body)?.unwrap_or(Stmt::StmtList(vec![]));
                self.close_loop()?;
                Ok(Some(Stmt::Loop(Box::new(sem_body))))
            }
            Stmt::While(expr, body) => {
                let sem_expr = self.sem_expr(*expr);
                self.open_loop();
                let sem_body = self.sem_stmt(*body)?.unwrap_or(Stmt::StmtList(vec![]));
                self.close_loop()?;
                Ok(Some(Stmt::While(Box::new(sem_expr), Box::new(sem_body))))
            }
            Stmt::Until(expr, body) => {
                self.open_loop();
                let sem_expr = self.sem_expr(*expr);
                let sem_body = self.sem_stmt(*body)?.unwrap_or(Stmt::StmtList(vec![]));
                self.close_loop()?;

                let not_token = Token {
                    kind: TokenKind::keyword_not,
                    literal: "not".to_string(),
                    line_info: LineInfo::new(usize::MAX, 0, 0, None, None),
                    value: TokenValue::None,
                };

                let while_expr = Expression::Unary(not_token, Box::new(sem_expr.clone()));
                let while_stmt = Stmt::While(Box::new(while_expr), Box::new(sem_body));

                Ok(Some(Stmt::If(
                    Box::new(sem_expr.clone()),
                    Box::new(Stmt::StmtList(vec![])),
                    Box::new(Stmt::StmtList(vec![
                        Stmt::UntilRegion(Box::new(sem_expr), Box::new(while_stmt))
                    ]))
                )))
            }
            Stmt::Times(num, body) => {
                let var_sym = self.def_var()?;
                let prologue = vec![
                    Stmt::DefVar(var_sym.clone()),
                    Stmt::WriteVar(var_sym.clone(), Box::new(Expression::Number(num as f64))),
                ];
                let epilogue = vec![
                    Stmt::KillVar(var_sym.clone()),
                ];

                let cond = Expression::Greater(
                    Box::new(Expression::ReadVar(Box::new(Expression::Sym(var_sym.clone())))),
                    Box::new(Expression::Number(0.0))
                );

                let decrement = Stmt::WriteVar(
                    var_sym.clone(),
                    Box::new(Expression::Sub(
                        Box::new(Expression::ReadVar(Box::new(Expression::Sym(var_sym.clone())))),
                        Box::new(Expression::Number(1.0))
                    ))
                );

                let wrapped_body = match *body {
                    Stmt::StmtList(mut stmts) => {
                        stmts.push(decrement);
                        Stmt::StmtList(stmts)
                    }
                    other => Stmt::StmtList(vec![other, decrement]),
                };

                let while_stmt = Stmt::While(Box::new(cond), Box::new(wrapped_body));
                let sem_while = self.sem_stmt(while_stmt)?.unwrap();

                let mut all_stmts = prologue;
                all_stmts.push(sem_while);
                all_stmts.extend(epilogue);

                self.mark_var_dead(var_sym)?;
                Ok(Some(Stmt::StmtList(all_stmts)))
            }
            Stmt::Return => {
                if self.block_nesting_level <= 0 {
                    return Err(SemError::Generic("Return used outside of block scope".to_string()));
                }
                let cleanup = self.gen_cleanup_all_vars()?;
                Ok(Some(Stmt::StmtList(vec![cleanup, Stmt::Return])))
            }
            Stmt::Break => {
                if self.loop_nesting_level <= 0 {
                    return Err(SemError::Generic("Break used outside of loop scope".to_string()));
                }
                Ok(Some(Stmt::Break))
            }
            Stmt::Mixin(name) => {
                if !self.scope.is_block {
                    return Err(SemError::Generic("Mixin is only allowed inside blocks".to_string()));
                }
                self.scope.mixins.insert(name);
                Ok(None)
            }
            Stmt::DefVar(sym) => Ok(Some(Stmt::DefVar(sym))),
            Stmt::WriteVar(sym, expr) => {
                let sem_expr = self.sem_expr(*expr);
                Ok(Some(Stmt::WriteVar(sym, Box::new(sem_expr))))
            }
            Stmt::KillVar(sym) => Ok(Some(Stmt::KillVar(sym))),
            Stmt::UntilRegion(expr, body) => Ok(Some(Stmt::UntilRegion(expr, body))),
        }
    }

    pub fn analyze_program(&mut self) -> Result<()> {
        // Collect all constants first as a preprocessing step
        // to allow forward references if needed, although sem.py seems sequential
        let stmts_clone = self.stmts.clone();
        for stmt in &stmts_clone {
            if let Stmt::ConstantDecl(name, value) = stmt {
                // Pre-populate if we want forward refs, but let's stick to faithful sequential for now
            }
        }

        let stmts = std::mem::take(&mut self.stmts);
        let mut res = Vec::new();
        for stmt in stmts {
            if let Some(semmed) = self.sem_stmt(stmt)? {
                res.push(semmed);
            }
        }
        self.stmts = res;
        Ok(())
    }
}
