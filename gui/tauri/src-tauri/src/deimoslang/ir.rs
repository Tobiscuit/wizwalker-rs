//! DeimosLang IR — Intermediate representation and compiler.
//!
//! Faithfully ported from `deimos-reference/src/deimoslang/ir.py`.
#![allow(dead_code, non_camel_case_types, unused_mut, unused_variables)]

use std::collections::HashMap;
use crate::deimoslang::types::*;
use crate::deimoslang::tokenizer::Tokenizer;
use crate::deimoslang::parser::Parser;
use crate::deimoslang::sem::Analyzer;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InstructionKind {
    kill,
    sleep,
    restart_bot,
    log_single,
    log_multi,
    jump,
    jump_if,
    jump_ifn,
    enter_until,
    exit_until,
    label,
    ret,
    call,
    deimos_call,
    load_playstyle,
    set_yaw,
    setdeck,
    getdeck,
    push_stack,
    pop_stack,
    write_stack,
    set_timer,
    end_timer,
    declare_constant,
    compound_deimos_call,
    nop,
}

#[derive(Debug, Clone)]
pub enum InstructionData {
    Int(i32),
    Float(f64),
    String(String),
    Symbol(Symbol),
    Expression(Box<Expression>),
    PlayerSelector(PlayerSelector),
    List(Vec<InstructionData>),
}

#[derive(Debug, Clone)]
pub struct Instruction {
    pub kind: InstructionKind,
    pub data: Option<InstructionData>,
}

impl std::fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.data {
            Some(data) => write!(f, "{:?} {:?}", self.kind, data),
            None => write!(f, "{:?}", self.kind),
        }
    }
}

pub struct StackInfo {
    pub offset: i32,
    pub slots: HashMap<Symbol, i32>,
}

impl StackInfo {
    pub fn new() -> Self {
        Self {
            offset: 0,
            slots: HashMap::new(),
        }
    }

    pub fn push(&mut self, sym: Symbol) {
        self.slots.insert(sym, self.offset);
        self.offset += 1;
    }

    pub fn pop(&mut self, sym: Symbol) -> Result<(), String> {
        self.offset -= 1;
        if let Some(&val) = self.slots.get(&sym) {
            if val != self.offset {
                return Err(format!(
                    "Attempted to pop a stack value that is not placed at the top: {:?}\n{:?}",
                    sym, self.slots
                ));
            }
        }
        self.slots.remove(&sym);
        Ok(())
    }

    pub fn loc(&self, sym: &Symbol) -> i32 {
        self.slots[sym] - self.offset
    }
}

pub struct Compiler<'a> {
    pub analyzer: &'a mut Analyzer,
    pub program: Vec<Instruction>,
    pub stacks: Vec<StackInfo>,
    pub loop_label_stack: Vec<Symbol>,
    pub outermost_until: Option<usize>,
}

impl<'a> Compiler<'a> {
    pub fn new(analyzer: &'a mut Analyzer) -> Self {
        Self {
            analyzer,
            program: Vec::new(),
            stacks: vec![StackInfo::new()],
            loop_label_stack: Vec::new(),
            outermost_until: None,
        }
    }

    pub fn from_text(code: &str) -> Result<Vec<Instruction>, String> {
        let mut tokenizer = Tokenizer::new();
        let tokens = tokenizer.tokenize(code, None).map_err(|e| e.to_string())?;
        let mut parser = Parser::new(tokens);
        let stmts = parser.parse().map_err(|e| e.to_string())?;
        let mut analyzer = Analyzer::new(stmts);
        analyzer.analyze_program();
        let mut compiler = Compiler::new(&mut analyzer);
        compiler.compile()
    }

    pub fn enter_branch(&mut self) {
        let top = self.stacks.last().unwrap();
        let mut new_top = StackInfo::new();
        new_top.offset = top.offset;
        new_top.slots = top.slots.clone();
        self.stacks.push(new_top);
    }

    pub fn exit_branch(&mut self) {
        self.stacks.pop();
    }

    pub fn stack_loc(&self, sym: &Symbol) -> Result<i32, String> {
        for info in self.stacks.iter().rev() {
            if info.slots.contains_key(sym) {
                return Ok(info.loc(sym));
            }
        }
        Err(format!("Failed to determine the stack location for symbol {:?}", sym))
    }

    pub fn emit(&mut self, kind: InstructionKind, data: Option<InstructionData>) {
        self.program.push(Instruction { kind, data });
    }

    pub fn gen_label(&mut self, name: &str) -> Symbol {
        self.analyzer.gen_label_sym(name)
    }

    pub fn emit_deimos_call(&mut self, com: &Command) {
        let data = InstructionData::List(vec![
            InstructionData::PlayerSelector(com.player_selector.clone().unwrap_or_default()),
            InstructionData::String(format!("{:?}", com.kind)),
            InstructionData::List(com.data.iter().map(|e| InstructionData::Expression(e.clone())).collect()),
        ]);
        self.emit(InstructionKind::deimos_call, Some(data));
    }

    pub fn compile_command(&mut self, com: &Command) -> Result<(), String> {
        match com.kind {
            CommandKind::restart_bot => {
                self.emit(InstructionKind::restart_bot, None);
            }
            CommandKind::toggle_combat | CommandKind::set_zone | CommandKind::set_goal | CommandKind::set_quest | CommandKind::autopet => {
                self.emit_deimos_call(com);
            }
            CommandKind::compound => {
                let mut command_entries = Vec::new();
                for sub_expr in &com.data {
                    if let Expression::CommandExpr(sub_command) = &**sub_expr {
                        let entry = InstructionData::List(vec![
                            InstructionData::PlayerSelector(sub_command.player_selector.clone().unwrap_or_default()),
                            InstructionData::String(format!("{:?}", sub_command.kind)),
                            InstructionData::List(sub_command.data.iter().map(|e| InstructionData::Expression(e.clone())).collect()),
                        ]);
                        command_entries.push(entry);
                    }
                }
                self.emit(InstructionKind::compound_deimos_call, Some(InstructionData::List(command_entries)));
            }
            CommandKind::kill => {
                self.emit(InstructionKind::kill, None);
            }
            CommandKind::sleep => {
                self.emit(InstructionKind::sleep, Some(InstructionData::Expression(com.data[0].clone())));
            }
            CommandKind::log => {
                let log_kind_str = match &*com.data[0] {
                    Expression::String(s) => s.clone(),
                    Expression::Ident(s) => s.clone(),
                    _ => "single".to_string(), // Fallback
                };

                if log_kind_str == "multi" {
                    self.emit(InstructionKind::log_multi, Some(InstructionData::List(vec![
                        InstructionData::PlayerSelector(com.player_selector.clone().unwrap_or_default()),
                        InstructionData::Expression(com.data[1].clone()),
                    ])));
                } else {
                    self.emit(InstructionKind::log_single, Some(InstructionData::Expression(com.data[1].clone())));
                }
            }
            CommandKind::sendkey | CommandKind::click | CommandKind::teleport | CommandKind::goto
            | CommandKind::usepotion | CommandKind::buypotions | CommandKind::relog | CommandKind::tozone | CommandKind::cursor
            | CommandKind::select_friend => {
                self.emit_deimos_call(com);
            }
            CommandKind::waitfor => {
                // copy the original data to split inverted waitfor in two
                let mut non_inverted_com = com.clone();
                if let Some(last) = non_inverted_com.data.last_mut() {
                    *last = Box::new(Expression::Number(0.0));
                }
                self.emit_deimos_call(&non_inverted_com);

                let is_inverted = match com.data.last().map(|b| &**b) {
                    Some(Expression::Number(n)) => *n != 0.0,
                    _ => false,
                };

                if is_inverted {
                    self.emit_deimos_call(com);
                }
            }
            CommandKind::set_yaw => {
                self.emit(InstructionKind::set_yaw, Some(InstructionData::List(vec![
                    InstructionData::PlayerSelector(com.player_selector.clone().unwrap_or_default()),
                    InstructionData::Expression(com.data[0].clone()),
                ])));
            }
            CommandKind::load_playstyle => {
                self.emit(InstructionKind::load_playstyle, Some(InstructionData::Expression(com.data[0].clone())));
            }
            CommandKind::setdeck => {
                self.emit(InstructionKind::setdeck, Some(InstructionData::List(vec![
                    InstructionData::PlayerSelector(com.player_selector.clone().unwrap_or_default()),
                    InstructionData::Expression(com.data[0].clone()),
                ])));
            }
            CommandKind::getdeck => {
                self.emit(InstructionKind::getdeck, Some(InstructionData::PlayerSelector(com.player_selector.clone().unwrap_or_default())));
            }
            _ => {
                return Err(format!("Unimplemented command: {:?}", com));
            }
        }
        Ok(())
    }

    pub fn process_labels(&mut self, program: Vec<Instruction>) -> Vec<Instruction> {
        let mut new_program: Vec<Instruction> = Vec::new();
        let mut offsets: HashMap<Symbol, i32> = HashMap::new();

        // discover labels
        for (idx, instr) in program.iter().enumerate() {
            match instr.kind {
                InstructionKind::label => {
                    if let Some(InstructionData::Symbol(ref sym)) = instr.data {
                        offsets.insert(sym.clone(), new_program.len() as i32);
                        if idx + 1 == program.len() {
                            // special case, jumping to the end may need padding
                            new_program.push(Instruction {
                                kind: InstructionKind::nop,
                                data: None,
                            });
                        }
                    }
                }
                _ => {
                    new_program.push(instr.clone());
                }
            }
        }

        let mut program = new_program;

        // resolve labels
        for idx in 0..program.len() {
            let instr = &mut program[idx];
            match instr.kind {
                InstructionKind::call | InstructionKind::jump => {
                    if let Some(InstructionData::Symbol(ref sym)) = instr.data {
                        if let Some(&offset) = offsets.get(sym) {
                            instr.data = Some(InstructionData::Int(offset - idx as i32));
                        }
                    }
                }
                InstructionKind::jump_if | InstructionKind::jump_ifn => {
                    if let Some(InstructionData::List(ref mut items)) = instr.data {
                        if items.len() >= 2 {
                            if let InstructionData::Symbol(ref sym) = items[1] {
                                if let Some(&offset) = offsets.get(sym) {
                                    items[1] = InstructionData::Int(offset - idx as i32);
                                }
                            }
                        }
                    }
                }
                InstructionKind::enter_until => {
                    if let Some(InstructionData::List(ref mut items)) = instr.data {
                        if items.len() >= 3 {
                            if let InstructionData::Symbol(ref sym) = items[2] {
                                if let Some(&offset) = offsets.get(sym) {
                                    items[2] = InstructionData::Int(offset - idx as i32);
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        program
    }

    pub fn compile_block_def(&mut self, block_def: &Stmt) -> Result<(), String> {
        if let Stmt::BlockDef(name, body, _) = block_def {
            if let Expression::Sym(ref sym) = **name {
                let enter_block_label = sym.clone();
                self.emit(InstructionKind::label, Some(InstructionData::Symbol(enter_block_label)));
                let prev_until = self.outermost_until;
                self.outermost_until = None;
                self.compile_stmt(body)?;
                self.outermost_until = prev_until;
            } else if let Expression::Ident(_) = **name {
                return Err(format!("Encountered an unresolved block sym during compilation: {:?}", block_def));
            } else {
                return Err(format!("Encountered a malformed block sym during compilation: {:?}", block_def));
            }
        }
        Ok(())
    }

    pub fn compile_call(&mut self, call: &Stmt) -> Result<(), String> {
        if let Stmt::Call(name) = call {
            if let Expression::Sym(ref sym) = **name {
                self.emit(InstructionKind::call, Some(InstructionData::Symbol(sym.clone())));
            } else if let Expression::Ident(_) = **name {
                return Err(format!("Encountered an unresolved call during compilation: {:?}", call));
            } else {
                return Err(format!("Encountered a malformed call during compilation: {:?}", call));
            }
        }
        Ok(())
    }

    pub fn prep_expression(&mut self, expr: &mut Expression) -> Result<(), String> {
        match expr {
            Expression::ConstantCheck(_, value) => {
                self.prep_expression(value)?;
            }
            Expression::And(expressions) | Expression::Or(expressions) => {
                for sub_expr in expressions {
                    self.prep_expression(sub_expr)?;
                }
            }
            Expression::Sub(lhs, rhs) | Expression::Divide(lhs, rhs) | Expression::Equivalent(lhs, rhs)
            | Expression::ContainsString(lhs, rhs) | Expression::Greater(lhs, rhs) => {
                self.prep_expression(lhs)?;
                self.prep_expression(rhs)?;
            }
            Expression::ReadVar(loc) => {
                if let Expression::Sym(ref sym) = **loc {
                    *loc = Box::new(Expression::StackLoc(self.stack_loc(sym)? as usize));
                } else if let Expression::StackLoc(_) = **loc {
                    // Already processed
                } else {
                    return Err(format!("Malformed ReadVarExpr: {:?}", expr));
                }
            }
            Expression::SelectorGroup(_, sub_expr) | Expression::Unary(_, sub_expr) => {
                self.prep_expression(sub_expr)?;
            }
            Expression::List(items) => {
                for item in items {
                    self.prep_expression(item)?;
                }
            }
            Expression::RangeMin(range_expr) | Expression::RangeMax(range_expr) => {
                self.prep_expression(range_expr)?;
            }
            Expression::IndexAccess(sub_expr, index) => {
                self.prep_expression(sub_expr)?;
                self.prep_expression(index)?;
            }
            Expression::Constant(_, value) => {
                self.prep_expression(value)?;
            }
            Expression::StrFormat(_, values) => {
                for val in values {
                    self.prep_expression(val)?;
                }
            }
            Expression::Eval(_, args) => {
                for arg in args {
                    self.prep_expression(arg)?;
                }
            }
            Expression::XYZ(x, y, z) => {
                self.prep_expression(x)?;
                self.prep_expression(y)?;
                self.prep_expression(z)?;
            }
            Expression::Number(_) | Expression::String(_) | Expression::Key(_)
            | Expression::CommandExpr(_) | Expression::Ident(_) | Expression::Sym(_)
            | Expression::StackLoc(_) | Expression::ConstantReference(_)
            | Expression::Boolean(_) | Expression::ExprKind(_) | Expression::TeleportKind(_)
            | Expression::WaitforKind(_) | Expression::CursorKind(_) | Expression::ClickKind(_)
            | Expression::LogKind(_) => {
                // Nothing to do
            }
        }
        Ok(())
    }

    pub fn compile_if_stmt(&mut self, expr: &mut Expression, branch_true: &Stmt, branch_false: &Stmt) -> Result<(), String> {
        let after_if_label = self.gen_label("after_if");
        let branch_true_label = self.gen_label("branch_true");
        self.prep_expression(expr)?;
        self.emit(InstructionKind::jump_if, Some(InstructionData::List(vec![
            InstructionData::Expression(Box::new(expr.clone())),
            InstructionData::Symbol(branch_true_label.clone()),
        ])));
        self.enter_branch();
        self.compile_stmt(branch_false)?;
        self.exit_branch();
        self.emit(InstructionKind::jump, Some(InstructionData::Symbol(after_if_label.clone())));
        self.emit(InstructionKind::label, Some(InstructionData::Symbol(branch_true_label)));
        self.enter_branch();
        self.compile_stmt(branch_true)?;
        self.exit_branch();
        self.emit(InstructionKind::label, Some(InstructionData::Symbol(after_if_label)));
        Ok(())
    }

    pub fn compile_loop_stmt(&mut self, body: &Stmt) -> Result<(), String> {
        let start_loop_label = self.gen_label("start_loop");
        let end_loop_label = self.gen_label("end_loop");
        self.loop_label_stack.push(end_loop_label.clone());
        self.emit(InstructionKind::label, Some(InstructionData::Symbol(start_loop_label.clone())));
        self.enter_branch();
        self.compile_stmt(body)?;
        self.exit_branch();
        self.emit(InstructionKind::jump, Some(InstructionData::Symbol(start_loop_label)));
        self.emit(InstructionKind::label, Some(InstructionData::Symbol(end_loop_label)));
        self.loop_label_stack.pop();
        Ok(())
    }

    pub fn compile_while_stmt(&mut self, expr: &mut Expression, body: &Stmt) -> Result<(), String> {
        let start_while_label = self.gen_label("start_while");
        let end_while_label = self.gen_label("end_while");
        self.loop_label_stack.push(end_while_label.clone());
        self.prep_expression(expr)?;
        self.emit(InstructionKind::jump_ifn, Some(InstructionData::List(vec![
            InstructionData::Expression(Box::new(expr.clone())),
            InstructionData::Symbol(end_while_label.clone()),
        ])));
        self.emit(InstructionKind::label, Some(InstructionData::Symbol(start_while_label.clone())));
        self.enter_branch();
        self.compile_stmt(body)?;
        self.exit_branch();
        self.emit(InstructionKind::jump_if, Some(InstructionData::List(vec![
            InstructionData::Expression(Box::new(expr.clone())),
            InstructionData::Symbol(start_while_label),
        ])));
        self.emit(InstructionKind::label, Some(InstructionData::Symbol(end_while_label)));
        self.loop_label_stack.pop();
        Ok(())
    }

    pub fn compile_until_region(&mut self, expr: &mut Expression, body: &Stmt) -> Result<(), String> {
        let id = self.analyzer.gen_sym_id();
        let exit_until_label = self.gen_label("exit_until");
        self.loop_label_stack.push(exit_until_label.clone());
        self.prep_expression(expr)?;
        self.emit(InstructionKind::enter_until, Some(InstructionData::List(vec![
            InstructionData::Expression(Box::new(expr.clone())),
            InstructionData::Int(id as i32),
            InstructionData::Symbol(exit_until_label.clone()),
        ])));

        let is_outermost = self.outermost_until.is_none();
        if is_outermost {
            self.outermost_until = Some(id);
        }
        self.enter_branch();
        self.compile_stmt(body)?;
        self.exit_branch();
        if is_outermost {
            self.outermost_until = None;
        }
        self.emit(InstructionKind::label, Some(InstructionData::Symbol(exit_until_label)));
        self.emit(InstructionKind::exit_until, Some(InstructionData::Int(id as i32)));
        self.loop_label_stack.pop();
        Ok(())
    }

    pub fn compile_return_stmt(&mut self) {
        if let Some(id) = self.outermost_until {
            self.emit(InstructionKind::exit_until, Some(InstructionData::Int(id as i32)));
        }
        self.emit(InstructionKind::ret, None);
    }

    pub fn compile_stmt(&mut self, stmt: &Stmt) -> Result<(), String> {
        let mut stmt_clone = stmt.clone();
        match &mut stmt_clone {
            Stmt::ConstantDecl(name, value) => {
                self.prep_expression(value)?;
                self.emit(InstructionKind::declare_constant, Some(InstructionData::List(vec![
                    InstructionData::String(name.clone()),
                    InstructionData::Expression(value.clone()),
                ])));
            }
            Stmt::ParallelCommand(commands) => {
                let mut command_entries = Vec::new();
                for command in commands {
                    let entry = InstructionData::List(vec![
                        InstructionData::PlayerSelector(command.player_selector.clone().unwrap_or_default()),
                        InstructionData::String(format!("{:?}", command.kind)),
                        InstructionData::List(command.data.iter().map(|e| InstructionData::Expression(e.clone())).collect()),
                    ]);
                    command_entries.push(entry);
                }
                self.emit(InstructionKind::compound_deimos_call, Some(InstructionData::List(command_entries)));
            }
            Stmt::Timer(action, timer_name) => {
                if *action == TimerAction::start {
                    self.emit(InstructionKind::set_timer, Some(InstructionData::String(timer_name.clone())));
                } else {
                    self.emit(InstructionKind::end_timer, Some(InstructionData::String(timer_name.clone())));
                }
            }
            Stmt::StmtList(stmts) => {
                for inner in stmts {
                    self.compile_stmt(inner)?;
                }
            }
            Stmt::Command(com) => {
                self.compile_command(com)?;
            }
            Stmt::Call(_) => {
                self.compile_call(stmt)?;
            }
            Stmt::BlockDef(_, _, _) => {
                self.compile_block_def(stmt)?;
            }
            Stmt::If(expr, true_branch, false_branch) => {
                self.compile_if_stmt(expr, true_branch, false_branch)?;
            }
            Stmt::Loop(body) => {
                self.compile_loop_stmt(body)?;
            }
            Stmt::While(expr, body) => {
                self.compile_while_stmt(expr, body)?;
            }
            Stmt::DefVar(sym) => {
                self.stacks.last_mut().unwrap().push(sym.clone());
                self.emit(InstructionKind::push_stack, None);
            }
            Stmt::KillVar(sym) => {
                self.emit(InstructionKind::pop_stack, None);
                self.stacks.last_mut().unwrap().pop(sym.clone())?;
            }
            Stmt::WriteVar(sym, expr) => {
                self.prep_expression(expr)?;
                self.emit(InstructionKind::write_stack, Some(InstructionData::List(vec![
                    InstructionData::Int(self.stack_loc(sym)?),
                    InstructionData::Expression(expr.clone()),
                ])));
            }
            Stmt::Break => {
                let label = self.loop_label_stack.last().ok_or("Break outside of loop")?.clone();
                self.emit(InstructionKind::jump, Some(InstructionData::Symbol(label)));
            }
            Stmt::Return => {
                self.compile_return_stmt();
            }
            Stmt::UntilRegion(expr, body) => {
                self.compile_until_region(expr, body)?;
            }
            _ => {
                return Err(format!("Unknown statement: {:?}", stmt));
            }
        }
        Ok(())
    }

    pub fn compile(&mut self) -> Result<Vec<Instruction>, String> {
        let toplevel_start_label = self.gen_label("program_start");
        self.emit(InstructionKind::jump, Some(InstructionData::Symbol(toplevel_start_label.clone())));

        let block_defs = self.analyzer.block_defs.clone();
        for stmt in &block_defs {
            self.compile_stmt(stmt)?;
        }

        self.emit(InstructionKind::label, Some(InstructionData::Symbol(toplevel_start_label)));

        let stmts = self.analyzer.stmts.clone();
        for stmt in &stmts {
            self.compile_stmt(stmt)?;
        }

        let program = std::mem::take(&mut self.program);
        Ok(self.process_labels(program))
    }
}
