#![windows_subsystem = "windows"]

use crate::gui::show_gui;

mod gui;
mod fix_file;

fn main() {
    show_gui().unwrap();
}
