use std::process::exit;
use std::time::Instant;

use vm::{InterpretResult, VM};

mod chunk;
mod compiler;
mod debug;
mod function;
mod interner;
mod scanner;
mod value;
mod vm;
use std::{env, fs, io};

fn main() {
    let now = Instant::now();

    let mut vm = VM::new();
    let mut argv = env::args();
    match argv.len() {
        1 => {
            repl(&mut vm);
        }
        2 => {
            run_file(&mut vm, &argv.nth(1).expect("Could not parse argv"));
        }
        _ => {
            eprintln!("Usage: clox [path]");
            exit(64);
        }
    }

    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?}", elapsed);
}

fn repl(vm: &mut VM) {
    // char line[1024];
    let mut buffer = String::new();
    let stdin = io::stdin();

    loop {
        print!("> ");
        match stdin.read_line(&mut buffer) {
            Ok(0) | Err(_) => {
                println!();
                break;
            }
            Ok(_) => {
                vm.interpret(&buffer);
            }
        }
    }
}

fn run_file(vm: &mut VM, path: &str) {
    let source = fs::read_to_string(path).expect("Could not open file");
    let result = vm.interpret(&source);
    // free(source);

    match result {
        Ok(_) => exit(0),
        Err(InterpretResult::CompileError) => exit(65),
        Err(InterpretResult::RuntimeError) => exit(70),
        Err(InterpretResult::Ok) => exit(0), // should not happen
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryInto;

    use crate::chunk::{Chunk, OpCode};
    use crate::debug::disassemble_chunk;
    use crate::interner::Interner;
    use crate::value;
    use crate::vm::VM;

    #[test]
    fn ch14_chunk() {
        let mut vm = VM::new();
        let res = vm.interpret("print 1.2;");
        assert_eq!(res.err(), None);
    }

    #[test]
    fn ch15_vm() {
        let mut vm = VM::new();
        let res = vm.interpret("print - (1.2 + 3.4 / 5.6);");
        assert_eq!(res.err(), None);
    }

    #[test]
    fn ch18_values() {
        let mut vm = VM::new();
        // let res = vm.interpret("print 5 - 4 > 3 * 2;");
        // assert_eq!(res.err(), None); // false
        // let res = vm.interpret("print !nil;");
        // assert_eq!(res.err(), None); // true
        // let res = vm.interpret("print (5 - 4 > 3 * 2 == !nil);");
        // assert_eq!(res.err(), None); // false
        let res = vm.interpret("print !(5 - 4 > 3 * 2 == !nil);");
        assert_eq!(res.err(), None); // true
    }

    #[test]
    fn ch21_global() {
        let mut vm = VM::new();        
        let res = vm.interpret("print (1 * 2 = 3 + 4);");
        assert_eq!(res.err(), Some(crate::vm::InterpretResult::CompileError));
    }
}
