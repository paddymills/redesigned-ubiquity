
use crossterm::{cursor, terminal, QueueableCommand};
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::style::{Print, Stylize};

use std::io::{self, Seek, SeekFrom, Write};
use std::sync::mpsc::Sender;

use super::{DisplayUpdate, InputBuffer, InputHandler};

const COMMANDS: &str = r#"
    ##############################################################
    #                          Commands                          #
    #                     ------------------                     #
    # :c, :clear     ->  Clear the table                         #
    # :r, :reset     ->  Reset the input history                 #
    # :p, :print     ->  Write the table to the system clipboard #
    ##############################################################
"#;
const TIPS: &str = r#"
    ##############################################################
    #                        Instructions                        #
    #                     ------------------                     #
    # This utility will query the database for the status of the #
    # program the user inputs.                                   #
    #                     ------------------                     #
    # * Type a program name and press enter to get the status    #
    # * To exit, either press Escape or submit a blank input     #
    # * The prefix(input before a - or _) of the last input      #
    #     will be used to autocomplete the current input prefix  #
    #     (as shown by the dimmed text)                          #
    #                     ------------------                     #
    # * Type : for command mode (commands will be displayed)     #
    # * If the prefix completion becomes an issue, type :reset   #
    #                     ------------------                     #
    #                (press Escape or ? to close)                #
    ##############################################################
"#;

#[derive(Debug, PartialEq)]
enum InputMode {
    Prompt(bool /* show tips */ ),
    Command
}

#[derive(Debug)]
pub struct ProgramInputHandler<T: Into<comfy_table::Row>> {
    buffers: Vec<InputBuffer>,
    buffer_id: usize,
    submit_to: Sender<String>,
    display: Sender<DisplayUpdate<T>>,
    mode: InputMode,
}

impl<T: Into<comfy_table::Row>> ProgramInputHandler<T> {
    pub fn new(submit_to: Sender<String>, display: Sender<DisplayUpdate<T>>) -> Self {
        Self {
            buffers: vec![InputBuffer::new()],
            buffer_id: 0,
            submit_to,
            display,
            mode: InputMode::Prompt(false)
        }
    }

    fn buffer(&self) -> &InputBuffer {
        assert!(self.buffer_id < self.buffers.len());
        self.buffers.get(self.buffer_id).unwrap()
    }

    fn buffer_mut(&mut self) -> &mut InputBuffer {
        assert!(self.buffer_id < self.buffers.len());
        self.buffers.get_mut(self.buffer_id).unwrap()
    }

    fn switch_buffer(&mut self, direction: i64) {
        if let InputMode::Prompt(_) = self.mode {
            self.buffer_id = (self.buffer_id as i64 + direction).clamp(0i64, self.buffers.len() as i64 - 1) as usize;
        }
    }

    fn previous_prefix(&self) -> Option<&str> {
        match self.buffers.len() {
            l if l > 1 && self.buffer_id == l-1
                // current buffer is the last buffer and there is a previous buffer
                => Some(self.buffers.get(self.buffer_id - 1).unwrap().prefix()),
            _ => None
        }
    }

    fn commit_buffer(&mut self) -> io::Result<()> {
        match self.mode {
            InputMode::Prompt(_) => {
                // apply previous buffer prefix to current one
                let prefix = String::from(self.previous_prefix().unwrap_or(""));    // String required to escape mutable and immutable reference conflict
                self.buffers.get_mut(self.buffer_id).unwrap().apply_prefix(&prefix)?;

                // Send input to database for results
                self.buffer().to_string()
                    .split(' ')
                    .for_each(|val| {
                        match self.submit_to.send(String::from(val)) {
                            Ok(_) => log::info!("`{}` sent to db thread", self.buffer()),
                            Err(e) => log::error!("Error sending input to db thread: {}", e)
                        }
                    });

                // reset buffer
                self.buffer_id = self.buffers.len();
                self.buffers.push( InputBuffer::new() );
            },
            InputMode::Command => {
                match self.buffer().to_string().to_lowercase().as_str() {
                    ":r" | ":reset" => {
                        self.buffers = self.buffers.split_off(self.buffers.len() - 3);
                        self.buffer_id = 0;
                    },
                    ":c" | ":clear" => self.ui_update(DisplayUpdate::ClearTable),
                    ":p" | ":print" => self.ui_update(DisplayUpdate::CopyToClipboard),
                    cmd => self.ui_update(DisplayUpdate::Message( format!("unrecognized command `:{}`", cmd)))
                }

                self.mode = InputMode::Prompt(false);
                self.switch_buffer(-1);
                let _ = self.buffers.pop();
            }
        }
        
        Ok(())
    }
}

impl<T: Into<comfy_table::Row>> InputHandler<T> for ProgramInputHandler<T> {
    fn handle_key_input(&mut self, event: KeyEvent) -> io::Result<()> {
        match event {
            KeyEvent { code, modifiers: KeyModifiers::NONE | KeyModifiers::SHIFT, kind: KeyEventKind::Press | KeyEventKind::Repeat, .. } => {
                match code {
                    KeyCode::Esc => {
                        match self.mode {
                            InputMode::Prompt(false) => Self::terminate()?,
                            InputMode::Prompt(true) => self.mode = InputMode::Prompt(false),
                            InputMode::Command => {
                                // return to Prompt mode
                                self.mode = InputMode::Prompt(false);
                                self.switch_buffer(-1);
                                let _ = self.buffers.pop();
                            }
                        }
                    },
                    KeyCode::Enter => {
                        if self.buffer().get_ref().len() == 0 {
                            Self::terminate()?
                        } else {
                            self.commit_buffer()?
                        }
                    },
                    
                    KeyCode::Backspace => self.buffer_mut().backspace()?,
                    KeyCode::Delete    => self.buffer_mut().delete(),

                    // {Left, Right, Home, End}
                    // ignore errors because a seek out of bounds is an error
                    KeyCode::Home  => { let _ = self.buffer_mut().seek(SeekFrom::Start(0)); },
                    KeyCode::End   => { let _ = self.buffer_mut().seek(SeekFrom::End(0)); },
                    KeyCode::Left  => { let _ = self.buffer_mut().seek(SeekFrom::Current(-1)); },
                    KeyCode::Right => { let _ = self.buffer_mut().seek(SeekFrom::Current(1)); },

                    // {Up, Down} buffer switching, if in Prompt mode
                    KeyCode::Up   => self.switch_buffer(-1),
                    KeyCode::Down => self.switch_buffer(1),
                    
                    KeyCode::Char(':') => {
                        if self.mode != InputMode::Command {
                            self.mode = InputMode::Command;
                            self.buffer_id = self.buffers.len();
                            self.buffers.push( InputBuffer::new() );
                            self.write_buffer(b":")?;
                        }
                    },
                    KeyCode::Char('?') => {
                        self.mode = match self.mode {
                            InputMode::Prompt(true) => InputMode::Prompt(false),
                            _ => InputMode::Prompt(true)
                        };
                    },

                    KeyCode::Char(c) => self.write_buffer(c.encode_utf8(&mut [0u8 ;1]).as_bytes())?,
                    _ => ()
                }
            },
            _ => ()
        }

        Ok(())
    }

    fn write_buffer(&mut self, buf: &[u8]) -> io::Result<()> {
        self.buffer_mut().write(&buf).map(|_| ())
    }

    fn draw_prompt(&self, stdout: &mut io::Stdout) -> io::Result<()> {
        stdout
            .queue(cursor::MoveToColumn(0))?
            .queue(terminal::Clear(terminal::ClearType::FromCursorDown))?;

        match self.mode {
            InputMode::Prompt(show_tips) => {
                let prefix_hint = match self.previous_prefix() {
                    Some(prev) if self.buffer().get_ref().len() > 0 => self.buffer().trim_prefix(prev),
                    _ => ""
                };

                if show_tips {
                    stdout.queue(Print( format!("{}\n", TIPS).blue() ))?;
                }
        
                stdout
                    .queue(Print(format!("Program > {}{}", prefix_hint.dark_grey(), self.buffer())))?
                    .queue(cursor::MoveToColumn(0))?
                    .queue(cursor::MoveRight((10 + prefix_hint.len() + self.buffer().position() as usize) as u16))?;
            },
            InputMode::Command => {
                stdout
                    .queue(Print( format!("{}\n", COMMANDS).blue() ))?
                    .queue(Print( format!("Command > {}", self.buffer()) ))?
                    .queue(cursor::MoveToColumn(0))?
                    .queue(cursor::MoveRight((10 + self.buffer().position()) as u16))?;
            },
        }

        stdout.flush()
    }

    fn ui_update(&self, message: DisplayUpdate<T>) {
        // we don't want to crash the program
        let _ = self.display.send(message);
    }
}