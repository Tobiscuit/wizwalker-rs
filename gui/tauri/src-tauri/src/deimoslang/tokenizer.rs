//! DeimosLang tokenizer — Lexer for converting script text into tokens.
//!
//! Faithfully ported from `deimos-reference/src/deimoslang/tokenizer.py`.
#![allow(dead_code, non_camel_case_types)]

use std::fmt;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TokenizerError {
    #[error("{0}")]
    Error(String),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Percent(pub f64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenKind {
    player_num,
    player_wildcard,
    string,
    number,
    contains,
    percent,
    path, // A/B/C
    logical_and,
    logical_to,
    logical_from,
    logical_on,
    logical_off,
    boolean_true,
    boolean_false,

    greater,
    less,
    equals,

    keyword_block,
    keyword_call,
    keyword_loop,
    keyword_while,
    keyword_until,
    keyword_times,
    keyword_if,
    keyword_elif,
    keyword_else,
    keyword_except,
    keyword_mass,
    keyword_mob,
    keyword_quest,
    keyword_icon,
    keyword_ifneeded,
    keyword_completion,
    keyword_from,
    keyword_to,
    keyword_xyz,
    keyword_orient,
    keyword_not,
    keyword_return,
    keyword_break,
    keyword_mixin,
    keyword_and,
    keyword_or,
    keyword_any_player,
    keyword_settimer,
    keyword_endtimer,
    keyword_same_any,
    keyword_isbetween,
    keyword_con,
    keyword_constant_reference,
    keyword_counter,
    keyword_addone_counter,
    keyword_minusone_counter,
    keyword_reset_counter,

    command_kill,
    command_sleep,
    command_log,
    command_goto,
    command_sendkey,
    command_waitfor_dialog,
    command_waitfor_battle,
    command_waitfor_zonechange,
    command_waitfor_free,
    command_waitfor_window,
    command_usepotion,
    command_buypotions,
    command_relog,
    command_click,
    command_clickwindow,
    command_teleport,
    command_friendtp,
    command_entitytp,
    command_plus_teleport,
    command_minus_teleport,
    command_tozone,
    command_load_playstyle,
    command_set_yaw,
    command_nav,
    command_setdeck,
    command_getdeck,
    command_select_friend,
    command_autopet,
    command_set_goal,
    command_set_quest,
    command_set_zone,
    command_toggle_combat,
    command_restart_bot,
    command_move_cursor,
    command_move_cursor_window,

    // command expressions
    command_expr_window_visible,
    command_expr_in_zone,
    command_expr_same_zone,
    command_expr_same_quest,
    command_expr_same_yaw,
    command_expr_same_xyz,
    command_expr_playercount,
    command_expr_playercountabove,
    command_expr_playercountbelow,
    command_expr_tracking_quest,
    command_expr_tracking_goal,
    command_expr_loading,
    command_expr_in_combat,
    command_expr_has_dialogue,
    command_expr_has_xyz,
    command_expr_health_below,
    command_expr_health_above,
    command_expr_health,
    command_expr_bagcount,
    command_expr_bagcount_above,
    command_expr_bagcount_below,
    command_expr_mana,
    command_expr_mana_above,
    command_expr_mana_below,
    command_expr_energy,
    command_expr_energy_above,
    command_expr_energy_below,
    command_expr_in_range,
    command_expr_gold,
    command_expr_gold_above,
    command_expr_gold_below,
    command_expr_window_disabled,
    command_expr_same_place,
    command_expr_window_text,
    command_expr_potion_count,
    command_expr_potion_countabove,
    command_expr_potion_countbelow,
    command_expr_any_player_list,
    command_expr_has_quest,
    command_expr_has_yaw,
    command_expr_window_num,
    command_expr_item_dropped,
    command_expr_duel_round,
    command_expr_quest_changed,
    command_expr_goal_changed,
    command_expr_zone_changed,
    command_expr_account_level,

    colon, // :
    comma,

    plus,
    minus,
    star,
    slash,

    slash_slash,
    star_star,

    paren_open, // (
    paren_close, // )
    square_open, // [
    square_close, // ]
    curly_open, // {
    curly_close, // }

    identifier,

    END_LINE,
    END_FILE,
}

#[derive(Debug, Clone)]
pub struct LineInfo {
    pub line: usize,
    pub column: usize,
    pub last_column: usize,
    pub last_line: usize,
    pub filename: Option<String>,
}

impl LineInfo {
    pub fn new(line: usize, column: usize, last_column: usize, last_line: Option<usize>, filename: Option<String>) -> Self {
        Self {
            line,
            column,
            last_column,
            last_line: last_line.unwrap_or(line),
            filename,
        }
    }
}

impl fmt::Display for LineInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref filename) = self.filename {
            write!(f, "{}:{}:{}-{}", filename, self.line, self.column, self.last_column)
        } else {
            write!(f, "{}:{}-{}", self.line, self.column, self.last_column)
        }
    }
}

#[derive(Debug, Clone)]
pub enum TokenValue {
    None,
    String(String),
    Number(f64),
    Percent(f64),
    Path(Vec<String>),
    Int(i32),
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub literal: String,
    pub line_info: LineInfo,
    pub value: TokenValue,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {:?}`{}`({:?})", self.line_info, self.kind, self.literal, self.value)
    }
}

/// Python: `render_tokens(toks)` — tokenizer.py:165
pub fn render_tokens(toks: &[Token]) -> String {
    let mut lines_strs: HashMap<usize, String> = HashMap::new();
    let mut max_line = 0;
    for tok in toks {
        let line = tok.line_info.line;
        if line > max_line { max_line = line; }
        let current_str = lines_strs.entry(line).or_insert_with(String::new);
        let spaces_needed = if tok.line_info.column > 1 {
            tok.line_info.column - 1 - current_str.len()
        } else {
            0
        };
        for _ in 0..spaces_needed {
            current_str.push(' ');
        }
        current_str.push_str(&tok.literal);
    }
    
    let mut sorted_lines: Vec<usize> = lines_strs.keys().cloned().collect();
    sorted_lines.sort();
    
    let mut result = Vec::new();
    for line in sorted_lines {
        result.push(lines_strs[&line].clone());
    }
    result.join("\n")
}

/// Python: `normalize_ident(dirty)` — tokenizer.py:177
pub fn normalize_ident(dirty: &str) -> String {
    dirty.to_lowercase().replace('_', "")
}

pub struct Tokenizer {
    in_multiline_string: bool,
    multiline_buffer: String,
    multiline_start_line_info: LineInfo,
}

impl Tokenizer {
    pub fn new() -> Self {
        Self {
            in_multiline_string: false,
            multiline_buffer: String::new(),
            multiline_start_line_info: LineInfo::new(0, 0, 0, None, None),
        }
    }

    /// Python: `tokenize_line(l, line_num, filename)` — tokenizer.py:186
    pub fn tokenize_line(&mut self, l: &str, line_num: usize, filename: Option<String>) -> Result<Vec<Token>, TokenizerError> {
        let mut result = Vec::new();
        let mut i = 0;
        let chars: Vec<char> = l.chars().collect();

        while i < chars.len() {
            let c = chars[i];

            if self.in_multiline_string {
                self.multiline_buffer.push(c);
                if c == '`' {
                    self.multiline_start_line_info.last_column = i + 1;
                    self.multiline_start_line_info.last_line = line_num + 1;
                    let value = if self.multiline_buffer.len() >= 2 {
                        self.multiline_buffer[1..self.multiline_buffer.len()-1].to_string()
                    } else {
                        String::new()
                    };
                    result.push(Token {
                        kind: TokenKind::string,
                        literal: self.multiline_buffer.clone(),
                        line_info: self.multiline_start_line_info.clone(),
                        value: TokenValue::String(value),
                    });
                    self.in_multiline_string = false;
                    self.multiline_buffer.clear();
                }
                i += 1;
            } else {
                match c {
                    '&' if i + 1 < chars.len() && chars[i + 1] == '&' => {
                        result.push(self.make_token(TokenKind::logical_and, "&&", line_num, i, &filename, TokenValue::None));
                        i += 2;
                    }
                    ':' => {
                        result.push(self.make_token(TokenKind::colon, ":", line_num, i, &filename, TokenValue::None));
                        i += 1;
                    }
                    ',' => {
                        result.push(self.make_token(TokenKind::comma, ",", line_num, i, &filename, TokenValue::None));
                        i += 1;
                    }
                    '+' => {
                        result.push(self.make_token(TokenKind::plus, "+", line_num, i, &filename, TokenValue::None));
                        i += 1;
                    }
                    '-' => {
                        result.push(self.make_token(TokenKind::minus, "-", line_num, i, &filename, TokenValue::None));
                        i += 1;
                    }
                    '>' => {
                        result.push(self.make_token(TokenKind::greater, ">", line_num, i, &filename, TokenValue::None));
                        i += 1;
                    }
                    '<' => {
                        result.push(self.make_token(TokenKind::less, "<", line_num, i, &filename, TokenValue::None));
                        i += 1;
                    }
                    '=' => {
                        if i + 1 < chars.len() && chars[i + 1] == '=' {
                            result.push(self.make_token(TokenKind::equals, "==", line_num, i, &filename, TokenValue::None));
                            i += 2;
                        } else {
                            result.push(self.make_token(TokenKind::equals, "=", line_num, i, &filename, TokenValue::None));
                            i += 1;
                        }
                    }
                    '*' => {
                        if i + 1 < chars.len() && chars[i + 1] == '*' {
                            result.push(self.make_token(TokenKind::star_star, "**", line_num, i, &filename, TokenValue::None));
                            i += 2;
                        } else {
                            result.push(self.make_token(TokenKind::star, "*", line_num, i, &filename, TokenValue::None));
                            i += 1;
                        }
                    }
                    '/' => {
                        if i + 1 < chars.len() && chars[i + 1] == '/' {
                            result.push(self.make_token(TokenKind::slash_slash, "//", line_num, i, &filename, TokenValue::None));
                            i += 2;
                        } else {
                            result.push(self.make_token(TokenKind::slash, "/", line_num, i, &filename, TokenValue::None));
                            i += 1;
                        }
                    }
                    '(' => {
                        result.push(self.make_token(TokenKind::paren_open, "(", line_num, i, &filename, TokenValue::None));
                        i += 1;
                    }
                    ')' => {
                        result.push(self.make_token(TokenKind::paren_close, ")", line_num, i, &filename, TokenValue::None));
                        i += 1;
                    }
                    '[' => {
                        result.push(self.make_token(TokenKind::square_open, "[", line_num, i, &filename, TokenValue::None));
                        i += 1;
                    }
                    ']' => {
                        result.push(self.make_token(TokenKind::square_close, "]", line_num, i, &filename, TokenValue::None));
                        i += 1;
                    }
                    '{' => {
                        result.push(self.make_token(TokenKind::curly_open, "{", line_num, i, &filename, TokenValue::None));
                        i += 1;
                    }
                    '}' => {
                        result.push(self.make_token(TokenKind::curly_close, "}", line_num, i, &filename, TokenValue::None));
                        i += 1;
                    }
                    '"' | '\'' => {
                        let quote_kind = c;
                        let mut str_lit = String::from(c);
                        let mut j = i + 1;
                        while j < chars.len() && chars[j] != quote_kind {
                            str_lit.push(chars[j]);
                            j += 1;
                        }
                        if j >= chars.len() {
                            return Err(TokenizerError::Error(format!("Unclosed string encountered\n{}\n{}^\nLine: {} | Column: {}", l, " ".repeat(i), line_num, i+1)));
                        }
                        str_lit.push(chars[j]);
                        j += 1;
                        let val = str_lit[1..str_lit.len()-1].to_string();
                        result.push(self.make_token(TokenKind::string, &str_lit, line_num, i, &filename, TokenValue::String(val)));
                        i = j;
                    }
                    '`' => {
                        self.multiline_buffer = String::from(c);
                        self.in_multiline_string = true;
                        self.multiline_start_line_info = LineInfo::new(line_num, i + 1, i + 1, Some(line_num), filename.clone());
                        i += 1;
                    }
                    '#' => break,
                    _ if c.is_whitespace() => i += 1,
                    _ => {
                        let mut full = String::new();
                        let mut j = i;
                        while j < chars.len() && !chars[j].is_whitespace() && !"():[],`".contains(chars[j]) {
                            full.push(chars[j]);
                            j += 1;
                        }

                        if !full.is_empty() {
                            if full.chars().all(|x| x.is_numeric() || ".e-%".contains(x)) {
                                if full.contains('%') {
                                    if let Ok(num) = full[..full.len()-1].parse::<f64>() {
                                        result.push(self.make_token(TokenKind::percent, &full, line_num, i, &filename, TokenValue::Percent(num / 100.0)));
                                    } else {
                                        return Err(TokenizerError::Error(format!("Unable to convert to percent\n{}\n{}^\nLine: {} | Column: {}", l, " ".repeat(i), line_num, i+1)));
                                    }
                                } else {
                                    if let Ok(num) = full.parse::<f64>() {
                                        result.push(self.make_token(TokenKind::number, &full, line_num, i, &filename, TokenValue::Number(num)));
                                    } else {
                                        return Err(TokenizerError::Error(format!("Unable to convert to number\n{}\n{}^\nLine: {} | Column: {}", l, " ".repeat(i), line_num, i+1)));
                                    }
                                }
                            } else if full.contains('/') {
                                if full.ends_with('/') {
                                    return Err(TokenizerError::Error(format!("Invalid path\n{}\n{}^\nLine: {} | Column: {}", l, " ".repeat(i), line_num, i+1)));
                                }
                                let parts: Vec<String> = full.split('/').map(|s| s.to_string()).collect();
                                result.push(self.make_token(TokenKind::path, &full, line_num, i, &filename, TokenValue::Path(parts)));
                            } else if full.to_lowercase().starts_with('p') && full[1..].chars().all(|x| x.is_numeric()) {
                                if let Ok(num) = full[1..].parse::<i32>() {
                                    result.push(self.make_token(TokenKind::player_num, &full, line_num, i, &filename, TokenValue::Int(num)));
                                }
                            } else {
                                let norm = normalize_ident(&full);
                                let kind = match norm.as_str() {
                                    "block" => TokenKind::keyword_block,
                                    "call" => TokenKind::keyword_call,
                                    "loop" => TokenKind::keyword_loop,
                                    "while" => TokenKind::keyword_while,
                                    "until" => TokenKind::keyword_until,
                                    "times" => TokenKind::keyword_times,
                                    "if" => TokenKind::keyword_if,
                                    "else" => TokenKind::keyword_else,
                                    "elif" => TokenKind::keyword_elif,
                                    "except" => TokenKind::keyword_except,
                                    "mass" => TokenKind::keyword_mass,
                                    "closestmob" | "mob" => TokenKind::keyword_mob,
                                    "quest" | "questpos" | "questposition" => TokenKind::keyword_quest,
                                    "icon" => TokenKind::keyword_icon,
                                    "ifneeded" => TokenKind::keyword_ifneeded,
                                    "completion" => TokenKind::keyword_completion,
                                    "xyz" => TokenKind::keyword_xyz,
                                    "orient" => TokenKind::keyword_orient,
                                    "not" => TokenKind::keyword_not,
                                    "return" | "exitblock" => TokenKind::keyword_return,
                                    "break" | "exitloop" => TokenKind::keyword_break,
                                    "mixin" => TokenKind::keyword_mixin,
                                    "and" => TokenKind::keyword_and,
                                    "or" => TokenKind::keyword_or,
                                    "anyplayer" | "anyclient" | "any" => TokenKind::keyword_any_player,
                                    "createtimer" | "starttimer" | "logtimer" => TokenKind::keyword_settimer,
                                    "endtimer" | "canceltimer" | "stoptimer" => TokenKind::keyword_endtimer,
                                    "sameany" | "sameanyplayer" | "sameanyclient" => TokenKind::keyword_same_any,
                                    "isbetween" | "between" => TokenKind::keyword_isbetween,
                                    "from" => TokenKind::logical_from,
                                    "to" => TokenKind::logical_to,
                                    "on" => TokenKind::logical_on,
                                    "off" => TokenKind::logical_off,
                                    "con" | "set" | "setvar" | "var" => TokenKind::keyword_con,
                                    "true" => TokenKind::boolean_true,
                                    "false" => TokenKind::boolean_false,
                                    "$" => TokenKind::keyword_constant_reference,
                                    "rerun" | "restart" | "restartbot" => TokenKind::command_restart_bot,
                                    "startcounter" | "counter" | "createcounter" => TokenKind::keyword_counter,
                                    "endcounter" | "deletecounter" | "resetcounter" => TokenKind::keyword_reset_counter,
                                    "addone" => TokenKind::keyword_addone_counter,
                                    "minusone" => TokenKind::keyword_minusone_counter,

                                    "kill" | "killbot" | "stop" | "stopbot" | "end" | "exit" => TokenKind::command_kill,
                                    "sleep" | "wait" | "delay" => TokenKind::command_sleep,
                                    "log" | "debug" | "print" => TokenKind::command_log,
                                    "teleport" | "tp" | "setpos" => TokenKind::command_teleport,
                                    "goto" | "walkto" => TokenKind::command_goto,
                                    "sendkey" | "press" | "presskey" => TokenKind::command_sendkey,
                                    "waitfordialog" | "waitfordialogue" => TokenKind::command_waitfor_dialog,
                                    "waitforbattle" | "waitforcombat" => TokenKind::command_waitfor_battle,
                                    "waitforzonechange" => TokenKind::command_waitfor_zonechange,
                                    "waitforfree" => TokenKind::command_waitfor_free,
                                    "waitforwindow" | "waitforpath" => TokenKind::command_waitfor_window,
                                    "usepotion" => TokenKind::command_usepotion,
                                    "buypotions" | "refillpotions" | "buypots" | "refillpots" => TokenKind::command_buypotions,
                                    "relog" | "logoutandin" => TokenKind::command_relog,
                                    "click" => TokenKind::command_click,
                                    "clickwindow" => TokenKind::command_clickwindow,
                                    "friendtp" | "friendteleport" => TokenKind::command_friendtp,
                                    "entitytp" | "entityteleport" => TokenKind::command_entitytp,
                                    "tozone" => TokenKind::command_tozone,
                                    "loadplaystyle" => TokenKind::command_load_playstyle,
                                    "turncam" | "setcamyaw" => TokenKind::command_set_yaw,
                                    "nav" | "navtp" => TokenKind::command_nav,
                                    "getdeck" => TokenKind::command_getdeck,
                                    "setdeck" => TokenKind::command_setdeck,
                                    "selectfriend" | "choosefriend" => TokenKind::command_select_friend,
                                    "plustp" | "plusteleport" => TokenKind::command_plus_teleport,
                                    "minustp" | "minusteleport" => TokenKind::command_minus_teleport,
                                    "autopet" | "toggleautopet" => TokenKind::command_autopet,
                                    "loggoal" => TokenKind::command_set_goal,
                                    "logquest" => TokenKind::command_set_quest,
                                    "logzone" => TokenKind::command_set_zone,
                                    "togglecombat" | "togglecombatmode" => TokenKind::command_toggle_combat,
                                    "cursor" | "movecursor" | "mousexy" | "movemouse" => TokenKind::command_move_cursor,
                                    "cursorwindow" | "mousewindow" => TokenKind::command_move_cursor_window,

                                    "contains" => TokenKind::contains,
                                    "windowvisible" => TokenKind::command_expr_window_visible,
                                    "inzone" => TokenKind::command_expr_in_zone,
                                    "samezone" => TokenKind::command_expr_same_zone,
                                    "playercount" | "clientcount" => TokenKind::command_expr_playercount,
                                    "playercountabove" | "clientcountabove" => TokenKind::command_expr_playercountabove,
                                    "playercountbelow" | "clientcountbelow" => TokenKind::command_expr_playercountbelow,
                                    "trackingquest" => TokenKind::command_expr_tracking_quest,
                                    "trackinggoal" => TokenKind::command_expr_tracking_goal,
                                    "loading" => TokenKind::command_expr_loading,
                                    "incombat" => TokenKind::command_expr_in_combat,
                                    "hasdialogue" => TokenKind::command_expr_has_dialogue,
                                    "hasxyz" => TokenKind::command_expr_has_xyz,
                                    "healthbelow" => TokenKind::command_expr_health_below,
                                    "healthabove" => TokenKind::command_expr_health_above,
                                    "health" => TokenKind::command_expr_health,
                                    "manabelow" => TokenKind::command_expr_mana_below,
                                    "manaabove" => TokenKind::command_expr_mana_above,
                                    "mana" => TokenKind::command_expr_mana,
                                    "energybelow" => TokenKind::command_expr_energy_below,
                                    "energyabove" => TokenKind::command_expr_energy_above,
                                    "energy" => TokenKind::command_expr_energy,
                                    "bagcount" => TokenKind::command_expr_bagcount,
                                    "bagcountbelow" => TokenKind::command_expr_bagcount_below,
                                    "bagcountabove" => TokenKind::command_expr_bagcount_above,
                                    "gold" => TokenKind::command_expr_gold,
                                    "goldabove" => TokenKind::command_expr_gold_above,
                                    "goldbelow" => TokenKind::command_expr_gold_below,
                                    "windowdisabled" => TokenKind::command_expr_window_disabled,
                                    "sameplace" => TokenKind::command_expr_same_place,
                                    "windowtext" => TokenKind::command_expr_window_text,
                                    "potioncount" => TokenKind::command_expr_potion_count,
                                    "potioncountabove" => TokenKind::command_expr_potion_countabove,
                                    "potioncountbelow" => TokenKind::command_expr_potion_countbelow,
                                    "hasquest" => TokenKind::command_expr_has_quest,
                                    "inrange" => TokenKind::command_expr_in_range,
                                    "hasyaw" => TokenKind::command_expr_has_yaw,
                                    "sameyaw" => TokenKind::command_expr_same_yaw,
                                    "samexyz" => TokenKind::command_expr_same_xyz,
                                    "samequest" => TokenKind::command_expr_same_quest,
                                    "anyplayerlist" | "anyclientlist" => TokenKind::command_expr_any_player_list,
                                    "windownum" => TokenKind::command_expr_window_num,
                                    "itemdropped" => TokenKind::command_expr_item_dropped,
                                    "combatround" | "duelround" | "fightround" => TokenKind::command_expr_duel_round,
                                    "questchanged" => TokenKind::command_expr_quest_changed,
                                    "goalchanged" => TokenKind::command_expr_goal_changed,
                                    "zonechanged" => TokenKind::command_expr_zone_changed,
                                    "accountlevel" | "level" => TokenKind::command_expr_account_level,

                                    _ => TokenKind::identifier,
                                };
                                result.push(self.make_token(kind, &full, line_num, i, &filename, TokenValue::None));
                            }
                        }
                        i = j;
                    }
                }
            }
        }
        if !self.in_multiline_string {
            result.push(self.make_token(TokenKind::END_LINE, "", line_num, i, &filename, TokenValue::None));
        }
        Ok(result)
    }

    fn make_token(&self, kind: TokenKind, literal: &str, line_num: usize, i: usize, filename: &Option<String>, value: TokenValue) -> Token {
        Token {
            kind,
            literal: literal.to_string(),
            line_info: LineInfo::new(line_num, i + 1, i + literal.len() + 1, Some(line_num), filename.clone()),
            value,
        }
    }

    /// Python: `tokenize(contents, filename)` — tokenizer.py:539
    pub fn tokenize(&mut self, contents: &str, filename: Option<String>) -> Result<Vec<Token>, TokenizerError> {
        let mut result = Vec::new();
        for (line_num, line) in contents.lines().enumerate() {
            let toks = self.tokenize_line(line, line_num + 1, filename.clone())?;
            if self.in_multiline_string {
                self.multiline_buffer.push('\n');
            } else if toks.len() == 1 {
                // only end line
                continue;
            }
            result.extend(toks);
        }
        if self.in_multiline_string {
            return Err(TokenizerError::Error(format!("Unclosed multiline string: {} {}", self.multiline_buffer, self.multiline_start_line_info)));
        }
        Ok(result)
    }
}
