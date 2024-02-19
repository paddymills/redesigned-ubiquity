
use crossterm::{
    cursor,
    event,
    style::{self, Stylize},
    terminal::{self, disable_raw_mode, enable_raw_mode},
    QueueableCommand
};

use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL_CONDENSED;
use comfy_table::*;

use std::{io::{Stdout, Write}, sync::mpsc, time::Duration};

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


#[derive(Debug)]
pub enum DisplayUpdate<T> {
    DbResult(T),
    DbMessage(String),
}

#[derive(Debug)]
pub struct TableTerminal {
    write_to: Stdout,
    handler: InputHandler,

    table: Table,
    lines_drawn: u16,
}

impl TableTerminal {
    pub fn new(header: impl Into<Row>, handler: InputHandler) -> Self {
        let mut table = Table::new();

        table
            .load_preset(UTF8_FULL_CONDENSED)
            .apply_modifier(UTF8_ROUND_CORNERS)
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_header(header);

        Self {
            write_to: std::io::stdout(),
            handler,
            table,
            lines_drawn: 0
        }
    }

    pub fn input_loop<T>(&mut self, rx: mpsc::Receiver<DisplayUpdate<T>>) -> anyhow::Result<()>
        where
            T: Into<Row> + Send + 'static + std::fmt::Debug
    {
        println!("{INSTRUCTIONS}");

        enable_raw_mode().expect("failed to enable raw mode for keyboard input");

        loop {
            if let Ok(true) = event::poll(Duration::from_millis(100)) {
                if let Err(0) = self.handler.handle_input(event::read()?) {
                    self.write_to.queue(cursor::MoveToColumn(0))?;
                    self.write_to.queue(terminal::Clear(terminal::ClearType::FromCursorDown))?;
                    self.write_to.queue(style::Print(String::from("Goodbye...")))?;
                    self.write_to.flush()?;
                    break;
                }
            }

            // move to start of row, in case no vertical moves are processed
            self.write_to.queue(cursor::MoveToColumn(0))?;
            self.write_to.queue(terminal::Clear(terminal::ClearType::FromCursorDown))?;

            // handle database results
            if let Ok(res) = rx.try_recv() {
                match res {
                    DisplayUpdate::DbResult(parsed) => {
                        log::trace!("Adding row to table: {:?}", parsed);
                        self.table.add_row(parsed);

                        self.draw_table()?
                    },
                    DisplayUpdate::DbMessage(msg) => self.draw_message(msg)?
                }
            }

            self.draw_prompt()?;
            self.write_to.flush()?;
        }

        log::info!("Terminal writer shutting down");
        let _ = disable_raw_mode();
        
        Ok(())
    }

    fn draw_table(&mut self) -> anyhow::Result<()> {
        // prepare for reprint
        self.write_to.queue(cursor::MoveUp(self.lines_drawn))?;
        self.write_to.queue(terminal::Clear(terminal::ClearType::FromCursorDown))?;

        // print table
        self.lines_drawn = 0;
        for line in self.table.lines() {
            self.write_to.queue(style::Print( format!("{line}\n") ))?;
            self.lines_drawn += 1;
        }

        self.draw_prompt()
    }

    fn draw_message(&mut self, msg: String) -> anyhow::Result<()> {
        log::trace!("Displaying message `{}`", msg);

        if self.lines_drawn == self.table.lines().count() as u16 + 1 {
            // message was printed
            log::trace!("Previous message printed. drawn: {} table_len: {}", self.lines_drawn, self.table.lines().count());
            self.write_to.queue(cursor::MoveUp(1))?;
            self.lines_drawn -= 1;
        } else {
            log::trace!("No previous message printed. drawn: {} table_len: {}", self.lines_drawn, self.table.lines().count());
            self.write_to.queue(cursor::MoveToColumn(0))?;
        }

        self.write_to.queue(terminal::Clear(terminal::ClearType::FromCursorDown))?;

        self.write_to.queue(style::PrintStyledContent( format!("{msg}\n").red() ))?;
        self.lines_drawn += 1;

        self.draw_prompt()
    }

    fn draw_prompt(&mut self) -> anyhow::Result<()> {
        self.write_to.queue(cursor::MoveToColumn(0))?;
        self.write_to.queue(terminal::Clear(terminal::ClearType::FromCursorDown))?;

        self.handler.prompt(&mut self.write_to)?;

        Ok(())
    }
}
