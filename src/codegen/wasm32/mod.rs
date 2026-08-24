use core::ffi::*;
use core::mem::zeroed;
use core::slice;
use crate::arena;
use crate::codegen_common::{parse_int_literal_to_u32, parse_char_literal_to_u32};
use crate::crust::libc::*;
use crate::diagf;
use crate::errors::bump_error_count;
use crate::ir::*;
use crate::nob::*;
use crate::params::*;
use crate::targets::TargetAPI;

// Used this doc to get bin format description:
// https://github.com/WebAssembly/spec/blob/main/specification/wasm-1.0/A-binary.spectec
// this spec is wrong about saying that module version after magic number is `1:Bu32`, because it
// stored as LE number and not as LEB128
// To understand wasm I used: 
// https://developer.mozilla.org/en-US/docs/WebAssembly
// And of course some small llm searches (but mostly for general stuff, and to make wat example
// that uses all wasm file sections to compare with)
// I hope I will not use it to write real code. It is still a hobby codding after all.
// And based this codegen mostly on uxn and mos6502 codegen code

// TODO: Remove this idea dump after implementing codegen
// Main questions for wasm codegen are:
// - How could we have all functions with varargs:
//   - well, we don't have them even in x86 right now, only if nargs>nregisters
//   - fake stack till we try to optimize stuff
//   - we only be able to use extrn for wrapped functions,
//   - because wasm only allows functions with explicitly defined signature
//   - all WASI functions to run tests with wasm3 would be imported
//   - probably we have some kind of top level __asm__ like in gcc to import functions
//   - or define types
//   - but it is not funny, because it really breaks an abstraction
//   - but it is the most b way possible, because this way we trust programmers
//   - or, or, we could infer types from usage but this way we will need to use full paths for
//     functions but now functions does not allow dots in names
//     so we will need some kind of convention for functions that use full paths
//     (for java and dalvik also, btw)
//     hehe, I really write text like LLMs in this repo, probably because I am not focused as them
//     I can assure you: I'm not a cat

struct Wasm32 {
    output: String_Builder,
    cmd: Cmd,
}

#[derive(Clone, Copy)]
pub struct Assembler {
    pub used_strings: Array<usize>,
    pub error_count: *mut usize, 
}

pub unsafe fn get_apis(targets: *mut Array<TargetAPI>) {
    da_append(targets, TargetAPI::V1 {
        name: c!("wasm32-wasi"),
        file_ext: c!(".wasm"),
        new,
        build: generate_program,
        run: |gen, program_path, run_args| {
            fprintf(stderr(), c!("RUN WASM 32 HERE\n"));
            todo!("Call wasm3 runner here");
            Some(())
        },
    });
}

pub unsafe fn usage(params: *const [Param]) {
    fprintf(stderr(), c!("wasm32 codegen for the B compiler\n"));
    fprintf(stderr(), c!("OPTIONS:\n"));
    print_params_help(params);
}

pub unsafe fn new(a: *mut arena::Arena, args: *const [*const c_char]) -> Option<*mut c_void> {
    let gen = arena::alloc_type::<Wasm32>(a);
    memset(gen as _ , 0, size_of::<Wasm32>());

    let mut help = false;
    let params = &[
        Param {
            name:        c!("help"),
            description: c!("Print this help message"),
            value:       ParamValue::Flag { var: &mut help },
        },
    ];

    if let Err(message) = parse_args(params, args) {
        usage(params);
        log(Log_Level::ERROR, c!("wasm32: %s"), message);
        return None;
    }

    if help {
        usage(params);
        return None;
    }

    Some(gen as *mut c_void)
}

pub unsafe fn write_byte(out: *mut String_Builder, byte: u8) {
    da_append(out, byte as c_char);
}

pub unsafe fn write_bytes(out: *mut String_Builder, bytes: *mut [u8]) {
    for i in 0..bytes.len() {
        let byte = (*bytes)[i];

        write_byte(out, byte);
    }

}

pub unsafe fn write_u32(output: *mut String_Builder, value: u32) {
    let bytes = value.to_le_bytes();
    for byte in bytes {
        write_byte(output, byte);
    }
}

// Write it as LEB128
pub unsafe fn write_u32_leb128(output: *mut String_Builder, mut value: u32) -> usize {
    let mut bytes_written = 0;
    loop {
        let mut byte = (value & 0x7F) as u8;
        value >>= 7;

        if value != 0 {
            byte |= 0x80;
            write_byte(output, byte);
            bytes_written += 1;
        } else {
            write_byte(output, byte);
            bytes_written += 1;
            break;
        }
    }
    bytes_written
}

// Write it as SLEB128
pub unsafe fn write_s32_sleb128(output: *mut String_Builder, mut value: i32) {
    let mut more = true;
    while more {
        let mut byte = (value & 0x7F) as u8;
        value >>= 7;

        let sign_bit = (byte & 0x40) != 0;

        if (value == 0 && !sign_bit) || (value == -1 && sign_bit) {
            more = false;
        } else {
            byte |= 0x80;
        }

        write_byte(output, byte);
    }
}

pub unsafe fn generate_globals(output: *mut String_Builder, globals: *const [Global], assembler: *mut Assembler, p: *const Program) -> Option<()> {
    for i in 0..globals.len() {
        let global = (*globals)[i];

        // TODO: replace with asm when implemented
        let data_offset = (*output).count as i32;
        // memory page:
        write_byte(output, 0x00); // memory 0
        // init with what:
        write_byte(output, 0x41); // i32_const value:SLEB128
        write_s32_sleb128(output, data_offset);
        write_byte(output, 0x0b); // end

        let mut nwords = 0u32;
        if global.is_vec {
            nwords += 1;
        }
        if global.minimum_size > 0 {
            nwords += global.minimum_size as u32;
        } else {
            nwords += global.values.count as u32;
        }
        write_u32_leb128(output, 4 * nwords);
        if global.is_vec {
            let vec_data_offset = ((*output).count + 4) as u32;
            nwords += 1;
            write_u32(output, vec_data_offset);
        }
        for j in 0..global.values.count {
            match *global.values.items.add(j) {
                ImmediateValue::IntLiteral(int_literal, radix) => {
                    let value: u32;
                    if let Some(v) = parse_int_literal_to_u32(int_literal, radix) {
                        value = v;
                    } else {
                        let prefix = match radix {
                            Radix::Dec => c!(""),
                            Radix::Oct => c!("0"),
                            Radix::Hex => c!("0x"),
                            _ => unreachable!()
                        };
                        diagf!(*global.value_locs.items.add(j), c!("ERROR: uxn: constant %s%s out of range for 16 bits\n"), prefix, int_literal);
                        value = bump_error_count((*p).error_count).map(|()| 0)?;
                    }
                    write_u32(output, value);
                }
                ImmediateValue::NegatedIntLiteral(int_literal, radix) => {
                    let value: u32;
                    if let Some(v) = parse_int_literal_to_u32(int_literal, radix) {
                        value = !v + 1;
                    } else {
                        let prefix = match radix {
                            Radix::Dec => c!(""),
                            Radix::Oct => c!("0"),
                            Radix::Hex => c!("0x"),
                            _ => unreachable!()
                        };
                        diagf!(*global.value_locs.items.add(j), c!("ERROR: uxn: constant -%s%s out of range for 16 bits\n"), prefix, int_literal);
                        value = bump_error_count((*p).error_count).map(|()| 0)?;
                    }
                    write_u32(output, value);
                }
                ImmediateValue::CharLiteral(char_literal, count) => {
                    let value: u32;
                    if let Some(v) = parse_char_literal_to_u32(char_literal, count) {
                        value = v;
                    } else {
                        diagf!(*global.value_locs.items.add(j), c!("ERROR: uxn: char constant '%s' out of range for 16 bits\n"), char_literal);
                        value = bump_error_count((*p).error_count).map(|()| 0)?;
                    }
                    write_u32(output, value);
                }
                ImmediateValue::Name(name) => {
                    write_u32(output, 0x00);
                }
                ImmediateValue::String(string_number) => {
                    da_append(&mut (*assembler).used_strings, string_number);
                    write_u32(output, 0x00);
                }
            }
        }
        for _ in global.values.count..global.minimum_size {
            write_u32(output, 0);
        }
    }
    Some(())
}

pub unsafe fn generate_strings(output: *mut String_Builder, assembler: *mut Assembler, strings: *const [String]) {
    let word_size = 4;

    for i in 0..(*assembler).used_strings.count {
        let used_string = *(*assembler).used_strings.items.add(i);
        let string = (*strings)[used_string];

        // TODO: replace with asm when implemented
        let data_offset = (*output).count as i32;
        // memory page:
        write_byte(output, 0x00); // memory 0
        // init with what:
        write_byte(output, 0x41); // i32_const value:SLEB128
        write_s32_sleb128(output, data_offset);
        write_byte(output, 0x0b); // end

        let mut temp_output: String_Builder = zeroed();
        dump_string(&mut temp_output, string.string, string.count);

        let bytes = slice::from_raw_parts(string.string as *const u8, string.count as usize);

        let mut nwords = bytes.len() / word_size;
        if bytes.len() % word_size > 0 {
            nwords += 1;
        }
        let full_len = (bytes.len() / word_size) * word_size;
        write_u32_leb128(output, (nwords * word_size) as u32);
        for chunk in bytes[..full_len].chunks(word_size) {
            for &byte in chunk.iter().rev() {
                write_byte(output, byte);
            }
        }

        let remainder = &bytes[full_len..];
        let mut tail = [0u8; 16];
        
        for (idx, &byte) in remainder.iter().enumerate() {
            tail[idx] = byte;
        }
        tail[remainder.len()] = word_size as u8;

        for &byte in tail[..word_size].iter().rev() {
            write_byte(output, byte);
        }
    }
}

pub unsafe fn generate_program(
    gen: *mut c_void, p: *const Program, program_path: *const c_char, 
    _garbage_base: *const c_char, _nostdlib: bool, debug: bool, 
) -> Option<()> {
    let gen = gen as *mut Wasm32;
    let output = &mut (*gen).output;
    let mut temp_output: String_Builder = zeroed();
    let mut type_output: String_Builder = zeroed();
    let mut func_output: String_Builder = zeroed();
    let mut data_output: String_Builder = zeroed();

    if debug { todo!("Debug information for wasm32") }

    let mut assembler: Assembler = zeroed();
    generate_globals(&mut data_output, da_slice((*p).globals), &mut assembler, p)?;
    generate_strings(&mut data_output, &mut assembler, da_slice((*p).strings));
    // generate_func_types(type_output, da_slice((*p).funcs), &mut assembler, p)?;
    // generate_extrns(da_slice((*p).extrns), da_slice((*p).funcs), da_slice((*p).asm_funcs), da_slice((*p).globals), p)?;
    // generate_funcs(func_output, da_slice((*p).funcs), &mut assembler, p)?;

    // Since wasm needs sizes in the begining of each section I pregenerate full sections and then
    // write them

    // Wasm module, version 1 (yes, version is not in LEB128, but an LE number):
    write_bytes(output, &mut [0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00]);

    // Memory section write:
    // Leaving it hardcoded for now
    // TODO: make it generate multiple meory pages
    write_byte(output, 0x05);
    write_u32_leb128(output, 3);
    write_u32_leb128(output, 0x01); // Blist count: 1 
    write_byte(output, 0x00); // As I understand it means memory only with minimum amount of
                              // pages, so resizible in theory

    write_u32_leb128(output, 0x01); // 1 page 

    // Data section write:
    write_byte(output, 0x0b);
    let global_count = ((*p).globals.count + assembler.used_strings.count) as u32;
    let global_count_length = write_u32_leb128(&mut temp_output, global_count) as u32;
    temp_output.count = 0;
    write_u32_leb128(output, data_output.count as u32 + global_count_length);
    write_u32_leb128(output, global_count as u32);
    let data_sec_bytes = slice::from_raw_parts_mut(data_output.items.cast::<u8>(), data_output.count);
    write_bytes(output, data_sec_bytes);
    write_entire_file(program_path, (*output).items as *const c_void, (*output).count)?;
    Some(())
}

