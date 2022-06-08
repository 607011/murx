/**
 * Copyright (c) 2022 Oliver Lau <oliver@ersatzworld.net>
 * All rights reserved.
 */

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let obj_filename = &args[1];
    let mut vm = riscvm::Machine::new();
    match vm.load(&obj_filename) {
        Ok(()) => match vm.run() {
            Ok(()) => (),
            Err(e) => panic!("{}", e),
        },
        Err(e) => panic!("{}", e),
    }
}
