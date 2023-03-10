use std::{
    collections::VecDeque,
    num::NonZeroUsize,
    ops::{Deref, DerefMut},
    sync::mpsc::{self, Receiver, Sender},
};

#[derive(Debug)]
pub struct ParBuf<T> {
    vec: VecDeque<T>,
    rx: Receiver<T>,
    pub tx: Sender<T>,
    limit: Option<NonZeroUsize>,
}

impl<T> ParBuf<T> {
    pub fn new() -> ParBuf<T> {
        let (tx, rx) = mpsc::channel::<T>();
        ParBuf {
            vec: VecDeque::new(),
            rx,
            tx,
            limit: None,
        }
    }

    /// Set the limit of the parallel buffer.
    /// Will panic if the limit is zero.
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(NonZeroUsize::new(limit).expect("non-zero ParBuf limit"));
        self
    }

    pub fn update(&mut self) {
        while let Ok(item) = self.rx.try_recv() {
            if let Some(limit) = self.limit {
                while self.vec.len() >= limit.into() {
                    self.vec.pop_front();
                }
            }

            self.vec.push_back(item);
        }
    }
}

impl<T> Default for ParBuf<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Deref for ParBuf<T> {
    type Target = VecDeque<T>;

    fn deref(&self) -> &Self::Target {
        &self.vec
    }
}

impl<T> DerefMut for ParBuf<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.vec
    }
}
