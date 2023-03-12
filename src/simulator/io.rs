#[derive(Debug, Default)]
pub struct Io {
    pub lines: Vec<String>,
    pub buf: String,
    pub in_buf: String,
}

impl Io {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reset(&mut self) {
        std::mem::take(&mut self.lines);
        std::mem::take(&mut self.buf);
    }

    pub fn add(&mut self, string: String) {
        for c in string.chars() {
            match c {
                '\n' => self.lines.push(std::mem::take(&mut self.buf)),
                _ => self.buf.push(c),
            }
        }
    }
}
