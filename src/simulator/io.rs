use std::sync::mpsc::{self, Receiver, Sender};

#[derive(Debug)]
pub struct Io {
    pub lines: Vec<String>,
    pub buf: String,
    pub out_rx: Receiver<String>,
    pub out_tx: Sender<String>,
}

impl Io {
    pub fn new() -> Self {
        let (out_tx, out_rx) = mpsc::channel();

        Self {
            lines: vec![],
            buf: String::new(),
            out_rx,
            out_tx,
        }
    }

    pub fn reset(&mut self) {
        std::mem::take(&mut self.lines);
        std::mem::take(&mut self.buf);
    }

    pub fn update(&mut self) {
        while let Ok(chunk) = self.out_rx.try_recv() {
            for c in chunk.chars() {
                match c {
                    '\n' => self.lines.push(std::mem::take(&mut self.buf)),
                    _ => self.buf.push(c),
                }
            }
        }
    }
}
