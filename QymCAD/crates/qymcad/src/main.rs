//! QymCAD - the executable: the program itself is the `qymcad` library.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() -> std::process::ExitCode {
    qymcad::run()
}
