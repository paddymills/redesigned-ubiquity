
use crossterm::{
    event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    style::{Print, PrintStyledContent, Stylize},
    QueueableCommand
};
use std::{
    io::{self, Write},
    sync::mpsc::Sender
};
// use tokio::sync::mpsc::Sender;

const DELIMS: &[char; 2] = &['-', '_'];
const EXPECTED_PROGRAM_NAME_LENGTH: usize = 5;


#[derive(Debug)]
pub struct InputHandler {
    prompt: String,
    last_input: String,
    buffer: String,

    db: Sender<String>,
}

impl InputHandler {
    pub fn new<S: ToString>(prompt: S, db: Sender<String>) -> Self {
        Self {
            prompt: prompt.to_string(),
            last_input: String::new(),
            buffer: String::new(),

            db,
        }
    }

    /// Handle input event
    /// 
    /// returns Err(0) if the input matches a quit condition
    pub fn handle_input(&mut self, event: Event) -> Result<(), u32> {
        log::trace!("Received event: {:?}", event);

        match event {
            Event::Key(KeyEvent { code: KeyCode::Char('c'), modifiers: KeyModifiers::CONTROL, kind: KeyEventKind::Release, .. }) |
            Event::Key(KeyEvent { code: KeyCode::Esc, modifiers: KeyModifiers::NONE, kind: KeyEventKind::Release, .. })
                => return Err(0),
            Event::Key(KeyEvent { code, modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, .. }) => {
                match code {
                    KeyCode::Enter => {
                        if self.buffer.len() == 0 {
                            return Err(0)
                        }

                        // match on buffer length (buffer suffix if the buffer is delimited by DELIM)
                        match self.buffer_prefix().len() {
                            l if l == EXPECTED_PROGRAM_NAME_LENGTH => self.last_input = self.buffer.drain(..).collect(),

                            // input is shorter than last_input -> update last_input RTL
                            //   note the use of `buffer.drain(..)`. This takes essentially clears the buffer and returns the results
                            l if l < self.last_input.len() =>
                                self.last_input.replace_range(self.last_input.len()-l.., self.buffer.drain(..).collect::<String>().as_str()),
                            
                            _ => self.last_input = self.buffer.drain(..).collect()
                        }

                        // Send input to database for results
                        let _ = self.db.send(self.last_input.clone());
                        log::trace!("InputHandler asked db for `{}` results", self.last_input);

                        // if last_program has a '-' or '_' in it, only retain the first part
                        if let Some((head, _tail)) = self.last_input.split_once(DELIMS) {
                            self.last_input = String::from(head);
                        }
                    },
                    KeyCode::Backspace => { self.buffer.pop(); },
                    KeyCode::Char(c) => self.buffer.push(c),
                    _ => ()
                }
            },
            Event::Paste(data) => self.buffer.push_str(&data),
            // TODO: keyboard: {Left, Right, Up, Down, Home, End}
            // TODO: keyboard: {Delete}
            // TODO: keyboard: {PrintScreen} -> copy to clipboard
            // TODO: mouse support
            // Event::Mouse(event) => (),  
            _ => ()
        }

        Ok(())
    }

    fn buffer_prefix(&self) -> &str {
        self.buffer.split_once(DELIMS).map(|x| x.0).unwrap_or(&self.buffer)
    }

    pub fn prompt<T>(&self, print_to: &mut T) -> io::Result<()>
        where T: Write + Sized
    {
        print_to.queue(Print(&self.prompt))?;
        print_to.queue(Print(String::from(" > ")))?;

        match (self.last_input.len(), self.buffer_prefix().len()) {
            (l, b) if l > 0 && l > b => {
                print_to.queue(PrintStyledContent(self.last_input.get(..l-b).unwrap().dim()))?;
            },
            _ => ()
        }

        print_to.queue(Print(&self.buffer))?;

        Ok(())
    }
}