#[derive(Debug)]
pub struct Io {
    pub lines: Vec<String>,
    pub buf: String,
    pub in_buf: String,
    pub out_tx: crossbeam::channel::Sender<String>,
    pub out_rx: crossbeam::channel::Receiver<String>,
    pub in_tx: crossbeam::channel::Sender<String>,
    pub in_rx: crossbeam::channel::Receiver<String>,
}

impl Io {
    pub fn new() -> Self {
        let (out_tx, out_rx) = crossbeam::channel::unbounded();
        let (in_tx, in_rx) = crossbeam::channel::unbounded();

        Self {
            lines: vec![],
            buf: String::new(),
            in_buf: String::new(),
            out_rx,
            out_tx,
            in_tx,
            in_rx,
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
