use std::{
    collections::BTreeMap,
    io::{self, Read, Seek, SeekFrom, Write},
};

/// The length of a single block.
pub const BLOCK_SIZE: usize = 256;
pub const ADDR_MEM_MAX: usize = 0x100000000;
pub const ADDR_STACK_TOP: usize = 0x80000000;
pub const ADDR_HEAP: usize = 0x10008000;
pub const ADDR_STATIC: usize = 0x10000000;
pub const ADDR_TEXT: usize = 0x00400000;

type Block = [u8; BLOCK_SIZE];

#[derive(Debug)]
pub struct Memory {
    tree: BTreeMap<usize, Block>,
    pos: usize,
}

impl Memory {
    pub fn new() -> Self {
        Self {
            tree: BTreeMap::new(),
            pos: 0,
        }
    }

    pub fn pos(&self) -> usize {
        self.pos
    }

    pub fn set_pos(&mut self, pos: usize) {
        self.pos = pos;
    }

    pub fn skip(&mut self, bytes: usize) {
        self.pos += bytes;
    }

    pub fn align(&mut self, bytes: usize) {
        self.pos += bytes - self.pos % bytes;
    }

    /// Get all of the blocks across boundaries, starting at an address, lasting some amount of bytes.
    fn blocks(&self, start_addr: usize, size: usize) -> Vec<(&usize, &Block)> {
        self.tree
            .range((start_addr - BLOCK_SIZE + 1)..=(start_addr + size))
            .collect()
    }

    /// Get all of the block addresses that contain the start address and the size.
    fn block_addrs(&self, start_addr: usize, size: usize) -> Vec<usize> {
        let mut addrs = vec![start_addr / BLOCK_SIZE * BLOCK_SIZE];

        while addrs[addrs.len() - 1] + BLOCK_SIZE < start_addr + size {
            addrs.push(addrs[addrs.len() - 1] + BLOCK_SIZE);
        }

        addrs
    }
}

// TODO: this Seek impl may need to be moved into a new struct,
// TODO: something like a MemoryView<'a> so multiple threads
// TODO: can seek to different places in the memory at the
// TODO: same time

impl Seek for Memory {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        match pos {
            SeekFrom::Current(delta) => self.pos = (self.pos as i64 + delta) as usize,
            SeekFrom::Start(pos) => self.pos = pos as usize,
            SeekFrom::End(delta) => self.pos = (ADDR_MEM_MAX as i64 - delta) as usize,
        }

        Ok(self.pos as u64)
    }
}

impl Read for Memory {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let addr = self.pos;
        let len = buf.len();
        let blocks = self.blocks(addr, len);
        let mut read = 0;

        for (base_addr, block) in blocks {
            if *base_addr > addr && read < base_addr - addr {
                read = base_addr - addr;
            }
            let local_addr = addr.saturating_sub(*base_addr);
            let slice = &block[local_addr..(local_addr + len - read)];
            let (_, buf_slice) = buf.split_at_mut(read);
            let (left, _) = buf_slice.split_at_mut(slice.len());
            left.copy_from_slice(slice);
            read += slice.len();
        }

        self.pos += read;
        Ok(read)
    }
}

impl Write for Memory {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let addr = self.pos;
        let len = buf.len();
        let mut written = 0;

        for base_addr in self.block_addrs(addr, len) {
            let block = self
                .tree
                .entry(base_addr)
                .or_insert_with(|| [0u8; BLOCK_SIZE]);

            let start_addr = addr.saturating_sub(base_addr);
            let end_addr = (start_addr + len - written).min(BLOCK_SIZE);
            let slice = &buf[written..(written + end_addr - start_addr)];
            let (left, _) = block.split_at_mut(end_addr);
            let (_, inner) = left.split_at_mut(start_addr);
            inner.copy_from_slice(slice);
            written += slice.len();
        }

        self.pos += written;
        Ok(written)
    }

    fn flush(&mut self) -> io::Result<()> {
        // Noop. This is in-memory.
        Ok(())
    }
}
