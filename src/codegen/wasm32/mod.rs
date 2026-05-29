use core::ffi::*;
use crate::arena;
use crate::crust::libc::*;
use crate::ir::*;
use crate::nob::*;
use crate::params::*;
use crate::targets::TargetAPI;

struct Wasm32 {
    output: String_Builder,
    cmd: Cmd,
}

pub unsafe fn get_apis(targets: *mut Array<TargetAPI>) {
    da_append(targets, TargetAPI::V1 {
        name: c!("wasm32"),
        file_ext: c!(".wasm"),
        new,
        build: generate_program,
        run: |gen, program_path, run_args| {
            fprintf(stderr(), c!("RUN WASM 32\n"));
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

pub unsafe fn generate_program(
    gen: *mut c_void, p: *const Program, program_path: *const c_char, 
    _garbage_base: *const c_char, _nostdlib: bool, debug: bool, 
) -> Option<()> {
    let gen = gen as *mut Wasm32;
    let output = &mut (*gen).output;

    if debug { todo!("Debug information for wasm32") }

    fprintf(stderr(), c!("BUILD WASM 32\n"));
    Some(())
}

