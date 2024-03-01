
use std::fmt::Debug;
use std::io::{self, Write};
use std::sync::mpsc::Receiver;
use std::time::Duration;

use comfy_table::{ContentArrangement, Row, Table};
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL_CONDENSED;

use crossterm::event::MouseEvent;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    style::{Print, Stylize},
    terminal,
    QueueableCommand
};

use copypasta::{ClipboardContext, ClipboardProvider};

#[derive(Debug, Clone)]
pub enum DisplayUpdate<T: Into<Row>> {
    DbResult(T),
    Message(String),
    Redraw,
    CopyToClipboard,
    ClearTable,
}

#[derive(Debug)]
pub struct QueryTableUi {
    instructions: Option<String>,
    table: Table,
    message: Option<Result<String, String>>,

    stdout: io::Stdout,
}

impl QueryTableUi
{
    pub fn new(table_header: impl Into<Row>) -> Self {
        Self {
            instructions: None,
            table: Self::init_table(table_header),
            message: None,
            stdout: io::stdout(),
        }
    }

    fn init_table(table_header: impl Into<Row>) -> Table {
        let mut table = Table::new();
        table
            .load_preset(UTF8_FULL_CONDENSED)
            .apply_modifier(UTF8_ROUND_CORNERS)
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_header(table_header);

        table
    }

    pub fn with_instructions(&mut self, instructions: impl ToString) -> &mut Self {
        self.instructions = Some(instructions.to_string());

        self
    }

    pub fn run_loop<R: Into<Row> + Debug>(&mut self, handler: &mut impl InputHandler<R>, results: Receiver<DisplayUpdate<R>>) -> anyhow::Result<()> {

        terminal::enable_raw_mode().expect("failed to enable raw mode for keyboard input");
        self.stdout
            .queue(terminal::EnterAlternateScreen)?
            .flush()?;

        // draw initial
        self.draw(handler)?;

        loop {
            if let Ok(true) = event::poll(Duration::from_millis(100)) {
                match handler.handle_input(event::read()?) {
                    Ok(_) => self.draw(handler)?,
                    Err(error) if error.kind() == io::ErrorKind::Interrupted  => {
                        log::info!("Exit called");
                        self.stdout
                            .queue(cursor::MoveToColumn(0))?
                            .queue(terminal::Clear(terminal::ClearType::FromCursorDown))?
                            .queue(Print(String::from("Goodbye...")))?
                            .flush()?;
                        
                        break;
                    },
                    // TODO: handle io errors
                    _ => ()
                }
            }

            // handle database results
            for res in results.try_iter() {
                match res {
                    DisplayUpdate::DbResult(parsed) => {
                        log::trace!("Adding row to table: {:?}", parsed);
                        self.table.add_row(parsed);
                        self.message = None;

                        self.draw(handler)?
                    },
                    DisplayUpdate::Message(msg) => {
                        self.message = Some(Err(msg));

                        self.draw(handler)?
                    },
                    DisplayUpdate::CopyToClipboard => {
                        let contents = strip_ansi_escapes::strip_str(&self.table.to_string());

                        let mut ctx = ClipboardContext::new().unwrap();

                        match ctx.set_contents(contents) {
                            Ok(_) => self.message = Some(Ok(String::from("table written to clipboard"))),
                            Err(_) => self.message = Some(Err(String::from("failed to write table to clipboard")))
                        }
                    },
                    DisplayUpdate::Redraw => self.draw(handler)?,
                    DisplayUpdate::ClearTable => {
                        self.table = Self::init_table(self.table.header().unwrap().clone());
                        self.message = Some(Ok(String::from("table cleared")));

                        self.draw(handler)?
                    }
                }
            }
        }

        log::info!("Terminal writer shutting down");
        
        self.stdout
            .queue(terminal::LeaveAlternateScreen)?
            .queue(Print( format!("{}", self.table) ))?
            .flush()?;
        let _ = terminal::disable_raw_mode();

        Ok(())
    }


    fn draw<R: Into<Row> + Debug>(&mut self, handler: &mut impl InputHandler<R>) -> io::Result<()> {
        self.stdout
            .queue(cursor::MoveTo(0, 0))?
            .queue(terminal::Clear(terminal::ClearType::FromCursorDown))?;
            
        if let Some(msg) = &self.instructions {
            self.stdout.queue(Print( format!("{msg}\n").blue() ))?;
        }

        if !self.table.is_empty() {
            // print table
            self.stdout.queue(Print( format!("{}\n", self.table) ))?;
        }

        match &self.message {
            Some(Ok(msg))  => { self.stdout.queue(Print( format!("{msg}\n").green() ))?; },
            Some(Err(msg)) => { self.stdout.queue(Print( format!("{msg}\n").red() ))?; },
            _ => ()
        }

        // draw prompt should call stdout.flush()
        handler.draw_prompt(&mut self.stdout)
    }
}

pub trait InputHandler<T>
    where Self: Sized, T: Into<Row>
{
    fn terminate() -> io::Result<()> {
        Err(io::Error::new(io::ErrorKind::Interrupted, "Terminate loop called"))
    }

    fn write_buffer(&mut self, buf: &[u8]) -> io::Result<()>;
    fn ui_update(&self, message: DisplayUpdate<T>);
    
    /// Handle input events
    fn handle_key_input(&mut self, event: KeyEvent) -> io::Result<()>;
    fn handle_mouse_input(&mut self, event: MouseEvent) -> io::Result<()> {
        log::trace!("Received mouse event: {:?}", event);    
        Ok(())
    }

    fn handle_input(&mut self, event: Event) -> io::Result<()> {
        log::trace!("Received event: {:?}", event);

        match event {
            Event::Key(KeyEvent { code: KeyCode::Char('c'), modifiers: KeyModifiers::CONTROL, kind: KeyEventKind::Release, .. })
                => Self::terminate()?,
            // {PrintScreen} -> copy to clipboard
            Event::Key(KeyEvent { code: KeyCode::PrintScreen, modifiers: KeyModifiers::NONE, kind: KeyEventKind::Release, .. })
                => self.ui_update(DisplayUpdate::CopyToClipboard),
            Event::Key(event) => self.handle_key_input(event)?,
            Event::Paste(data) => self.write_buffer(&data.as_bytes())?,
            Event::Resize(..) => self.ui_update(DisplayUpdate::Redraw),
            Event::Mouse(event) => self.handle_mouse_input(event)?,  
            _ => ()  
        }

        Ok(())
    }

    fn draw_prompt(&self, stdout: &mut io::Stdout) -> io::Result<()>;
}
