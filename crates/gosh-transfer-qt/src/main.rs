// SPDX-License-Identifier: AGPL-3.0
// Gosh Transfer Qt - entry point

mod engine_bridge;
mod qt;

extern "C" {
    fn run_app() -> i32;
}

fn main() {
    // Qt event loop is driven from C++ side.
    let code = unsafe { run_app() };
    std::process::exit(code);
}
