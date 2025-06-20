
#![feature(slice_split_once)]
#![feature(linked_list_cursors)]

mod buffer;
mod input;
mod program;
mod query;
mod termui;

pub use buffer::InputBuffer;
pub use input::ProgramInputHandler;
pub use program::{Program, ProgramState, HEADER};
pub use query::Query;
pub use termui::{DisplayUpdate, InputHandler, QueryTableUi};
