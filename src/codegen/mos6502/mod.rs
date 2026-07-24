// This uses 16-bit words, because addresses in 6502 are 16bits, so otherwise pointers would not work.
// To emulate 16-bit words using 8-bit registers, we use Y to hold the high byte and A to hold the low byte.

// As 6502 has a fixed stack at $0100-$01FF, we only have 255 bytes available. Machine code is loaded at $E000 by default, but can be reconfigured via LOAD_OFFSET=<offset> "linker flag".

// "Calling convention": first argument in Y:A, remaining args on the stack.

use core::ffi::*;
use core::mem::zeroed;
use core::ptr;
use crate::lexer::*;
use crate::ir::*;
use crate::nob::*;
use crate::errors::bump_error_count;
use crate::diagf;
use crate::crust::libc::*;
use crate::lexer::{is_identifier_start, is_identifier};
use crate::arena::{self, Arena};
use crate::targets::TargetAPI;
use crate::params::*;
use crate::codegen_common::{parse_int_literal_to_u16, parse_char_literal_to_u16_le};

// TODO: does this have to be a macro?
macro_rules! instr_enum {
    (enum $n:ident { $($instr:ident),* }) => {
        #[derive(Clone, Copy)]
        #[repr(u8)]
        pub enum $n {
            $($instr),*,
            COUNT
        }

        // TODO: maybe not search linearly, if this is too slow
        pub unsafe fn instr_from_string(s: *const c_char) -> Option<Instr> {
            $(
                let curr = c!(stringify!($instr));
                if (strcmp(s, curr) == 0) {
                    return Some($n::$instr);
                }
            )*
            return None;
        }
    }
}

instr_enum! {
    enum Instr {
        ADC,
        AND,
        ASL,
        BCC, BCS, BEQ, BIT,
        BMI,
        BNE, BPL, BRK, BVC,
        BVS,
        CLC, CLD, CLI, CLV,
        CMP, CPX, CPY,
        DEC, DEX, DEY,
        EOR,
        INC, INX, INY,
        JMP, JSR,
        LDA, LDX, LDY,
        LSR,
        NOP,
        ORA,
        PHA, PHP, PLA, PLP,
        ROL, ROR,
        RTI, RTS,
        SBC,
        SEC, SED, SEI,
        STA, STX, STY,
        TAX, TAY, TSX, TXA, TXS, TYA
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum AddrMode {
    IMM = 0,
    ZP,
    ZP_X,
    ZP_Y,
    ABS,
    ABS_X,
    ABS_Y,
    IND_X,
    IND_Y,

    ACC,
    REL,
    IND,
    IMPL, // implied, no arg

    COUNT
}
use Instr::*;
use AddrMode::*;

// TODO: we currently use 0xFF for invalid opcode, because Some() and None
// make this table way too big/hard to read
const INVL: u8 = 0xFF;
const OPCODES: [[u8; AddrMode::COUNT as usize]; Instr::COUNT as usize] =
       [// IMM    ZP    ZP_X   ZP_Y,  ABS   ABS_X  ABS_Y  IND_X  IND_Y   ACC    REL   IND, IMPL
/*ADC*/[  0x69,  0x65,  0x75,  INVL, 0x6D,  0x7D,  0x79,  0x61,  0x71,  INVL,  INVL, INVL, INVL],
/*AND*/[  0x29,  0x25,  0x35,  INVL, 0x2D,  0x3D,  0x39,  0x21,  0x31,  INVL,  INVL, INVL, INVL],
/*ASL*/[  INVL,  0x06,  0x16,  INVL, 0x0E,  0x1E,  INVL,  INVL,  INVL,  0x0A,  INVL, INVL, INVL],
/*BCC*/[  INVL,  INVL,  INVL,  INVL, INVL,  INVL,  INVL,  INVL,  INVL,  INVL,  0x90, INVL, INVL],
/*BCS*/[  INVL,  INVL,  INVL,  INVL, INVL,  INVL,  INVL,  INVL,  INVL,  INVL,  0xB0, INVL, INVL],
/*BEQ*/[  INVL,  INVL,  INVL,  INVL, INVL,  INVL,  INVL,  INVL,  INVL,  INVL,  0xF0, INVL, INVL],
/*BIT*/[  INVL,  0x24,  INVL,  INVL, 0x2C,  INVL,  INVL,  INVL,  INVL,  INVL,  INVL, INVL, INVL],
/*BMI*/[  INVL,  INVL,  INVL,  INVL, INVL,  INVL,  INVL,  INVL,  INVL,  INVL,  0x30, INVL, INVL],
/*BNE*/[  INVL,  INVL,  INVL,  INVL, INVL,  INVL,  INVL,  INVL,  INVL,  INVL,  0xD0, INVL, INVL],
/*BPL*/[  INVL,  INVL,  INVL,  INVL, INVL,  INVL,  INVL,  INVL,  INVL,  INVL,  0x10, INVL, INVL],
/*BRK*/[  INVL,  INVL,  INVL,  INVL, INVL,  INVL,  INVL,  INVL,  INVL,  INVL,  INVL, INVL, 0x00],
/*BVC*/[  INVL,  INVL,  INVL,  INVL, INVL,  INVL,  INVL,  INVL,  INVL,  INVL,  0x50, INVL, INVL],
/*BVS*/[  INVL,  INVL,  INVL,  INVL, INVL,  INVL,  INVL,  INVL,  INVL,  INVL,  0x70, INVL, INVL],
/*CLC*/[  INVL,  INVL,  INVL,  INVL, INVL,  INVL,  INVL,  INVL,  INVL,  INVL,  INVL, INVL, 0x18],
/*CLD*/[  INVL,  INVL,  INVL,  INVL, INVL,  INVL,  INVL,  INVL,  INVL,  INVL,  INVL, INVL, 0xD8],
/*CLI*/[  INVL,  INVL,  INVL,  INVL, INVL,  INVL,  INVL,  INVL,  INVL,  INVL,  INVL, INVL, 0x58],
/*CLV*/[  INVL,  INVL,  INVL,  INVL, INVL,  INVL,  INVL,  INVL,  INVL,  INVL,  INVL, INVL, 0xB8],
/*CMP*/[  0xC9,  0xC5,  0xD5,  INVL, 0xCD,  0xDD,  0xD9,  0xC1,  0xD1,  INVL,  INVL, INVL, INVL],
/*CPX*/[  0xE0,  0xE4,  INVL,  INVL, 0xEC,  INVL,  INVL,  INVL,  INVL,  INVL,  INVL, INVL, INVL],
/*CPY*/[  0xC0,  0xC4,  INVL,  INVL, 0xCC,  INVL,  INVL,  INVL,  INVL,  INVL,  INVL, INVL, INVL],
/*DEC*/[  INVL,  0xC6,  0xD6,  INVL, 0xCE,  0xDE,  INVL,  INVL,  INVL,  INVL,  INVL, INVL, INVL],
/*DEX*/[  INVL,  INVL,  INVL,  INVL, INVL,  INVL,  INVL,  INVL,  INVL,  INVL,  INVL, INVL, 0xCA],
/*DEY*/[  INVL,  INVL,  INVL,  INVL, INVL,  INVL,  INVL,  INVL,  INVL,  INVL,  INVL, INVL, 0x88],
/*EOR*/[  0x49,  0x45,  0x55,  INVL, 0x4D,  0x5D,  0x59,  0x41,  0x51,  INVL,  INVL, INVL, INVL],
/*INC*/[  INVL,  0xE6,  0xF6,  INVL, 0xEE,  0xFE,  INVL,  INVL,  INVL,  INVL,  INVL, INVL, INVL],
/*INX*/[  INVL,  INVL,  INVL,  INVL, INVL,  INVL,  INVL,  INVL,  INVL,  INVL,  INVL, INVL, 0xE8],
/*INY*/[  INVL,  INVL,  INVL,  INVL, INVL,  INVL,  INVL,  INVL,  INVL,  INVL,  INVL, INVL, 0xC8],
/*JMP*/[  INVL,  INVL,  INVL,  INVL, 0x4C,  INVL,  INVL,  INVL,  INVL,  INVL,  INVL, 0x6C, INVL],
/*JSR*/[  INVL,  INVL,  INVL,  INVL, 0x20,  INVL,  INVL,  INVL,  INVL,  INVL,  INVL, INVL, INVL],
/*LDA*/[  0xA9,  0xA5,  0xB5,  INVL, 0xAD,  0xBD,  0xB9,  0xA1,  0xB1,  INVL,  INVL, INVL, INVL],
/*LDX*/[  0xA2,  0xA6,  INVL,  0xB6, 0xAE,  INVL,  0xBE,  INVL,  INVL,  INVL,  INVL, INVL, INVL],
/*LDY*/[  0xA0,  0xA4,  0xB4,  INVL, 0xAC,  0xBC,  INVL,  INVL,  INVL,  INVL,  INVL, INVL, INVL],
/*LSR*/[  INVL,  0x46,  0x56,  INVL, 0x4E,  0x5E,  INVL,  INVL,  INVL,  0x4A,  INVL, INVL, INVL],
/*NOP*/[  INVL,  INVL,  INVL,  INVL, INVL,  INVL,  INVL,  INVL,  INVL,  INVL,  INVL, INVL, 0xEA],
/*ORA*/[  0x09,  0x05,  0x15,  INVL, 0x0D,  0x1D,  0x19,  0x01,  0x11,  INVL,  INVL, INVL, INVL],
/*PHA*/[  INVL,  INVL,  INVL,  INVL, INVL,  INVL,  INVL,  INVL,  INVL,  INVL,  INVL, INVL, 0x48],
/*PHP*/[  INVL,  INVL,  INVL,  INVL, INVL,  INVL,  INVL,  INVL,  INVL,  INVL,  INVL, INVL, 0x08],
/*PLA*/[  INVL,  INVL,  INVL,  INVL, INVL,  INVL,  INVL,  INVL,  INVL,  INVL,  INVL, INVL, 0x68],
/*PLP*/[  INVL,  INVL,  INVL,  INVL, INVL,  INVL,  INVL,  INVL,  INVL,  INVL,  INVL, INVL, 0x28],
/*ROL*/[  INVL,  0x26,  0x36,  INVL, 0x2E,  0x3E,  INVL,  INVL,  INVL,  0x2A,  INVL, INVL, INVL],
/*ROR*/[  INVL,  0x66,  0x76,  INVL, 0x6E,  0x7E,  INVL,  INVL,  INVL,  0x6A,  INVL, INVL, INVL],
/*RTI*/[  INVL,  INVL,  INVL,  INVL, INVL,  INVL,  INVL,  INVL,  INVL,  INVL,  INVL, INVL, 0x40],
/*RTS*/[  INVL,  INVL,  INVL,  INVL, INVL,  INVL,  INVL,  INVL,  INVL,  INVL,  INVL, INVL, 0x60],
/*SBC*/[  0xE9,  0xE5,  0xF5,  INVL, 0xED,  0xFD,  0xF9,  0xE1,  0xF1,  INVL,  INVL, INVL, INVL],
/*SEC*/[  INVL,  INVL,  INVL,  INVL, INVL,  INVL,  INVL,  INVL,  INVL,  INVL,  INVL, INVL, 0x38],
/*SED*/[  INVL,  INVL,  INVL,  INVL, INVL,  INVL,  INVL,  INVL,  INVL,  INVL,  INVL, INVL, 0xF8],
/*SEI*/[  INVL,  INVL,  INVL,  INVL, INVL,  INVL,  INVL,  INVL,  INVL,  INVL,  INVL, INVL, 0x78],
/*STA*/[  INVL,  0x85,  0x95,  INVL, 0x8D,  0x9D,  0x99,  0x81,  0x91,  INVL,  INVL, INVL, INVL],
/*STX*/[  INVL,  0x86,  INVL,  0x96, 0x8E,  INVL,  INVL,  INVL,  INVL,  INVL,  INVL, INVL, INVL],
/*STY*/[  INVL,  0x84,  0x94,  INVL, 0x8C,  INVL,  INVL,  INVL,  INVL,  INVL,  INVL, INVL, INVL],
/*TAX*/[  INVL,  INVL,  INVL,  INVL, INVL,  INVL,  INVL,  INVL,  INVL,  INVL,  INVL, INVL, 0xAA],
/*TAY*/[  INVL,  INVL,  INVL,  INVL, INVL,  INVL,  INVL,  INVL,  INVL,  INVL,  INVL, INVL, 0xA8],
/*TSX*/[  INVL,  INVL,  INVL,  INVL, INVL,  INVL,  INVL,  INVL,  INVL,  INVL,  INVL, INVL, 0xBA],
/*TXA*/[  INVL,  INVL,  INVL,  INVL, INVL,  INVL,  INVL,  INVL,  INVL,  INVL,  INVL, INVL, 0x8A],
/*TXS*/[  INVL,  INVL,  INVL,  INVL, INVL,  INVL,  INVL,  INVL,  INVL,  INVL,  INVL, INVL, 0x9A],
/*TYA*/[  INVL,  INVL,  INVL,  INVL, INVL,  INVL,  INVL,  INVL,  INVL,  INVL,  INVL, INVL, 0x98],
       ]// IMM    ZP    ZP_X   ZP_Y,  ABS   ABS_X  ABS_Y  IND_X  IND_Y   ACC    REL   IND, IMPL
    ;

// zero page addresses
// TODO: Do we really have to use
// zero page for indirect function calls
// or derefs?
const ZP_DEREF_0:       u8 = 0;
const ZP_DEREF_1:       u8 = 1;
const ZP_DEREF_STORE_0: u8 = 2;
const ZP_DEREF_STORE_1: u8 = 3;
const ZP_RHS_L:         u8 = 4;
const ZP_RHS_H:         u8 = 5;
const ZP_TMP_0:         u8 = 6;
const ZP_TMP_1:         u8 = 7;
const ZP_TMP_2:         u8 = 8;
const ZP_TMP_3:         u8 = 9;
const ZP_TMP_4:         u8 = 10;
const ZP_TMP_5:         u8 = 11;
const ZP_DEREF_FUN_0:   u8 = 12; // can't be the same as ZP_DEREF,
const ZP_DEREF_FUN_1:   u8 = 13; // as we use this before argument loading

const STACK_PAGE: u16 = 0x0100;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Byte {
    Low,
    High,
    Both
}

#[derive(Clone, Copy)]
pub enum RelocationKind {
    Address {
        idx: usize,
        relative: bool,
    }, // address from Assembler.addresses
    DataOffset {
        off: u16,
        byte: Byte,
    },
    External {
        name: *const c_char,
        offset: usize,
        byte: Byte,
        relative: bool,
    },
    Label {
        func_name: *const c_char,
        label: usize
    },
}
impl RelocationKind {
    pub fn is16(self) -> bool {
        match self {
            RelocationKind::DataOffset{byte, ..}  => byte == Byte::Both,
            RelocationKind::External{byte, relative, ..} => byte == Byte::Both && !relative,
            RelocationKind::Label{..}             => true,
            RelocationKind::Address{relative, ..} => !relative,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Relocation {
    pub kind: RelocationKind,
    pub addr: u16,
}

#[derive(Clone, Copy)]
pub struct Label {
    pub func_name: *const c_char,
    pub label: usize,
    pub addr: u16,
}

#[derive(Clone, Copy)]
pub struct External {
    pub name: *const c_char,
    pub addr: u16,
    pub loc: Loc,
}

#[must_use]
pub unsafe fn add_external(name: *const c_char, addr: u16, loc: Loc, asm: *mut Assembler, p: *const Program) -> Option<()> {
    for i in 0..(*asm).externals.count {
        let ext = *(*asm).externals.items.add(i);
        if strcmp(ext.name, name) == 0 {
            diagf!(loc,     c!("ERROR: redefinition of name `%s`\n"), name);
            diagf!(ext.loc, c!("INFO: previously defined here\n"));
            bump_error_count((*p).error_count)?;
        }
    }

    da_append(&mut (*asm).externals, External {
        name, addr, loc
    });

    Some(())
}

#[derive(Clone, Copy)]
pub struct Assembler {
    pub relocs: Array<Relocation>,
    pub op_labels: Array<Label>,
    pub externals: Array<External>,
    pub addresses: Array<u16>,
    pub code_start: u16, // load address of code section
    pub frame_sz: u8, // current stack frame size in bytes, because 6502 has no base register
    pub string_arena: Arena, // used for inline assembly labels
}

pub unsafe fn write_byte(out: *mut String_Builder, byte: u8) {
    da_append(out, byte as c_char);
}
pub unsafe fn write_word(out: *mut String_Builder, word: u16) {
    write_byte(out, word as u8);
    write_byte(out, (word >> 8) as u8);
}
pub unsafe fn write_byte_at(out: *mut String_Builder, byte: u8, addr: u16) {
    *((*out).items.add(addr as usize)) = byte as c_char;
}
pub unsafe fn write_word_at(out: *mut String_Builder, word: u16, addr: u16) {
    write_byte_at(out, word as u8, addr);
    write_byte_at(out, (word>>8) as u8, addr+1);
}

#[must_use]
pub unsafe fn instr0(out: *mut String_Builder, inst: Instr, mode: AddrMode, p: *const Program) -> Option<()> {
    let opcode = OPCODES[inst as usize][mode as usize];
    if opcode == INVL {
        log(Log_Level::ERROR, c!("6502: Invalid combination of opcode and operand %u and %u"), inst as usize, mode as usize);
        bump_error_count((*p).error_count)?;
    }
    write_byte(out, opcode);
    Some(())
}
// IMPL (implied) addressing mode
#[must_use]
pub unsafe fn instr(out: *mut String_Builder, inst: Instr, p: *const Program) -> Option<()> {
    instr0(out, inst, IMPL, p)
}
#[must_use]
pub unsafe fn instr8(out: *mut String_Builder, inst: Instr, mode: AddrMode, v: u8, p: *const Program) -> Option<()> {
    instr0(out, inst, mode, p)?;
    write_byte(out, v);
    Some(())
}
#[must_use]
pub unsafe fn instr16(out: *mut String_Builder, inst: Instr, mode: AddrMode, v: u16, p: *const Program) -> Option<()> {
    instr0(out, inst, mode, p)?;
    write_word(out, v);
    Some(())
}

pub unsafe fn add_reloc(out: *mut String_Builder, kind: RelocationKind, asm: *mut Assembler) {
    da_append(&mut (*asm).relocs, Relocation {
        kind,
        addr: (*out).count as u16
    });
    if kind.is16() {
        write_word(out, 0);
    } else {
        write_byte(out, 0);
    }
}

pub unsafe fn create_address_label(asm: *mut Assembler) -> usize {
    let idx = (*asm).addresses.count;
    da_append(&mut (*asm).addresses, 0);
    idx
}
pub unsafe fn create_address_label_here(out: *const String_Builder, asm: *mut Assembler) -> usize {
    let label = create_address_label(asm);
    link_address_label_here(label, out, asm);
    label
}

// TODO: inform the caller, that `addr' is relative to code_start
pub unsafe fn link_address_label(label: usize, addr: u16, asm: *mut Assembler) {
    *(*asm).addresses.items.add(label) = addr;
}
pub unsafe fn link_address_label_here(label: usize, out: *const String_Builder, asm: *mut Assembler) {
    *(*asm).addresses.items.add(label) = (*out).count as u16;
}

#[must_use]
pub unsafe fn load_auto_var(out: *mut String_Builder, index: usize, asm: *mut Assembler, p: *const Program) -> Option<()> {
    // save current stack pointer
    instr(out, TSX, p)?;
    // load low byte
    instr16(out, LDA, ABS_X, STACK_PAGE + (*asm).frame_sz as u16 - (index-1) as u16 * 2 - 1, p)?;
    // load high byte
    instr16(out, LDY, ABS_X, STACK_PAGE + (*asm).frame_sz as u16 - (index-1) as u16 * 2, p)?;
    Some(())
}

#[must_use]
pub unsafe fn load_auto_var_ref(out: *mut String_Builder, index: usize, asm: *mut Assembler, p: *const Program) -> Option<()> {
    // save current stack pointer
    instr(out, TSX, p)?;
    instr(out, TXA, p)?;
    instr(out, CLC, p)?;
    instr8(out, ADC, IMM, (*asm).frame_sz as u8 - (index-1) as u8 * 2 - 1, p)?;
    instr8(out, LDY, IMM, (STACK_PAGE >> 8) as u8, p)?;
    Some(())
}

#[must_use]
pub unsafe fn load_arg(arg: Arg, loc: Loc, out: *mut String_Builder, asm: *mut Assembler, p: *const Program) -> Option<()> {
    match arg {
        Arg::Deref(index) => {
            load_auto_var(out, index, asm, p)?;

            // load address to buffer in ZP to dereference, because registers
            // only 8 bits
            instr8(out, STA, ZP, ZP_DEREF_0, p)?;
            instr8(out, STY, ZP, ZP_DEREF_1, p)?;

            // Y = ((0),1)
            instr8(out, LDY, IMM, 1, p)?;
            instr8(out, LDA, IND_Y, ZP_DEREF_0, p)?;
            instr(out, TAY, p)?;

            // A = ((0,0))
            instr8(out, LDX, IMM, 0, p)?;
            instr8(out, LDA, IND_X, ZP_DEREF_0, p)?;
        },
        Arg::RefExternal(name) => {
            instr0(out, LDA, IMM, p)?;
            add_reloc(out, RelocationKind::External {name, offset: 0, byte: Byte::Low, relative: false}, asm);
            instr0(out, LDY, IMM, p)?;
            add_reloc(out, RelocationKind::External {name, offset: 0, byte: Byte::High, relative: false}, asm);
        },
        Arg::External(name) => {
            instr0(out, LDA, ABS, p)?;
            add_reloc(out, RelocationKind::External {name, offset: 0, byte: Byte::Both, relative: false}, asm);
            instr0(out, LDY, ABS, p)?;
            add_reloc(out, RelocationKind::External {name, offset: 1, byte: Byte::Both, relative: false}, asm);
        },
        Arg::AutoVar(index) => load_auto_var(out, index, asm, p)?,
        Arg::RefAutoVar(index) => load_auto_var_ref(out, index, asm, p)?,
        Arg::IntLiteral(int_literal, radix) => {
            let value: u16;
            if let Ok(v) = parse_int_literal_to_u16(int_literal, radix) {
                value = v;
            } else {
                diagf!(loc, c!("ERROR: mos6502: constant %s out of range for 16 bits\n"), int_literal);
                value = bump_error_count((*p).error_count).map(|()| 0)?;
            }
            instr8(out, LDA, IMM, value as u8, p)?;
            instr8(out, LDY, IMM, (value >> 8) as u8, p)?;
        }
        Arg::CharLiteral(char_literal, count) => {
            let value: u16;
            if let Ok(v) = parse_char_literal_to_u16_le(char_literal, count) {
                value = v;
            } else {
                diagf!(loc, c!("ERROR: mos6502: Character constant '%s' out of range for 16 bits\n"), char_literal);
                value = bump_error_count((*p).error_count).map(|()| 0)?;
            }
            instr8(out, LDA, IMM, value as u8, p)?;
            instr8(out, LDY, IMM, (value >> 8) as u8, p)?;
        }
        Arg::DataOffset(offset) => {
            assert!(offset < 65536, "data offset out of range");
            instr0(out, LDA, IMM, p)?;
            add_reloc(out, RelocationKind::DataOffset{off: offset as u16, byte: Byte::Low}, asm);
            instr0(out, LDY, IMM, p)?;
            add_reloc(out, RelocationKind::DataOffset{off: offset as u16, byte: Byte::High}, asm);
        },
        Arg::Bogus => unreachable!("bogus-amogus"),
    };
    Some(())
}

#[must_use]
pub unsafe fn store_auto(out: *mut String_Builder, index: usize, asm: *mut Assembler, p: *const Program) -> Option<()> {
    // save current stack pointer
    instr(out, TSX, p)?;
    // save low byte
    instr16(out, STA, ABS_X, STACK_PAGE + (*asm).frame_sz as u16 - (index-1) as u16 * 2 - 1, p)?;

    // save high byte
    instr(out, TYA, p)?;
    instr16(out, STA, ABS_X, STACK_PAGE + (*asm).frame_sz as u16 - (index-1) as u16 * 2, p)?;
    Some(())
}

#[must_use]
// TODO: can this be done better?
pub unsafe fn add_sp(out: *mut String_Builder, bytes: u8, asm: *mut Assembler, p: *const Program) -> Option<()> {
    (*asm).frame_sz -= bytes;
    if bytes < 8 {
        for _ in 0 .. bytes {
            instr(out, PLA, p)?;
        }
    } else {
        instr(out, TSX, p)?;
        instr(out, TXA, p)?;
        instr(out, CLC, p)?;
        instr8(out, ADC, IMM, bytes, p)?;
        instr(out, TAX, p)?;
        instr(out, TXS, p)?;
    }
    Some(())
}

#[must_use]
// cannot modify Y:A here, as they hold first argument
// TODO: look, if this can be done without a loop, like in `add_sp` without modifying
// Y:A. Either save them temporarily or write the first arg to stack before decrementing
// SP
pub unsafe fn sub_sp(out: *mut String_Builder, bytes: u8, asm: *mut Assembler, p: *const Program) -> Option<()> {
    (*asm).frame_sz += bytes;
    for _ in 0 .. bytes {
        instr(out, PHA, p)?;
    }
    Some(())
}

#[must_use]
pub unsafe fn push16(out: *mut String_Builder, asm: *mut Assembler, p: *const Program) -> Option<()> {
    (*asm).frame_sz += 2;

    instr(out, TAX, p)?;
    instr(out, TYA, p)?;
    // push high byte first
    instr(out, PHA, p)?;
    instr(out, TXA, p)?;
    // then low
    instr(out, PHA, p)?;
    Some(())
}

#[must_use]
pub unsafe fn pop16_discard(out: *mut String_Builder, asm: *mut Assembler, p: *const Program) -> Option<()> {
    (*asm).frame_sz -= 2;

    instr(out, PLA, p)?;
    instr(out, PLA, p)?;
    Some(())
}

#[must_use]
// load lhs in Y:A, rhs in RHS_L:RHS_H
pub unsafe fn load_two_args(out: *mut String_Builder, lhs: Arg, rhs: Arg, op: OpWithLocation, asm: *mut Assembler, p: *const Program) -> Option<()> {
    load_arg(rhs, op.loc, out, asm, p)?;
    instr8(out, STA, ZP, ZP_RHS_L, p)?;
    instr8(out, STY, ZP, ZP_RHS_H, p)?;
    load_arg(lhs, op.loc, out, asm, p)?;
    Some(())
}

#[derive(Clone, Copy)]
#[repr(C)]
pub enum Address {
    Literal(u16),
    Label(*const c_char),
}


pub unsafe fn parse_num(line_begin: *const c_char, mut line: *const c_char, mut loc: Loc, p: *const Program) -> Option<(u16, *const c_char)> {
    while isspace(*line as i32) != 0 {line = line.add(1);}

    let (v, mut end) = match *line as u8 {
        b'$' => {
            let mut end = ptr::null_mut();
            let v = strtoull(line.add(1), &mut end, 16);
            (v, end as *const c_char)
        }
        b'0'..=b'9' => {
            let mut end = ptr::null_mut();
            let v = strtoull(line, &mut end, 10);
            (v, end as *const c_char)
        },
        c => {
            loc.line_offset += (line as isize - line_begin as isize + 1) as i32;
            diagf!(loc, c!("ERROR: unexpected character `%c` in numberic literal\n"),
                   c as c_int);
            bump_error_count((*p).error_count).map(|()| (0 as c_ulonglong, line.wrapping_add(1)))?
        }
    };
    if v > 0xFFFF {
        loc.line_offset += (line as isize - line_begin as isize + 1) as i32;
        diagf!(loc, c!("ERROR: contant $%X out of range for 16 bits\n"), v);
        bump_error_count((*p).error_count)?;
    }
    while isspace(*end as i32) != 0 {end = end.add(1);}
    Some((v as u16, end))
}

pub unsafe fn parse_addr_or_label(line_begin: *const c_char, mut line: *const c_char, loc: Loc,
                                  asm: *mut Assembler, p: *const Program) -> Option<(Address, *const c_char)> {
    while isspace(*line as i32) != 0 {line = line.add(1);}

    let (v, mut end) = match *line {
        c if is_identifier_start(c) => {
            let start = line;
            while is_identifier(*line) {line = line.add(1);}
            let len = line as isize - start as isize;

            let label = arena::sprintf(&mut (*asm).string_arena, c!("%.*s"), len, start);
            (Address::Label(label), line)
        },
        _ => {
            let (v, line) = parse_num(line_begin, line, loc, p)?;
            (Address::Literal(v), line)
        }
    };
    while isspace(*end as i32) != 0 {end = end.add(1);}
    Some((v, end))
}

#[must_use]
pub unsafe fn assemble_statement(out: *mut String_Builder,
                                 mut line: *const c_char, mut loc: Loc,
                                 asm: *mut Assembler, p: *const Program) -> Option<()> {

    let line_begin = line;
    // TODO: IMPORTANT! What we are doing in here is basically lexing.
    // Consider maybe reusing and adapting our B lexer in here?
    while isspace(*line as i32) != 0 {
        line = line.add(1);
    }

    let inst_start = line;
    while *line != 0 && isspace(*line as i32) == 0 {
        line = line.add(1);
    }
    let len = line as usize - inst_start as usize;
    let name = arena::sprintf(&mut (*asm).string_arena, c!("%.*s"), len, inst_start);

    if len > 0 && *name.add(len-1) as u8 == b':' {
        *name.add(len-1) = 0;
        let label_addr = (*out).count as u16;
        let mut lloc = loc;
        lloc.line_offset += (line as isize - line_begin as isize + 1) as i32;

        add_external(name, label_addr, lloc, asm, p)?;

        if *line != 0 {
            diagf!(lloc, c!("ERROR: trailing garbage after label: `%s`\n"), line);
            bump_error_count((*p).error_count)?;
        }
        return Some(());
    }

    for i in 0..len {
        *name.add(i) = toupper(*name.add(i) as i32) as c_char;
    }
    let instr = match instr_from_string(name) {
        Some(v) => v,
        None => {
            loc.line_offset += (line as isize - line_begin as isize + 1) as i32;
            diagf!(loc, c!("ERROR: invalid instruction mnemonic `%s`\n"), name);
            bump_error_count((*p).error_count).map(|()| Instr::ADC)?
        }
    };

    while isspace(*line as i32) != 0 {line = line.add(1);}

    let operand = line;
    let mut arg8 = None;
    let mut arg16 = None;
    let mut arg_label = None;

    let mut mode = match *line as u8 {
        0    => IMPL,
        b'*' => {
            line = line.add(1);

            let rel = match *line as u8 {
                b'+' | b'-' => {
                    let mut end = ptr::null_mut();
                    let num = strtoull(line, &mut end, 10) as u16 as i16;
                    line = end;
                    num as i8
                },
                _ => 0
            };
            arg8 = Some((rel - 2) as u8);
            REL
        },
        b'#' => {
            line = line.add(1);

            let num;
            (num, line) = parse_num(line_begin, line, loc, p)?;
            if num > 0xFF {
                loc.line_offset += (line as isize - line_begin as isize + 1) as i32;
                diagf!(loc, c!("ERROR: constant $%X out of range for 8 bit immediate\n"),
                       num as c_uint);
                bump_error_count((*p).error_count)?;
            }
            arg8 = Some(num as u8);
            IMM
        },
        b'(' => {
            line = line.add(1);
            let addr;
            (addr, line) = parse_addr_or_label(line_begin, line, loc, asm, p)?;

            if *line as u8 == b',' {
                let num = match addr {
                    Address::Literal(l) => l,
                    Address::Label(_) => {
                        loc.line_offset += (line as isize - line_begin as isize + 1) as i32;
                        diagf!(loc, c!("ERROR: cannot use 16-bit label address for X-inderect addressing\n"));
                        bump_error_count((*p).error_count).map(|()| 0)?
                    },
                };

                if num > 0xFF {
                    loc.line_offset += (line as isize - line_begin as isize + 1) as i32;
                    diagf!(loc, c!("ERROR: constant $%X out of 8-bit range for indirect X\n"),
                           num as c_uint);
                    bump_error_count((*p).error_count)?;
                }
                arg8 = Some(num as u8);

                line = line.add(1);
                while isspace(*line as i32) != 0 {line = line.add(1);}
                if toupper(*line as i32) as u8 != b'X' {
                    loc.line_offset += (line as isize - line_begin as isize + 1) as i32;
                    diagf!(loc, c!("ERROR: X expected for indirect addressing mode\n"), *line as c_int);
                    bump_error_count((*p).error_count)?;
                }
                line = line.add(1);
                while isspace(*line as i32) != 0 {line = line.add(1);}
                if toupper(*line as i32) as u8 != b')' {
                    loc.line_offset += (line as isize - line_begin as isize + 1) as i32;
                    diagf!(loc, c!("ERROR: ) expected after X-indirect address\n"), *line as c_int);
                    bump_error_count((*p).error_count)?;
                }
                line = line.add(1);
                IND_X
            } else {
                if *line as u8 != b')' {
                    loc.line_offset += (line as isize - line_begin as isize + 1) as i32;
                    diagf!(loc, c!("ERROR: expected ',' or ')' after indirect address\n"), *line as c_int);
                    bump_error_count((*p).error_count)?;
                }
                line = line.add(1);
                while isspace(*line as i32) != 0 {line = line.add(1);}

                if *line as u8 == b',' {
                    let num = match addr {
                        Address::Literal(l) => l,
                        Address::Label(_) => {
                            loc.line_offset += (line as isize - line_begin as isize + 1) as i32;
                            diagf!(loc, c!("ERROR: cannot use 16-bit label address for Y-inderect addressing\n"));
                            bump_error_count((*p).error_count).map(|()| 0)?
                        },
                    };

                    if num > 0xFF {
                        loc.line_offset += (line as isize - line_begin as isize + 1) as i32;
                        diagf!(loc, c!("ERROR: constant $%X out of 8-bit range for indirect Y\n"),
                               num as c_uint);
                        bump_error_count((*p).error_count)?;
                    }
                    arg8 = Some(num as u8);

                    line = line.add(1);
                    while isspace(*line as i32) != 0 {line = line.add(1);}
                    if toupper(*line as i32) as u8 != b'Y' {
                        loc.line_offset += (line as isize - line_begin as isize + 1) as i32;
                        diagf!(loc, c!("ERROR: Y expected for Y-indirect addressing mode\n"), *line as c_int);
                        bump_error_count((*p).error_count)?;
                    }
                    line = line.add(1);
                    IND_Y
                } else {
                    match addr {
                        Address::Literal(l) => arg16 = Some(l),
                        Address::Label(s) => arg_label = Some(s),
                    }
                    IND
                }
            }
        },
        _  => {
            let addr;
            (addr, line) = parse_addr_or_label(line_begin, line, loc, asm, p)?;
            match addr {
                Address::Literal(l) => arg16 = Some(l),
                Address::Label(s) => arg_label = Some(s),
            }

            if *line as u8 == b',' {
                line = line.add(1);
                while isspace(*line as i32) != 0 {line = line.add(1);}

                if toupper(*line as i32) as u8 == b'X' {
                    line = line.add(1);
                    ABS_X
                } else if toupper(*line as i32) as u8 == b'Y' {
                    line = line.add(1);
                    ABS_Y
                } else {
                    ABS
                }
            } else {
                ABS
            }
        },
    };

    // prefer zeropage instructions, if they exist
    if let Some(v) = arg16 {
        if mode == ABS && v <= 0xFF && OPCODES[instr as usize][ZP as usize] != INVL {
            mode = ZP;
            arg8 = Some(v as u8);
            arg16 = None;
        } else if mode == ABS_X && v <= 0xFF && OPCODES[instr as usize][ZP_X as usize] != INVL {
            mode = ZP_X;
            arg8 = Some(v as u8);
            arg16 = None;
        } else if mode == ABS_Y && v <= 0xFF && OPCODES[instr as usize][ZP_Y as usize] != INVL {
            mode = ZP_Y;
            arg8 = Some(v as u8);
            arg16 = None;
        }
    }

    // labels for REL-only instructions should use REL
    if let Some(_) = arg_label {
        if mode == ABS && OPCODES[instr as usize][ABS as usize] == INVL &&
            OPCODES[instr as usize][REL as usize] != INVL {
            mode = REL;
        }
    }

    let opcode = OPCODES[instr as usize][mode as usize];
    if opcode == INVL {
        loc.line_offset += (line as isize - line_begin as isize + 1) as i32;
        diagf!(loc, c!("ERROR: invalid combination of instruction `%s` and operand `%s`\n"),
               name, operand);
        bump_error_count((*p).error_count)?;
    }

    write_byte(out, opcode);
    if let Some(a) = arg8 {
        write_byte(out, a);
    } else if let Some(a) = arg16 {
        write_word(out, a);
    } else if let Some(name) = arg_label {
        add_reloc(out, RelocationKind::External {name, offset: 0, byte: Byte::Both,
                                                 relative: mode == REL}, asm);
    }

    if *line != 0 {
        loc.line_offset += (line as isize - line_begin as isize + 1) as i32;
        diagf!(loc, c!("ERROR: trailing garbage: `%s`\n"), line);
        bump_error_count((*p).error_count)?;
    }
    Some(())
}

// repetitve code for emulating 16bit instructions
// TODO: most of these could probably be converted
// to intrinsic functions
mod ops {
    use super::*;
#[must_use]
    pub unsafe fn save_and_remove_signs(out: *mut String_Builder, asm: *mut Assembler, p: *const Program) -> Option<()> {
        let if0_end = create_address_label(asm);
        let if1_end = create_address_label(asm);
        // if (lhs < 0) {
        instr8(out, CPY, IMM, 0, p)?;
        instr0(out, BPL, REL, p)?;
        add_reloc(out, RelocationKind::Address{idx: if0_end, relative: true}, asm);

        // lhs = -lhs;
        instr8(out, LDA, IMM, 0, p)?;
        instr(out, SEC, p)?;
        instr8(out, SBC, ZP, ZP_TMP_0, p)?;
        instr8(out, STA, ZP, ZP_TMP_0, p)?;
        instr8(out, LDA, IMM, 0, p)?;
        instr8(out, SBC, ZP, ZP_TMP_1, p)?;
        instr8(out, STA, ZP, ZP_TMP_1, p)?;

        // tmp4 = 1;
        instr8(out, LDA, IMM, 1, p)?;
        instr8(out, STA, ZP, ZP_TMP_4, p)?;
        // }
        link_address_label_here(if0_end, out, asm);

        // if (rhs < 0) {
        instr8(out, CPY, IMM, 0, p)?;
        instr0(out, BPL, REL, p)?;
        add_reloc(out, RelocationKind::Address{idx: if1_end, relative: true}, asm);

        // lhs = -lhs;
        instr8(out, LDA, IMM,  0, p)?;
        instr(out, SEC, p)?;
        instr8(out, SBC, ZP, ZP_TMP_0, p)?;
        instr8(out, STA, ZP, ZP_TMP_0, p)?;
        instr8(out, LDA, IMM, 0, p)?;
        instr8(out, SBC, ZP, ZP_TMP_1, p)?;
        instr8(out, STA, ZP, ZP_TMP_1, p)?;

        // tmp4 ^= 1;
        instr8(out, LDA, ZP, ZP_TMP_4, p)?;
        instr8(out, EOR, IMM, 1, p)?;
        instr8(out, STA, ZP, ZP_TMP_4, p)?;
        // }
        link_address_label_here(if1_end, out, asm);
        Some(())
    }
}

pub unsafe fn generate_function(name: *const c_char, loc: Loc, params_count: usize, auto_vars_count: usize,
                                body: *const [OpWithLocation], out: *mut String_Builder,
                                asm: *mut Assembler, p: *const Program) -> Option<()> {
    (*asm).frame_sz = 0;
    let fun_addr = (*out).count as u16;
    add_external(name, fun_addr, loc, asm, p)?;

    // prepare function labels for each op and the end of the function
    let mut op_addresses: Array<usize> = zeroed();
    for _ in 0..=body.len() {
        let idx = (*asm).addresses.count;
        da_append(&mut op_addresses, idx);

        da_append(&mut (*asm).addresses, 0);
    }

    fprintf(stderr(), c!("FUNC: %s: auto_vars_count: %d\n"), name, auto_vars_count);
    // TODO: use params_count, auto_vars_count
    assert!(auto_vars_count*2 < 256);
    let stack_size = (auto_vars_count * 2) as u8;
    sub_sp(out, stack_size, asm, p)?;

    for i in 0..(params_count as u16) {
        instr(out, TSX, p)?;
        if i == 0 {
            // low
            instr16(out, STA, ABS_X, STACK_PAGE + stack_size as u16 - 2*i - 1, p)?;

            // high
            instr(out, TYA, p)?;
            instr16(out, STA, ABS_X, STACK_PAGE + stack_size as u16 - 2*i, p)?;
            continue;
        }

        // low
        instr16(out, LDA, ABS_X, STACK_PAGE + stack_size as u16 + 2*i + 1, p)?;
        instr16(out, STA, ABS_X, STACK_PAGE + stack_size as u16 - 2*i - 1, p)?;

        // high
        instr16(out, LDA, ABS_X, STACK_PAGE + stack_size as u16 + 2*i + 2, p)?;
        instr16(out, STA, ABS_X, STACK_PAGE + stack_size as u16 - 2*i, p)?;
    }

    for i in 0..body.len() {
        let addr_idx = *op_addresses.items.add(i);
        *(*asm).addresses.items.add(addr_idx) = (*out).count as u16; // update op address

        let op = (*body)[i];
        match op.opcode {
            Op::Bogus => unreachable!("bogus-amogus"),
            Op::Return {arg} => {
                if let Some(arg) = arg {
                    load_arg(arg, op.loc, out, asm, p)?;
                }

                // jump to ret statement
                instr0(out, JMP, ABS, p)?;
                add_reloc(out, RelocationKind::Address{idx: *op_addresses.items.add(body.len()),
                                                       relative: false}, asm);
            },
            Op::Store {index, arg} => {
                load_auto_var(out, index, asm, p)?;
                instr8(out, STA, ZP, ZP_DEREF_STORE_0, p)?;
                instr8(out, STY, ZP, ZP_DEREF_STORE_1, p)?;

                load_arg(arg, op.loc, out, asm, p)?;
                instr(out, TAX, p)?;
                instr(out, TYA, p)?;

                instr8(out, LDY, IMM, 1, p)?;
                instr8(out, STA, IND_Y, ZP_DEREF_STORE_0, p)?; // high
                instr(out, DEY, p)?;
                instr(out, TXA, p)?;
                instr8(out, STA, IND_Y, ZP_DEREF_STORE_0, p)?; // low
            },
            Op::ExternalAssign{name, arg} => {
                load_arg(arg, op.loc, out, asm, p)?;
                instr0(out, STA, ABS, p)?;
                add_reloc(out, RelocationKind::External {name, offset: 0, byte: Byte::Both, relative: false}, asm);
                instr0(out, STY, ABS, p)?;
                add_reloc(out, RelocationKind::External {name, offset: 1, byte: Byte::Both, relative: false}, asm);
            },
            Op::AutoAssign{index, arg} => {
                load_arg(arg, op.loc, out, asm, p)?;
                store_auto(out, index, asm, p)?;
            },
            Op::Negate {result, arg} => { // Y:A -> 0 - Y:A
                load_arg(arg, op.loc, out, asm, p)?;

                instr8(out, STA, ZP, ZP_TMP_0, p)?;
                instr8(out, STY, ZP, ZP_TMP_1, p)?;

                instr8(out, LDA, IMM, 0, p)?;
                instr(out, TAY, p)?;

                instr(out, SEC, p)?;
                instr8(out, SBC, ZP, ZP_TMP_0, p)?;
                instr(out, TAX, p)?;
                instr(out, TYA, p)?;
                instr8(out, SBC, ZP, ZP_TMP_1, p)?;
                instr(out, TAY, p)?;
                instr(out, TXA, p)?;

                store_auto(out, result, asm, p)?;
            },
            Op::UnaryNot{result, arg} => {
                load_arg(arg, op.loc, out, asm, p)?;

                instr8(out, LDX, IMM, 0, p)?;

                instr8(out, CMP, IMM, 0, p)?;
                instr8(out, BNE, REL, 5, p)?;

                instr(out, TYA, p)?;
                instr8(out, CMP, IMM, 0, p)?;
                instr8(out, BNE, REL, 1, p)?;

                instr(out, INX, p)?;

                instr(out, TXA, p)?;
                instr8(out, LDY, IMM, 0, p)?;

                store_auto(out, result, asm, p)?;
            },
            Op::Binop {binop, index, lhs, rhs} => {
                match binop {
                    Binop::BitOr => {
                        load_two_args(out, lhs, rhs, op, asm, p)?;

                        instr8(out, ORA, ZP, ZP_RHS_L, p)?;
                        instr(out, TAX, p)?;
                        instr(out, TYA, p)?;
                        instr8(out, ORA, ZP, ZP_RHS_H, p)?;
                        instr(out, TAY, p)?;
                        instr(out, TXA, p)?;
                    },
                    Binop::BitAnd => {
                        load_two_args(out, lhs, rhs, op, asm, p)?;

                        instr8(out, AND, ZP, ZP_RHS_L, p)?;
                        instr(out, TAX, p)?;
                        instr(out, TYA, p)?;
                        instr8(out, AND, ZP, ZP_RHS_H, p)?;
                        instr(out, TAY, p)?;
                        instr(out, TXA, p)?;
                    },
                    Binop::BitShl => {
                        load_two_args(out, lhs, rhs, op, asm, p)?;

                        instr8(out, STA, ZP, ZP_TMP_0, p)?;
                        instr8(out, STY, ZP, ZP_TMP_1, p)?;

                        // as maximum shift is 16, Y can be ignored.
                        // TODO: only shift 16 times if value > 16 provided
                        // TODO: do we have to handle negative shifts?
                        instr8(out, LDX, ZP, ZP_RHS_L, p)?;

                        let loop_start = create_address_label_here(out, asm);
                        instr8(out, BEQ, REL, 8, p)?;

                        instr8(out, ASL, ZP, ZP_TMP_0, p)?;
                        instr8(out, ROL, ZP, ZP_TMP_1, p)?;

                        instr(out, DEX, p)?;
                        instr0(out, JMP, ABS, p)?;
                        add_reloc(out, RelocationKind::Address{idx: loop_start, relative: false}, asm);

                        instr8(out, LDA, ZP, ZP_TMP_0, p)?;
                        instr8(out, LDY, ZP, ZP_TMP_1, p)?;
                    },
                    Binop::BitShr => {
                        load_two_args(out, lhs, rhs, op, asm, p)?;

                        instr8(out, STA, ZP, ZP_TMP_0, p)?;
                        instr8(out, STY, ZP, ZP_TMP_1, p)?;

                        // as maximum shift is 16, Y can be ignored.
                        // TODO: only shift 16 times if value > 16 provided
                        // TODO: do we have to handle negative shifts?
                        instr8(out, LDX, ZP, ZP_RHS_L, p)?;

                        let loop_start = create_address_label_here(out, asm);
                        instr8(out, BEQ, REL, 8, p)?;

                        instr8(out, LSR, ZP, ZP_TMP_1, p)?;
                        instr8(out, ROR, ZP, ZP_TMP_0, p)?;

                        instr(out, DEX, p)?;
                        instr0(out, JMP, ABS, p)?;
                        add_reloc(out, RelocationKind::Address{idx: loop_start, relative: false}, asm);
                        instr8(out, LDA, ZP, ZP_TMP_0, p)?;
                        instr8(out, LDY, ZP, ZP_TMP_1, p)?;
                    },
                    Binop::Plus => {
                        load_two_args(out, lhs, rhs, op, asm, p)?;

                        instr(out, CLC, p)?;
                        instr8(out, ADC, ZP, ZP_RHS_L, p)?;
                        instr(out, TAX, p)?;
                        instr(out, TYA, p)?;
                        instr8(out, ADC, ZP, ZP_RHS_H, p)?;
                        instr(out, TAY, p)?;
                        instr(out, TXA, p)?;
                    },
                    Binop::Minus  => {
                        load_two_args(out, lhs, rhs, op, asm, p)?;

                        instr(out, SEC, p)?;
                        instr8(out, SBC, ZP, ZP_RHS_L, p)?;
                        instr(out, TAX, p)?;
                        instr(out, TYA, p)?;
                        instr8(out, SBC, ZP, ZP_RHS_H, p)?;
                        instr(out, TAY, p)?;
                        instr(out, TXA, p)?;
                    },
                    Binop::Mod => {
                        // !! TODO !! this should be implemented here and not as a B functions.
                        // TODO: current mod implementation is linear, we can do better.
                        load_arg(rhs, op.loc, out, asm, p)?;
                        push16(out, asm, p)?;
                        load_arg(lhs, op.loc, out, asm, p)?;

                        instr0(out, JSR, ABS, p)?;
                        add_reloc(out, RelocationKind::External{name: c!("_rem"), offset: 0,
                                                                   byte: Byte::Both, relative: false}, asm);
                        instr(out, TAX, p)?;
                        pop16_discard(out, asm, p)?;
                        instr(out, TXA, p)?;
                    },
                    Binop::Div => {
                        // !! TODO !! this should be implemented here and not as a B functions.
                        // TODO: current div implementation is linear, we can do better.
                        load_arg(rhs, op.loc, out, asm, p)?;
                        push16(out, asm, p)?;
                        load_arg(lhs, op.loc, out, asm, p)?;

                        instr0(out, JSR, ABS, p)?;
                        add_reloc(out, RelocationKind::External{name: c!("_div"), offset: 0,
                                                                   byte: Byte::Both, relative: false}, asm);
                        instr(out, TAX, p)?;
                        pop16_discard(out, asm, p)?;
                        instr(out, TXA, p)?;
                    },
                    Binop::Mult => {
                        load_two_args(out, lhs, rhs, op, asm, p)?;

                        // TODO: maybe move this to an intrinsic function,
                        // because it is rather long. Consider this, if we run
                        // out of memory at some point.

                        // shift-and-add/long multiplication
                        // see: https://en.wikipedia.org/wiki/Multiplication_algorithm

                        // store lhs
                        instr8(out, STA, ZP, ZP_TMP_0, p)?;
                        instr8(out, STY, ZP, ZP_TMP_1, p)?;

                        // shift 16 times
                        instr8(out, LDA, IMM, 16, p)?;
                        instr8(out, STA, ZP, ZP_TMP_5, p)?;

                        ops::save_and_remove_signs(out, asm, p)?;

                        // from here on: unsigned multiplication
                        // store Y:A in ZP, because shifting and adding is easier
                        // without all the register switching
                        instr8(out, LDA, IMM, 0, p)?;
                        instr8(out, STA, ZP, ZP_TMP_2, p)?;
                        instr8(out, STA, ZP, ZP_TMP_3, p)?;

                        let loop_start = create_address_label_here(out, asm);
                        let cont = create_address_label(asm);
                        let finished = create_address_label(asm);

                        // if shifted 16 times, we are finished
                        instr8(out, LDA, ZP, ZP_TMP_5, p)?;
                        instr0(out, BNE, REL, p)?;
                        add_reloc(out, RelocationKind::Address{idx: cont, relative: true}, asm);

                        instr0(out, JMP, ABS, p)?;
                        add_reloc(out, RelocationKind::Address{idx: finished, relative: false}, asm);

                        link_address_label_here(cont, out, asm);

                        instr8(out, DEC, ZP, ZP_TMP_5, p)?;

                        // shift left current accumulater between single adds
                        instr8(out, ASL, ZP, ZP_TMP_2, p)?;
                        instr8(out, ROL, ZP, ZP_TMP_3, p)?;

                        instr8(out, ASL, ZP, ZP_RHS_L, p)?;
                        instr8(out, ROL, ZP, ZP_RHS_H, p)?;

                        // if bit is 0, do not add anything
                        instr0(out, BCC, REL, p)?;
                        add_reloc(out, RelocationKind::Address{idx: loop_start, relative: true}, asm);

                        // bit is 1 here, we have to add entire lhs to acc
                        instr(out, CLC, p)?;
                        instr8(out, LDA, ZP, ZP_TMP_2, p)?; // acc, low
                        instr8(out, ADC, ZP, ZP_TMP_0, p)?; // lhs, low
                        instr8(out, STA, ZP, ZP_TMP_2, p)?; // acc, low

                        instr8(out, LDA, ZP, ZP_TMP_3, p)?; // acc, high
                        instr8(out, ADC, ZP, ZP_TMP_1, p)?; // lhs, high
                        instr8(out, STA, ZP, ZP_TMP_3, p)?; // acc, high

                        // continue loop
                        instr0(out, JMP, ABS, p)?;
                        add_reloc(out, RelocationKind::Address{idx: loop_start, relative: false}, asm);
                        link_address_label_here(finished, out, asm);

                        // move back in Y:A
                        instr8(out, LDA, ZP, ZP_TMP_2, p)?;
                        instr8(out, LDY, ZP, ZP_TMP_3, p)?;

                        instr8(out, LDX, ZP, ZP_TMP_4, p)?;
                        // if (negative == 1) {
                        instr8(out, BEQ, REL, 12, p)?;

                        instr8(out, LDA, IMM, 0, p)?;
                        instr(out, TAY, p)?;

                        // Y:A = -Y:A
                        instr(out, SEC, p)?;
                        instr8(out, SBC, ZP, ZP_TMP_2, p)?;
                        instr(out, TAX, p)?;
                        instr(out, TYA, p)?;
                        instr8(out, SBC, ZP, ZP_TMP_3, p)?;
                        instr(out, TXA, p)?;
                        instr(out, TAY, p)?;
                        // }

                        // missingf!(op.loc, c!("implement Mult\n"))
                    },

                    // TODO: use same less code everywhere without duplication
                    Binop::Less => {
                        load_two_args(out, lhs, rhs, op, asm, p)?;
                        // we subtract, then check sign

                        instr8(out, LDX, IMM, 1, p)?;

                        instr(out, SEC, p)?; // set carry
                        // sub low byte
                        instr8(out, SBC, ZP, ZP_RHS_L, p)?;
                        // sub high byte
                        instr(out, TYA, p)?;
                        instr8(out, SBC, ZP, ZP_RHS_H, p)?;
                        // high result in A, N flag if less.

                        // if less skip, we already have X=1
                        instr8(out, BMI, REL, 1, p)?;
                        instr(out, DEX, p)?;
                        instr(out, TXA, p)?;
                        // zero extend result
                        instr8(out, LDY, IMM, 0, p)?;
                    },
                    Binop::Greater => { // A > B <=> B < A
                        load_two_args(out, rhs, lhs, op, asm, p)?;
                        // we subtract, then check sign

                        instr8(out, LDX, IMM, 1, p)?;

                        instr(out, SEC, p)?; // set carry
                        // sub low byte
                        instr8(out, SBC, ZP, ZP_RHS_L, p)?;
                        // sub high byte
                        instr(out, TYA, p)?;
                        instr8(out, SBC, ZP, ZP_RHS_H, p)?;
                        // high result in A, N flag if less.

                        // if less skip, we already have X=1
                        instr8(out, BMI, REL, 1, p)?;

                        instr(out, DEX, p)?;
                        instr(out, TXA, p)?;
                        // zero extend result
                        instr8(out, LDY, IMM, 0, p)?;
                    },
                    Binop::Equal => {
                        load_two_args(out, lhs, rhs, op, asm, p)?;

                        instr8(out, LDX, IMM, 0, p)?;

                        instr8(out, CMP, ZP, ZP_RHS_L, p)?;
                        instr8(out, BNE, REL, 5, p)?;

                        instr8(out, CPY, ZP, ZP_RHS_H, p)?;
                        instr8(out, BNE, REL, 1, p)?;

                        instr(out, INX, p)?;
                        instr(out, TXA, p)?;
                        instr8(out, LDY, IMM, 0, p)?;
                    },
                    Binop::NotEqual => {
                        load_two_args(out, lhs, rhs, op, asm, p)?;

                        instr8(out, LDX, IMM, 1, p)?;

                        instr8(out, CMP, ZP, ZP_RHS_L, p)?;
                        instr8(out, BNE, REL, 5, p)?;

                        instr8(out, CPY, ZP, ZP_RHS_H, p)?;
                        instr8(out, BNE, REL, 1, p)?;

                        instr(out, DEX, p)?;
                        instr(out, TXA, p)?;
                        instr8(out, LDY, IMM, 0, p)?;
                    },
                    Binop::GreaterEqual => { // A >= B <=> !(A < B)
                        load_two_args(out, lhs, rhs, op, asm, p)?;
                        // we subtract, then check sign

                        instr8(out, LDX, IMM, 0, p)?;

                        instr(out, SEC, p)?; // set carry
                        // sub low byte
                        instr8(out, SBC, ZP, ZP_RHS_L, p)?;
                        // sub high byte
                        instr(out, TYA, p)?;
                        instr8(out, SBC, ZP, ZP_RHS_H, p)?;
                        // high result in A, N flag if less.

                        // if less skip, we already have X=0
                        instr8(out, BMI, REL, 1, p)?;
                        instr(out, INX, p)?;
                        instr(out, TXA, p)?;
                        // zero extend result
                        instr8(out, LDY, IMM, 0, p)?;
                    },
                    Binop::LessEqual => { // X <= Y <=> Y >= X <=> !(Y < X)
                        load_two_args(out, rhs, lhs, op, asm, p)?;
                        // we subtract, then check sign

                        instr8(out, LDX, IMM, 0, p)?;

                        instr(out, SEC, p)?; // set carry
                        // sub low byte
                        instr8(out, SBC, ZP, ZP_RHS_L, p)?;
                        // sub high byte
                        instr(out, TYA, p)?;
                        instr8(out, SBC, ZP, ZP_RHS_H, p)?;
                        // high result in A, N flag if less.

                        // if greater skip, we already have X=0
                        instr8(out, BMI, REL, 1, p)?;
                        instr(out, INX, p)?;

                        instr(out, TXA, p)?;
                        // zero extend result
                        instr8(out, LDY, IMM, 0, p)?;
                    },
                }
                store_auto(out, index, asm, p)?;
            },
            Op::Funcall{result, fun, args} => {
                match fun {
                    Arg::RefExternal(_) | Arg::External(_)  | Arg::IntLiteral(_, _) | Arg::CharLiteral(_, _) => {},
                    arg => {
                        load_arg(arg, op.loc, out, asm, p)?;
                        instr8(out, STA, ZP, ZP_DEREF_FUN_0, p)?;
                        instr8(out, STY, ZP, ZP_DEREF_FUN_1, p)?;
                    }
                }

                for i in (0..args.count).rev() {
                    load_arg(*args.items.add(i), op.loc, out, asm, p)?;
                    // first arg in Y:A to be compatible with wozmon routines
                    if i != 0 {
                        push16(out, asm, p)?;
                    }
                }
                match fun {
                    Arg::RefExternal(name) | Arg::External(name) => {
                        instr0(out, JSR, ABS, p)?;
                        add_reloc(out, RelocationKind::External{name, offset: 0, byte: Byte::Both, relative: false}, asm);
                    },
                    Arg::IntLiteral(int_literal, radix) => {
                        let value: u16;
                        if let Ok(v) = parse_int_literal_to_u16(int_literal, radix) {
                            value = v;
                        } else {
                            diagf!(op.loc, c!("ERROR: mos6502: function address %s out of range for 16 bits\n"), int_literal);
                            value = bump_error_count((*p).error_count).map(|()| 0)?;
                        }
                        instr16(out, JSR, ABS, value, p)?;
                    },
                    Arg::CharLiteral(char_literal, count) => {
                        let value: u16;
                        if let Ok(v) = parse_char_literal_to_u16_le(char_literal, count) {
                            value = v;
                        } else {
                            diagf!(op.loc, c!("ERROR: mos6502: function address '%s' out of range for 16 bits\n"), char_literal);
                            value = bump_error_count((*p).error_count).map(|()| 0)?;
                        }
                        instr16(out, JSR, ABS, value, p)?;
                    },
                    _ => { // function pointer already loaded in ZP_DEREF_FUN
                        // there is no jsr (indirect), so emulate using jsr and jmp (indirect).
                        instr16(out, JSR, ABS, (*asm).code_start + (*out).count as u16 + 6, p)?;
                        instr16(out, JMP, ABS, (*asm).code_start + (*out).count as u16 + 6, p)?;
                        instr16(out, JMP, IND, ZP_DEREF_FUN_0 as u16, p)?;
                    },
                }
                if args.count > 1 {
                    instr(out, TAX, p)?;
                    // clear stack
                    for i in 0 .. args.count {
                        if i == 0 {
                            continue;
                        }
                        pop16_discard(out, asm, p)?;
                    }
                    instr(out, TXA, p)?;
                }
                store_auto(out, result, asm, p)?;
            },
            Op::Asm {stmts} => {
                for i in 0..stmts.count {
                    let stmt = *stmts.items.add(i);
                    assemble_statement(out, stmt.line, stmt.loc, asm, p)?;
                }
            },
            Op::Label{label} => {
                // RE: https://github.com/tsoding/b/pull/147#issue-3154667157
                // > For this thing I introduces a new NOP instruction because it would be a bit too
                // > risky to just blindly jump on an address that could possibly be unused.
                //
                // We now, this label will always be followed by an instruction: either the next
                // generated OP, or the return code at the end of a function. No NOP needed.
                da_append(&mut (*asm).op_labels, Label {
                    func_name: name,
                    label,
                    addr: (*out).count as u16,
                });
            },
            Op::JmpLabel{label} => {
                instr0(out, JMP, ABS, p)?;
                add_reloc(out, RelocationKind::Label{func_name: name, label}, asm);
            },
            Op::JmpIfNotLabel{label, arg} => {
                load_arg(arg, op.loc, out, asm, p)?;

                instr8(out, CMP, IMM, 0, p)?;

                // if !=0, skip next check and branch
                instr8(out, BNE, REL, 7, p)?; // skip next 4 instructions
                instr8(out, CPY, IMM, 0, p)?;
                instr8(out, BNE, REL, 3, p)?;

                instr0(out, JMP, ABS, p)?;
                add_reloc(out, RelocationKind::Label{func_name: name, label}, asm);
            },
            Op::Index {result, arg, offset} => {
                load_two_args(out, arg, offset, op, asm, p)?;

                // shift offset to the left by one bit
                instr8(out, ASL, ZP, ZP_RHS_L, p)?;
                instr8(out, ROL, ZP, ZP_RHS_H, p)?;

                // add offset and arg
                instr(out, CLC, p)?;
                instr8(out, ADC, ZP, ZP_RHS_L, p)?;
                instr(out, TAX, p)?;
                instr(out, TYA, p)?;
                instr8(out, ADC, ZP, ZP_RHS_H, p)?;
                instr(out, TAY, p)?;
                instr(out, TXA, p)?;

                store_auto(out, result, asm, p)?;
            },
        }
    }

    instr8(out, LDA, IMM, 0, p)?;
    instr(out, TAY, p)?;

    let addr_idx = *op_addresses.items.add(body.len());
    *(*asm).addresses.items.add(addr_idx) = (*out).count as u16;

    if stack_size > 0 {
        // seriously... we don't have enough registers to save A to...
        instr8(out, STA, ZP, ZP_TMP_0, p)?;
        add_sp(out, stack_size, asm, p)?;
        instr8(out, LDA, ZP, ZP_TMP_0, p)?;
    }
    instr(out, RTS, p)?;
    Some(())
}

pub unsafe fn generate_funcs(out: *mut String_Builder, funcs: *const [Func], asm: *mut Assembler, p: *const Program) {
    for i in 0..funcs.len() {
        generate_function((*funcs)[i].name, (*funcs)[i].name_loc, (*funcs)[i].params_count, (*funcs)[i].auto_vars_count, da_slice((*funcs)[i].body), out, asm, p);
    }
}

pub unsafe fn apply_relocations(out: *mut String_Builder, data_start: u16, asm: *mut Assembler) {
    'reloc_loop: for i in 0..(*asm).relocs.count {
        let reloc = *(*asm).relocs.items.add(i);
        let caddr = reloc.addr;
        match reloc.kind {
            RelocationKind::DataOffset{off, byte} => {
                let faddr = data_start + off;
                match byte {
                    Byte::Low  => write_byte_at(out, faddr as u8, caddr),
                    Byte::High => write_byte_at(out, (faddr >> 8) as u8, caddr),
                    Byte::Both => write_word_at(out, faddr, caddr),
                }
            },
            RelocationKind::Label{func_name: name, label} => {
                for i in 0..(*asm).op_labels.count {
                    let op_label = *(*asm).op_labels.items.add(i);
                    if strcmp(op_label.func_name, name) == 0 && op_label.label == label {
                        write_word_at(out, (*asm).code_start + op_label.addr, caddr);
                        continue 'reloc_loop;
                    }
                }
                log(Log_Level::ERROR, c!("6502: Linking failed. Could not find label `%s.%u'"), name, label);
                unreachable!();
            },
            RelocationKind::External{name, offset, byte, relative} => {
                for i in 0..(*asm).externals.count {
                    let label = *(*asm).externals.items.add(i);
                    if strcmp(label.name, name) == 0 {
                        let faddr = (*asm).code_start + label.addr + offset as u16;
                        if relative {
                            let rel = (faddr as i64) - ((caddr + 1) as i64);
                            write_byte_at(out, rel as i8 as u8, caddr);
                        } else {
                            match byte {
                                Byte::Low  => write_byte_at(out, faddr as u8, caddr),
                                Byte::High => write_byte_at(out, (faddr >> 8) as u8, caddr),
                                Byte::Both => write_word_at(out, faddr, caddr)
                            }
                        }
                        continue 'reloc_loop;
                    }
                }
                log(Log_Level::ERROR, c!("6502: Linking failed. Could not find extrn `%s'"), name);
            },
            RelocationKind::Address{idx, relative: true} => {
                let jaddr = *(*asm).addresses.items.add(idx);
                let rel: i16 = jaddr as i16 - (caddr + 1) as i16;
                assert!(rel < 128 && rel >= -128);
                write_byte_at(out, rel as u8, caddr);
            },
            RelocationKind::Address{idx, relative: false} => {
                let saddr = *(*asm).addresses.items.add(idx) + (*asm).code_start;
                write_word_at(out, saddr, caddr);
            },
        }
    }
}

#[must_use]
pub unsafe fn generate_extrns(_out: *mut String_Builder, extrns: *const [*const c_char],
                              funcs: *const [Func], globals: *const [Global],
                              asm_funcs: *const [AsmFunc], p: *const Program) -> Option<()> {
    'skip_function_or_global: for i in 0..extrns.len() {
        // assemble a few "stdlib" functions which can't be programmed in B
        let name = (*extrns)[i];
        for j in 0..funcs.len() {
            let func = (*funcs)[j].name;
            if strcmp(func, name) == 0 {
                continue 'skip_function_or_global
            }
        }
        for j in 0..globals.len() {
            let global = (*globals)[j].name;
            if strcmp(global, name) == 0 {
                continue 'skip_function_or_global
            }
        }
        for j in 0..asm_funcs.len() {
            let func = (*asm_funcs)[j].name;
            if strcmp(func, name) == 0 {
                continue 'skip_function_or_global
            }
        }

        log(Log_Level::ERROR, c!("6502: Unknown extrn: `%s`, can not link"), name);
        bump_error_count((*p).error_count)?;
    }
    Some(())
}

#[must_use]
pub unsafe fn generate_globals(out: *mut String_Builder, globals: *mut [Global], asm: *mut Assembler, p: *const Program) -> Option<()> {
    for i in 0..globals.len() {
        let global = (*globals)[i];
        add_external(global.name, (*out).count as u16, global.name_loc, asm, p)?;

        if global.is_vec {
            let address = create_address_label(asm);
            add_reloc(out, RelocationKind::Address{idx: address, relative: false}, asm);
            link_address_label_here(address, out, asm);
        }
        for j in 0..global.values.count {
            match *global.values.items.add(j) {
                ImmediateValue::IntLiteral(int_literal, radix) => {
                    let value: u16;
                    if let Ok(v) = parse_int_literal_to_u16(int_literal, radix) {
                        value = v;
                    } else {
                        let prefix = match radix {
                            Radix::Dec => c!(""),
                            Radix::Oct => c!("0"),
                            Radix::Hex => c!("0x"),
                            _ => unreachable!()
                        };
                        let fmt_str = temp_sprintf(c!("ERROR: mos6502: constant %s%%s out of range for 16 bits\n"), prefix);
                        diagf!(*global.value_locs.items.add(j), fmt_str, int_literal);
                        value = bump_error_count((*p).error_count).map(|()| 0)?;
                    }
                    write_word(out, value)
                }
                ImmediateValue::NegatedIntLiteral(int_literal, radix) => {
                    let value: u16;
                    if let Ok(v) = parse_int_literal_to_u16(int_literal, radix) {
                        value = !v + 1;
                    } else {
                        let prefix = match radix {
                            Radix::Dec => c!(""),
                            Radix::Oct => c!("0"),
                            Radix::Hex => c!("0x"),
                            _ => unreachable!()
                        };
                        let fmt_str = temp_sprintf(c!("ERROR: mos6502: constant %s%%s out of range for 16 bits\n"), prefix);
                        diagf!(*global.value_locs.items.add(j), fmt_str, int_literal);
                        value = bump_error_count((*p).error_count).map(|()| 0)?;
                    }
                    write_word(out, value)
                }
                ImmediateValue::CharLiteral(char_literal, count) => {
                    let value: u16;
                    if let Ok(v) = parse_char_literal_to_u16_le(char_literal, count) {
                        value = v;
                    } else {
                        diagf!(*global.value_locs.items.add(j), c!("ERROR: mos6502: char constant '%s' out of range for 16 bits\n"), char_literal);
                        value = bump_error_count((*p).error_count).map(|()| 0)?;
                    }
                    write_word(out, value)
                }
                ImmediateValue::Name(name) =>
                    add_reloc(out, RelocationKind::External{name, byte: Byte::Both, offset: 0, relative: false}, asm),
                ImmediateValue::DataOffset(offset) => {
                    add_reloc(out, RelocationKind::DataOffset{off: offset as u16, byte: Byte::Both}, asm);
                }
            }
        }

        for _ in global.values.count..global.minimum_size {
            write_word(out, 0);
        }
    }
    Some(())
}

pub unsafe fn generate_data_section(out: *mut String_Builder, data: *const [u8]) {
    for i in 0..data.len() {
        write_byte(out, (*data)[i]);
    }
}

#[must_use]
pub unsafe fn generate_entry(out: *mut String_Builder, asm: *mut Assembler, p: *const Program) -> Option<()> {
    instr0(out, JSR, ABS, p)?;
    add_reloc(out, RelocationKind::External{name: c!("main"), offset: 0, byte: Byte::Both, relative: false}, asm);

    instr16(out, JMP, IND, 0xFFFC, p)?;
    Some(())
}

#[must_use]
pub unsafe fn generate_asm_funcs(out: *mut String_Builder, asm_funcs: *const [AsmFunc],
                                 asm: *mut Assembler, p: *const Program) -> Option<()> {
    for i in 0..asm_funcs.len() {
        let asm_func = (*asm_funcs)[i];

        let fun_addr = (*out).count as u16;
        add_external(asm_func.name, fun_addr, asm_func.name_loc, asm, p)?;

        for j in 0..asm_func.body.count {
            let stmt = *asm_func.body.items.add(j);
            assemble_statement(out, stmt.line, stmt.loc, asm, p)?;
        }
    }
    Some(())
}

pub unsafe fn usage(params: *const [Param]) {
    fprintf(stderr(), c!("mos6502 codegen for the B compiler\n"));
    fprintf(stderr(), c!("OPTIONS:\n"));
    print_params_help(params);
}

struct Mos6502 {
    load_offset: u64,
    out: String_Builder,
    cmd: Cmd,
}

pub unsafe fn get_apis(targets: *mut Array<TargetAPI>) {
    da_append(targets, TargetAPI::V1 {
        name: c!("6502-posix"),
        file_ext: c!(".6502"),
        new,
        build: generate_program,
        run: run_program,
    });
}

pub unsafe fn new(a: *mut arena::Arena, args: *const [*const c_char]) -> Option<*mut c_void> {
    let gen = arena::alloc_type::<Mos6502>(a);
    memset(gen as _ , 0, size_of::<Mos6502>());

    let mut help = false;
    let params = &[
        Param {
            name:        c!("help"),
            description: c!("Print this help message"),
            value:       ParamValue::Flag { var: &mut help },
        },
        Param {
            name:        c!("LOAD_OFFSET"),
            description: c!("Offset at which the rom is expected to be loaded"),
            value:       ParamValue::Hex { var: &mut (*gen).load_offset, default: 0x8000 },
        },
    ];

    if let Err(message) = parse_args(params, args) {
        usage(params);
        log(Log_Level::ERROR, c!("%s"), message);
        return None;
    }

    if help {
        usage(params);
        return None;
    }

    Some(gen as *mut c_void)
}

pub unsafe fn generate_program(
    gen: *mut c_void, p: *const Program, program_path: *const c_char, _garbage_base: *const c_char,
    _nostdlib: bool, debug: bool, 
) -> Option<()> {
    let gen = gen as *mut Mos6502;
    let out = &mut (*gen).out;

    if debug { todo!("Debug information for 6502") }

    let mut asm: Assembler = zeroed();
    generate_entry(out, &mut asm, p)?;
    asm.code_start = (*gen).load_offset as u16;

    generate_funcs(out, da_slice((*p).funcs), &mut asm, p);
    generate_asm_funcs(out, da_slice((*p).asm_funcs), &mut asm, p)?;
    generate_extrns(out, da_slice((*p).extrns), da_slice((*p).funcs), da_slice((*p).globals), da_slice((*p).asm_funcs), p)?;

    let data_start = (*gen).load_offset as u16 + (*out).count as u16;
    generate_data_section(out, da_slice((*p).data));
    generate_globals(out, da_slice((*p).globals), &mut asm, p)?;

    log(Log_Level::INFO, c!("Generated size: 0x%x"), (*out).count as c_uint);
    apply_relocations(out, data_start, &mut asm);
    arena::reset(&mut asm.string_arena);

    write_entire_file(program_path, (*out).items as *const c_void, (*out).count)?;
    log(Log_Level::INFO, c!("generated %s"), program_path);

    Some(())
}

pub unsafe fn run_program(
    gen: *mut c_void, program_path: *const c_char, run_args: *const [*const c_char],
) -> Option<()> {
    let gen = gen as *mut Mos6502;
    let cmd = &mut (*gen).cmd;
    cmd_append!{
        cmd,
        c!("posix6502"), c!("-load-offset"), temp_sprintf(c!("%u"), (*gen).load_offset as c_uint),
        program_path
    }
    if run_args.len() > 0 {
        cmd_append!(cmd, c!("--"));
        da_append_many(cmd, run_args);
    }
    if !cmd_run_sync_and_reset(cmd) { return None; }
    Some(())
}
