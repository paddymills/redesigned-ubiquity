
#![feature(cursor_remaining, slice_split_once)]
#![feature(linked_list_cursors)]

mod buffer;
mod input;
mod program;
mod termui;

pub use buffer::InputBuffer;
pub use input::ProgramInputHandler;
pub use program::{Program, ProgramState, HEADER};
pub use termui::{DisplayUpdate, InputHandler, QueryTableUi};
