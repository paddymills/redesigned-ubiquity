
use std::fmt::Display;
use std::io::{self, Cursor, Write, Seek, SeekFrom};
use std::ops::{Deref, DerefMut};

type CursorType = Cursor<Vec<u8>>;

#[derive(Debug, Default)]
pub struct InputBuffer(CursorType);

impl InputBuffer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn delete(&mut self) {
        let pos = self.position() as usize;
        
        if pos < self.0.get_ref().len() {
            self.0.get_mut().remove(pos);
        }
    }
    
    pub fn backspace(&mut self) -> io::Result<()> {
        let pos = self.position() as usize;
        if pos > 0 {
            self.0.get_mut().remove(pos-1);
            self.seek(SeekFrom::Current(-1))?;
        }
        
        Ok(())
    }

    pub fn prefix(&self) -> &str {
        let bytes = self.get_ref()
            .splitn(2, |x| b"-_".contains(x)).next()
            .unwrap_or(self.get_ref());

        unsafe { std::str::from_utf8_unchecked(bytes) }
    }

    pub fn trim_prefix<'o>(&self, other: &'o str) -> &'o str {
        match other.len().checked_sub(self.prefix().len()) {
            Some(len) => &other[..len],
            None => ""
        }
    }

    pub fn apply_prefix(&mut self, other: &str) -> io::Result<()> {
        let prepend = self.trim_prefix(other);
        if prepend.len() > 0 {
            let original_pos = self.position();
            
            self.set_position(0);
            self.write(prepend.as_bytes())?;
            
            self.set_position(prepend.len() as u64 + original_pos);
        }
        
        Ok(())
    }
}

impl Deref for InputBuffer {
    type Target = CursorType;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for InputBuffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Write for InputBuffer {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let current_pos = self.position() as usize;
        let buffer_data = self.0.get_ref();
        
        // Split the buffer at current position
        let (before, after) = buffer_data.split_at(current_pos);
        
        // Create new buffer with inserted data
        let mut new_data = Vec::with_capacity(before.len() + buf.len() + after.len());
        new_data.extend_from_slice(before);
        new_data.extend_from_slice(buf);
        new_data.extend_from_slice(after);
        
        // Replace buffer contents
        *self.0.get_mut() = new_data;
        
        // Update position
        self.set_position(current_pos as u64 + buf.len() as u64);

        Ok(buf.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        self.0.flush()
    }
}

impl Display for InputBuffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = unsafe { String::from_utf8_unchecked(self.0.get_ref().to_vec()) };
        // s.insert(self.position() as usize, '|');
        
        write!(f, "{}", s)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_writing() {
        let mut buf = InputBuffer::new();

        buf.write(b"abcd").unwrap();
        assert_eq!(format!("{}", buf), "abcd|");

        buf.write(&[b'e']).unwrap();
        assert_eq!(format!("{}", buf), "abcde|");

        buf.seek(SeekFrom::Current(-2)).unwrap();
        buf.write(&[b'0']).unwrap();
        assert_eq!(format!("{}", buf), "abc0|de");

        buf.write(b"xyz").unwrap();
        assert_eq!(format!("{}", buf), "abc0xyz|de");
    }

    #[test]
    fn test_delete() {
        let mut buf = InputBuffer::new();

        buf.write(b"abcd").unwrap();
        buf.delete();
        assert_eq!(format!("{}", buf), "abcd|");

        buf.seek(SeekFrom::Start(2)).unwrap();
        buf.delete();
        assert_eq!(format!("{}", buf), "ab|d");
    }

    #[test]
    fn test_backspace() {
        let mut buf = InputBuffer::new();

        buf.write(b"abcd").unwrap();
        buf.backspace().unwrap();
        assert_eq!(format!("{}", buf), "abc|");

        buf.seek(SeekFrom::Start(0)).unwrap();
        buf.backspace().unwrap();
        assert_eq!(format!("{}", buf), "|abc");
    }

    #[test]
    fn test_prefix() {
        let mut buf = InputBuffer::new();
        buf.write(b"ab_d").unwrap();
        assert_eq!(buf.prefix(), "ab");

        buf.seek(SeekFrom::Start(2)).unwrap();
        buf.delete();
        buf.write(b"-").unwrap();
        assert_eq!(buf.prefix(), "ab");

        buf.backspace().unwrap();
        assert_eq!(buf.prefix(), "abd");
    }

    #[test]
    fn test_apply_prefix() {
        let mut buf = InputBuffer::new();
        buf.write(b"abcd-1").unwrap();
        
        buf.seek(SeekFrom::Current(-3)).unwrap();
        buf.apply_prefix("12345").unwrap();
        assert_eq!(format!("{}", buf), "1abc|d-1");

        buf.apply_prefix("789").unwrap();
        assert_eq!(format!("{}", buf), "1abc|d-1");
        
        buf.seek(SeekFrom::End(0)).unwrap();
        buf.apply_prefix("9xxxxx").unwrap();
        assert_eq!(format!("{}", buf), "91abcd-1|");
    }
}