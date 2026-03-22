//! DeimosLang Types — Shared types for DeimosLang.
//!
//! Faithfully ported from `deimos-reference/src/deimoslang/types.py`.
#![allow(dead_code, non_camel_case_types)]

use std::fmt;
use crate::deimoslang::tokenizer::{Token, TokenKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommandKind {
    invalid,
    expr,
    expr_gt,
    expr_eq,
    kill,
    sleep,
    log,
    teleport,
    goto,
    sendkey,
    waitfor,
    usepotion,
    buypotions,
    relog,
    click,
    tozone,
    load_playstyle,
    set_yaw,
    setdeck,
    getdeck,
    select_friend,
    autopet,
    compound,
    set_goal,
    set_quest,
    set_zone,
    toggle_combat,
    restart_bot,
    cursor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TeleportKind {
    position,
    friend_icon,
    friend_name,
    entity_vague,
    entity_literal,
    mob,
    quest,
    client_num,
    nav,
    plusteleport,
    minusteleport,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EvalKind {
    health,
    max_health,
    mana,
    max_mana,
    energy,
    max_energy,
    bagcount,
    max_bagcount,
    gold,
    max_gold,
    windowtext,
    potioncount,
    max_potioncount,
    playercount,
    any_player_list,
    windownum,
    account_level,
    duel_round,
    reference_counter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WaitforKind {
    dialog,
    battle,
    zonechange,
    free,
    window,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CursorKind {
    position,
    window,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ClickKind {
    window,
    position,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LogKind {
    multi,
    single,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExprKind {
    window_visible,
    in_zone,
    same_zone,
    playercount,
    tracking_quest,
    tracking_goal,
    loading,
    in_combat,
    has_dialogue,
    has_xyz,
    has_quest,
    health_below,
    health_above,
    health,
    mana,
    mana_above,
    mana_below,
    energy,
    energy_above,
    energy_below,
    bag_count,
    bag_count_above,
    bag_count_below,
    gold,
    gold_above,
    gold_below,
    window_disabled,
    same_place,
    in_range,
    has_yaw,
    same_quest,
    same_xyz,
    same_yaw,
    items_dropped,
    duel_round,
    goal_changed,
    quest_changed,
    zone_changed,
    constant_check,
    constant_reference,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TimerAction {
    start,
    end,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct PlayerSelector {
    pub player_nums: Vec<i32>,
    pub mass: bool,
    pub inverted: bool,
    pub wildcard: bool,
    pub any_player: bool,
    pub same_any: bool,
}

impl PlayerSelector {
    pub fn validate(&mut self) {
        if self.mass && self.inverted { panic!("Invalid player selector: mass + except"); }
        if self.mass && !self.player_nums.is_empty() { panic!("Invalid player selector: mass + specified players"); }
        if self.inverted && self.player_nums.is_empty() { panic!("Invalid player selector: inverted + 0 players"); }
        if self.wildcard && (self.mass || !self.player_nums.is_empty()) { panic!("Invalid player selector: wildcard + mass or player_nums"); }
        if self.any_player && (self.mass || !self.player_nums.is_empty()) { panic!("Invalid player selector: any_player + mass or player_nums"); }
        if self.same_any && (self.mass || !self.player_nums.is_empty()) { panic!("Invalid player selector: same_any + mass or player_nums"); }
        self.player_nums.sort();
    }
}

#[derive(Debug, Clone)]
pub struct Command {
    pub kind: CommandKind,
    pub data: Vec<Box<Expression>>,
    pub player_selector: Option<PlayerSelector>,
}

#[derive(Debug, Clone)]
pub enum Expression {
    Constant(String, Box<Expression>),
    List(Vec<Box<Expression>>),
    Number(f64),
    String(String),
    StrFormat(String, Vec<Box<Expression>>),
    Unary(Token, Box<Expression>),
    Key(String),
    CommandExpr(Command),
    XYZ(Box<Expression>, Box<Expression>, Box<Expression>),
    Sub(Box<Expression>, Box<Expression>),
    Divide(Box<Expression>, Box<Expression>),
    Equivalent(Box<Expression>, Box<Expression>),
    ContainsString(Box<Expression>, Box<Expression>),
    Greater(Box<Expression>, Box<Expression>),
    And(Vec<Box<Expression>>),
    Or(Vec<Box<Expression>>),
    ConstantReference(String),
    ConstantCheck(String, Box<Expression>),
    RangeMin(Box<Expression>),
    RangeMax(Box<Expression>),
    IndexAccess(Box<Expression>, Box<Expression>),
    SelectorGroup(PlayerSelector, Box<Expression>),
    Ident(String),
    Sym(Symbol),
    StackLoc(usize),
    ReadVar(Box<Expression>),
    Eval(EvalKind, Vec<Box<Expression>>),
    ExprKind(ExprKind),
    TeleportKind(TeleportKind),
    WaitforKind(WaitforKind),
    CursorKind(CursorKind),
    ClickKind(ClickKind),
    LogKind(LogKind),
    Boolean(bool),
}

#[derive(Debug, Clone)]
pub enum Stmt {
    ConstantDecl(String, Box<Expression>),
    ParallelCommand(Vec<Command>),
    StmtList(Vec<Stmt>),
    Timer(TimerAction, String),
    Command(Command),
    If(Box<Expression>, Box<Stmt>, Box<Stmt>),
    Break,
    Return,
    Mixin(String),
    Loop(Box<Stmt>),
    While(Box<Expression>, Box<Stmt>),
    Until(Box<Expression>, Box<Stmt>),
    Times(i32, Box<Stmt>),
    BlockDef(Box<Expression>, Box<Stmt>, Vec<String>),
    Call(Box<Expression>),
    DefVar(Symbol),
    WriteVar(Symbol, Box<Expression>),
    KillVar(Symbol),
    UntilRegion(Box<Expression>, Box<Stmt>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SymbolKind {
    variable,
    block,
    label,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Symbol {
    pub literal: String,
    pub id: usize,
    pub kind: SymbolKind,
}
