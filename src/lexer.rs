use core::ffi::*;
use core::ptr;
use core::mem::zeroed;
use crate::nob::*;
use crate::crust::libc::*;
use crate::ir::{Radix};
use crate::errors::bump_error_count;

#[derive(Clone, Copy)]
pub struct Loc {
    pub input_path: *const c_char,
    pub line_number: c_int,
    pub line_offset: c_int,
}

#[macro_export]
macro_rules! diagf {
    ($loc:expr, $($args:tt)*) => {{
        fprintf(stderr(), c!("%s:%d:%d: "), $loc.input_path, $loc.line_number, $loc.line_offset);
        fprintf(stderr(), $($args)*);
    }};
}

#[macro_export]
macro_rules! missingf {
    ($loc:expr, $($args:tt)*) => {{
        let file = file!();
        fprintf(stderr(), c!("%s:%d:%d: TODO: "), $loc.input_path, $loc.line_number, $loc.line_offset);
        fprintf(stderr(), $($args)*);
        fprintf(stderr(), c!("%.*s:%d: INFO: implementation should go here\n"), file.len(), file.as_ptr(), line!());
        abort();
    }}
}

#[derive(Clone, Copy, PartialEq)]
pub enum Token {
    // Terminal
    EOF,
    ParseError,

    // Values
    ID,
    String,
    CharLit,
    IntLit,

    // Puncts
    OCurly,
    CCurly,
    OParen,
    CParen,
    OBracket,
    CBracket,
    Not,
    Mul,
    Div,
    Mod,
    And,
    Plus,
    PlusPlus,
    Minus,
    MinusMinus,
    Less,
    LessEq,
    Greater,
    GreaterEq,
    Or,
    Eq,
    EqEq,
    NotEq,
    Shl,
    ShlEq,
    Shr,
    ShrEq,
    ModEq,
    OrEq,
    AndEq,
    PlusEq,
    MinusEq,
    MulEq,
    DivEq,
    Question,
    Colon,
    SemiColon,
    Comma,

    // Keywords
    Auto,
    Extrn,
    Case,
    If,
    Else,
    While,
    Switch,
    Goto,
    Return,
    Asm,
    Variadic,
}

pub unsafe fn display_token(token: Token) -> *const c_char {
    match token {
        // Terminal
        Token::EOF        => c!("end of file"),
        Token::ParseError => c!("parse error"),

        // Values
        Token::ID         => c!("identifier"),
        Token::String     => c!("string"),
        Token::CharLit    => c!("character"),
        Token::IntLit     => c!("integer literal"),

        // Puncts
        Token::OCurly     => c!("`{`"),
        Token::CCurly     => c!("`}`"),
        Token::OParen     => c!("`(`"),
        Token::CParen     => c!("`)`"),
        Token::OBracket   => c!("`[`"),
        Token::CBracket   => c!("`]`"),
        Token::Not        => c!("`!`"),
        Token::Mul        => c!("`*`"),
        Token::Div        => c!("`/`"),
        Token::Mod        => c!("`%`"),
        Token::And        => c!("`&`"),
        Token::Plus       => c!("`+`"),
        Token::PlusPlus   => c!("`++`"),
        Token::Minus      => c!("`-`"),
        Token::MinusMinus => c!("`--`"),
        Token::Less       => c!("`<`"),
        Token::LessEq     => c!("`<=`"),
        Token::Greater    => c!("`>`"),
        Token::GreaterEq  => c!("`>=`"),
        Token::Or         => c!("`|`"),
        Token::NotEq      => c!("`!=`"),
        Token::Eq         => c!("`=`"),
        Token::EqEq       => c!("`==`"),
        Token::Shl        => c!("`<<`"),
        Token::ShlEq      => c!("`<<=`"),
        Token::Shr        => c!("`>>`"),
        Token::ShrEq      => c!("`>>=`"),
        Token::ModEq      => c!("`%=`"),
        Token::OrEq       => c!("`|=`"),
        Token::AndEq      => c!("`&=`"),
        Token::PlusEq     => c!("`+=`"),
        Token::MinusEq    => c!("`-=`"),
        Token::MulEq      => c!("`*=`"),
        Token::DivEq      => c!("`/=`"),
        Token::Question   => c!("`?`"),
        Token::Colon      => c!("`:`"),
        Token::SemiColon  => c!("`;`"),
        Token::Comma      => c!("`,`"),

        Token::Auto       => c!("keyword `auto`"),
        Token::Extrn      => c!("keyword `extrn`"),
        Token::Case       => c!("keyword `case`"),
        Token::If         => c!("keyword `if`"),
        Token::Else       => c!("keyword `else`"),
        Token::While      => c!("keyword `while`"),
        Token::Switch     => c!("keyword `switch`"),
        Token::Goto       => c!("keyword `goto`"),
        Token::Return     => c!("keyword `return`"),

        // TODO: document all this magical extension keywords somewhere
        Token::Asm        => c!("keyword `__asm__`"),
        Token::Variadic   => c!("keyword `__variadic__`"),
    }
}

// IMPORTANT! The order of PUNCTS and HISTORICAL_PUNCTS is important because they are checked as prefixes of input sequentially.
//   It's important to keep `+=` before `+` because otherwise `+=` may end up getting tokenized as `+` and `=`.
//   As a rule of thumb, if one token is a substring of another one, keep the array index of the longer one lower
//   so it's checked earlier.
//   TODO: Maybe we should create a function that analyses the PUNCTS and orders them accordingly, so this notice is
//   not needed
pub const MODERN_PUNCTS: *const [(*const c_char, Token)] = &[
    (c!("-="), Token::MinusEq),
    (c!("+="), Token::PlusEq),
    (c!("*="), Token::MulEq),
    (c!("%="), Token::ModEq),
    (c!("/="), Token::DivEq),
    (c!("|="), Token::OrEq),
    (c!("&="), Token::AndEq),
    (c!("<<="), Token::ShlEq),
    (c!(">>="), Token::ShrEq),
];

pub const HISTORICAL_PUNCTS: *const [(*const c_char, Token)] = &[
    (c!("=-"), Token::MinusEq),
    (c!("=+"), Token::PlusEq),
    (c!("=*"), Token::MulEq),
    (c!("=%"), Token::ModEq),
    (c!("=/"), Token::DivEq),
    (c!("=|"), Token::OrEq),
    (c!("=&"), Token::AndEq),
    (c!("=="), Token::EqEq),
    (c!("=<<"), Token::ShlEq),
    (c!("=>>"), Token::ShrEq),
];

pub const COMMON_PUNCTS: *const [(*const c_char, Token)] = &[
    (c!("?"), Token::Question),
    (c!("{"), Token::OCurly),
    (c!("}"), Token::CCurly),
    (c!("("), Token::OParen),
    (c!(")"), Token::CParen),
    (c!("["), Token::OBracket),
    (c!("]"), Token::CBracket),
    (c!(";"), Token::SemiColon),
    (c!(":"), Token::Colon),
    (c!(","), Token::Comma),
    (c!("--"), Token::MinusMinus),
    (c!("-"), Token::Minus),
    (c!("++"), Token::PlusPlus),
    (c!("+"), Token::Plus),
    (c!("-"), Token::Minus),
    (c!("*"), Token::Mul),
    (c!("%"), Token::Mod),
    (c!("/"), Token::Div),
    (c!("|"), Token::Or),
    (c!("&"), Token::And),
    (c!("=="), Token::EqEq),
    (c!("!="), Token::NotEq),
    (c!("!"), Token::Not),
    (c!("<<"), Token::Shl),
    (c!("<="), Token::LessEq),
    (c!("<"), Token::Less),
    (c!(">>"), Token::Shr),
    (c!(">="), Token::GreaterEq),

    (c!("="), Token::Eq),
    (c!(">"), Token::Greater),
];
const KEYWORDS: *const [(*const c_char, Token)] = &[
    (c!("auto"), Token::Auto),
    (c!("extrn"), Token::Extrn),
    (c!("case"), Token::Case),
    (c!("if"), Token::If),
    (c!("else"), Token::Else),
    (c!("while"), Token::While),
    (c!("switch"), Token::Switch),
    (c!("goto"), Token::Goto),
    (c!("return"), Token::Return),
    (c!("__asm__"), Token::Asm),
    (c!("__variadic__"), Token::Variadic),
];

#[derive(Clone, Copy)]
pub struct Parse_Point {
    pub current: *const c_char,
    pub line_start: *const c_char,
    pub line_number: usize,
}

#[derive(Clone, Copy)]
pub struct Lexer {
    pub input_path: *const c_char,
    pub input_stream: *const c_char,
    pub eof: *const c_char,
    pub parse_point: Parse_Point,

    pub historical: bool,
    pub string_storage: String_Builder,
    pub token: Token,
    pub string: *const c_char,
    pub radix: Radix,
    pub loc: Loc,
    pub error_count: *mut usize, 
}

pub unsafe fn new(input_path: *const c_char, input_stream: *const c_char, eof: *const c_char, historical: bool, error_count: *mut usize) -> Lexer {
    let mut l: Lexer = zeroed();
    l.input_path              = input_path;
    l.input_stream            = input_stream;
    l.eof                     = eof;
    l.parse_point.current     = input_stream;
    l.parse_point.line_start  = input_stream;
    l.parse_point.line_number = 1;
    l.historical              = historical;
    l.error_count             = error_count;
    l
}

pub unsafe fn is_eof(l: *mut Lexer) -> bool {
    (*l).parse_point.current >= (*l).eof
}

pub unsafe fn peek_char(l: *mut Lexer) -> Option<c_char> {
    if is_eof(l) {
        None
    } else {
        Some(*(*l).parse_point.current)
    }
}

pub unsafe fn skip_char(l: *mut Lexer) {
    assert!(!is_eof(l));

    let x = *(*l).parse_point.current;
    (*l).parse_point.current = (*l).parse_point.current.add(1);
    if x == '\n' as c_char {
        (*l).parse_point.line_start = (*l).parse_point.current;
        (*l).parse_point.line_number += 1;
    }
}

pub unsafe fn skip_whitespaces(l: *mut Lexer) {
    while let Some(x) = peek_char(l) {
        if isspace(x as i32) != 0 {
            skip_char(l)
        } else {
            break
        }
    }
}

#[must_use]
pub unsafe fn skip_whitespaces_and_comments(l: *mut Lexer) -> Option<()> {
    'comments: loop {
        skip_whitespaces(l);

        let saved_point = (*l).parse_point;

        if skip_prefix(l, c!("//")) {
            skip_until(l, c!("\n"));
            if (*l).historical {
                let end_point = (*l).parse_point;
                (*l).parse_point = saved_point;
                diagf!(loc(l), c!("LEXER ERROR: C++ style comments are not available in the historical mode.\n"));
                (*l).parse_point = end_point;
                bump_error_count((*l).error_count)?;
            }
            continue 'comments;
        }

        let begin_loc = loc(l);
        if skip_prefix(l, c!("/*")) {
            while !skip_prefix(l, c!("*/")) {
                if is_eof(l) {
                    diagf!(loc(l), c!("LEXER ERROR: Unfinished comment\n"));
                    diagf!(begin_loc, c!("LEXER INFO: Comment starts here\n"));
                    (*l).token = Token::ParseError;
                    return None;
                }
                skip_char(l);
            }
            continue 'comments;
        }

        break 'comments;
    }
    Some(())
}

pub unsafe fn skip_prefix(l: *mut Lexer, mut prefix: *const c_char) -> bool {
    let saved_point = (*l).parse_point;
    while *prefix != 0 {
        let Some(x) = peek_char(l) else {
            (*l).parse_point = saved_point;
            return false;
        };
        if x != *prefix {
            (*l).parse_point = saved_point;
            return false;
        }
        skip_char(l);
        prefix = prefix.add(1);
    }
    true
}

pub unsafe fn skip_until(l: *mut Lexer, prefix: *const c_char) {
    while !is_eof(l) && !skip_prefix(l, prefix) {
        skip_char(l);
    }
}

pub unsafe fn is_identifier(x: c_char) -> bool {
    isalnum(x as c_int) != 0 || x == '_' as c_char
}

pub unsafe fn is_identifier_start(x: c_char) -> bool {
    isalpha(x as c_int) != 0 || x == '_' as c_char
}

pub unsafe fn loc(l: *mut Lexer) -> Loc {
    Loc {
        input_path:  (*l).input_path,
        line_number: (*l).parse_point.line_number as i32,
        line_offset: (*l).parse_point.current.offset_from((*l).parse_point.line_start) as i32 + 1,
    }
}

#[must_use]
pub unsafe fn parse_string_into_storage(l: *mut Lexer, delim: c_char) -> Option<()> {
    let escape_chars: &[c_char] = if !(*l).historical {
        &['\\' as c_char, '*' as c_char]
    } else {
        &['*' as c_char]
    };
    
    while let Some(x) = peek_char(l) {
        match x {
            x if escape_chars.contains(&x) => {
                let current_escape = x;
                skip_char(l);
                let Some(x) = peek_char(l) else {
                    (*l).token = Token::ParseError;
                    diagf!(loc(l), c!("LEXER ERROR: Unfinished escape sequence\n"));
                    return None;
                };
                let x = match x {
                    x if x == '0'   as c_char => '\0' as c_char,
                    x if x == 'n'   as c_char => '\n' as c_char,
                    x if x == 't'   as c_char => '\t' as c_char,
                    x if x == 'r'   as c_char => '\r' as c_char,
                    x if x == delim           => delim,
                    x if x == current_escape => current_escape,
                    x => {
                        diagf!(loc(l), c!("LEXER ERROR: Unknown escape sequence starting with `%c`\n"), x as c_int);
                        skip_char(l);
                        bump_error_count((*l).error_count)?;
                        continue;
                    }
                };
                da_append(&mut (*l).string_storage, x);
                skip_char(l);
            },
            x if x == delim => break,
            _ => {
                da_append(&mut (*l).string_storage, x);
                skip_char(l);
            },
        }
    }
    Some(())
}

unsafe fn parse_digit(c: c_char, radix: Radix) -> Option<u8> {
    if isdigit(c as c_int) != 0 {
        let digit = c as u8 - '0' as u8;
        if digit >= (radix as u8) {
            return None;
        }

        return Some(digit as u8);
    }

    if matches!(radix, Radix::Hex) {
        let c = tolower(c as c_int) as c_char;
        if !('a' as c_char <= c && c <= 'f' as c_char) {
            return None;
        }
        return Some(c as u8 - 'a' as u8 + 10);
    }

    return None;
}

#[must_use]
unsafe fn parse_number(l: *mut Lexer, radix: Radix) -> Option<()> {
    while let Some(x) = peek_char(l) {
        let Some(_) = parse_digit(x, radix) else {
            break;
        };
        da_append(&mut (*l).string_storage, x);
        skip_char(l);
    };
    da_append(&mut (*l).string_storage, 0);
    (*l).radix = radix;
    (*l).string = (*l).string_storage.items;

    if strlen((*l).string) == 0 {
        None
    } else {
        Some(())
    }
}

#[must_use]
pub unsafe fn get_token(l: *mut Lexer) -> Option<()> {
    (*l).string = ptr::null();
    skip_whitespaces_and_comments(l)?;

    (*l).loc = loc(l);

    let Some(x) = peek_char(l) else {
        (*l).token = Token::EOF;
        return Some(())
    };

    let mut puncs = HISTORICAL_PUNCTS;
    for i in 0..puncs.len() {
        let (prefix, token) = (*puncs)[i];
        if skip_prefix(l, prefix) {
            (*l).token = token;
            return Some(())
        }
    }
    if !(*l).historical { 
        puncs = MODERN_PUNCTS;
        for i in 0..puncs.len() {
            let (prefix, token) = (*puncs)[i];
            if skip_prefix(l, prefix) {
                (*l).token = token;
                return Some(())
            }
        }
    }
    puncs = COMMON_PUNCTS;
    for i in 0..puncs.len() {
        let (prefix, token) = (*puncs)[i];
        if skip_prefix(l, prefix) {
            (*l).token = token;
            return Some(())
        }
    }

    if is_identifier_start(x) {
        (*l).token = Token::ID;
        (*l).string_storage.count = 0;
        while let Some(x) = peek_char(l) {
            if is_identifier(x) {
                da_append(&mut (*l).string_storage, x);
                skip_char(l);
            } else {
                break
            }
        }
        da_append(&mut (*l).string_storage, 0);
        (*l).string = (*l).string_storage.items;

        for i in 0..KEYWORDS.len() {
            let (id, token) = (*KEYWORDS)[i];
            if strcmp((*l).string, id) == 0 {
                (*l).token = token;
                return Some(());
            }
        }

        return Some(())
    }

    let start_of_number = (*l).parse_point;
    if skip_prefix(l, c!("0x")) {
        let value = parse_number(l, Radix::Hex);
        if (*l).historical {
            let end_point = (*l).parse_point;
            (*l).parse_point = start_of_number;
            diagf!(loc(l), c!("LEXER ERROR: hex literals are not available in the historical mode.\n"));
            (*l).parse_point = end_point;
            bump_error_count((*l).error_count)?;
        }
        (*l).token = Token::IntLit;
        (*l).string_storage.count = 0;
        return value;
    }

    if skip_prefix(l, c!("0")) {
        (*l).token = Token::IntLit;
        (*l).string_storage.count = 0;
        if let Some(_) = parse_number(l, Radix::Oct) {
            return Some(());
        } else {
            (*l).parse_point = start_of_number;
        }
    }

    if isdigit(x as c_int) != 0 {
        (*l).token = Token::IntLit;
        (*l).string_storage.count = 0;
        return parse_number(l, Radix::Dec);
    }

    if x == '"' as c_char {
        skip_char(l);
        (*l).token = Token::String;
        (*l).string_storage.count = 0;
        parse_string_into_storage(l, '"' as c_char)?;
        if is_eof(l) {
            diagf!(loc(l), c!("LEXER ERROR: Unfinished string literal\n"));
            diagf!((*l).loc, c!("LEXER INFO: Literal starts here\n"));
            (*l).token = Token::ParseError;
            return None;
        }
        skip_char(l);
        da_append(&mut (*l).string_storage, 0);
        (*l).string = (*l).string_storage.items;
        return Some(());
    }

    if x == '\'' as c_char {
        skip_char(l);
        (*l).token = Token::CharLit;
        (*l).string_storage.count = 0;
        parse_string_into_storage(l, '\'' as c_char)?;
        if is_eof(l) {
            diagf!(loc(l), c!("LEXER ERROR: Unfinished character literal\n"));
            diagf!((*l).loc, c!("LEXER INFO: Literal starts here\n"));
            (*l).token = Token::ParseError;
            return None;
        }
        skip_char(l);
        if (*l).string_storage.count == 0 {
            diagf!((*l).loc, c!("LEXER ERROR: Empty character literal\n"));
            bump_error_count((*l).error_count)?;
        }
        da_append(&mut (*l).string_storage, 0);
        (*l).string = (*l).string_storage.items;
        
        return Some(());
    }

    diagf!((*l).loc, c!("LEXER ERROR: Unknown token '%c'\n"), *(*l).parse_point.current as c_int);
    skip_char(l);
    (*l).token = Token::ParseError;
    bump_error_count((*l).error_count)?;
    None
}

#[must_use]
pub unsafe fn get_next_valid_token(l: *mut Lexer) -> Option<()> {
    loop {
        if let Some(()) = get_token(l) {
            if (*l).token == Token::EOF {
                return None;
            }
            return Some(());
        }
    }
}

