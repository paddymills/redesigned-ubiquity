

use crossterm::{
    cursor,
    event,
    style::{self, Print, Stylize},
    terminal::{self, disable_raw_mode, enable_raw_mode},
    ExecutableCommand, QueueableCommand
};

use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL_CONDENSED;
use comfy_table::*;

use std::{io::Write, sync::mpsc, time::Duration};

use crate::InputHandler;

const INSTRUCTIONS: &str = r#"
##############################################################
#                      Updated Programs                      #
#                     ------------------                     #
# This utility will query the database for the status of the #
# program the user inputs.                                   #
#                                                            #
# Type a program name and press enter to get the status      #
# To exit, either press Escape or submit a blank input       #
##############################################################
"#;


#[derive(Debug, Default)]
pub struct TableTerminal {}

#[derive(Debug)]
pub enum DisplayUpdate<T> {
    DbResult(T),
    DbMessage(String),
}

impl TableTerminal {
    pub fn input_loop<H, T>(header: H, rx: mpsc::Receiver<DisplayUpdate<T>>, mut handler: InputHandler) -> anyhow::Result<()>
        where
            H: Into<Row>,
            T: Into<Row> + Send + 'static + std::fmt::Debug
    {
        // let mut stdout = Term::stdout();
        let mut stdout = std::io::stdout();

        enable_raw_mode().expect("failed to enable raw mode for keyboard input");
        
        let mut table = Table::new();

        table
            .load_preset(UTF8_FULL_CONDENSED)
            .apply_modifier(UTF8_ROUND_CORNERS)
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_header(header);

        let mut lines_printed = 0u16;


        stdout.execute(Print(INSTRUCTIONS))?;

        loop {
            if let Ok(true) = event::poll(Duration::from_millis(100)) {
                if let Err(0) = handler.handle_input(event::read()?) {
                    stdout.queue(cursor::MoveToColumn(0))?;
                    stdout.queue(terminal::Clear(terminal::ClearType::FromCursorDown))?;
                    stdout.queue(style::Print(String::from("Goodbye...")))?;
                    stdout.flush()?;
                    break;
                }
            }

            // move to start of row, in case no vertical moves are processed
            stdout.queue(cursor::MoveToColumn(0))?;
            stdout.queue(terminal::Clear(terminal::ClearType::FromCursorDown))?;

            // handle database results
            if let Ok(res) = rx.try_recv() {
                match res {
                    DisplayUpdate::DbResult(parsed) => {
                        log::trace!("Adding row to table: {:?}", parsed);
                        table.add_row(parsed);

                        // prepare for reprint
                        stdout.queue(cursor::MoveUp(lines_printed))?;
                        stdout.queue(terminal::Clear(terminal::ClearType::FromCursorDown))?;

                        // print table
                        lines_printed = 0;
                        for line in table.lines() {
                            stdout.queue(style::Print( format!("{line}\n") ))?;
                            lines_printed += 1;
                        }
                    },
                    DisplayUpdate::DbMessage(msg) => {
                        log::trace!("Displaying message `{}`", msg);

                        // if message was printed
                        if lines_printed == table.lines().count() as u16 + 1 {
                            stdout.queue(cursor::MoveUp(1))?;
                        }

                        stdout.queue(style::PrintStyledContent( format!("{msg}\n").red() ))?;
                        lines_printed += 1;
                    }
                }
            }

            handler.prompt(&mut stdout)?;
            stdout.flush()?;
        }

        log::info!("Terminal writer shutting down");
        let _ = disable_raw_mode();
        
        Ok(())
    }
}
