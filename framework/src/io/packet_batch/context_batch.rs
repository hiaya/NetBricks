use super::act::Act;
use super::Batch;
use super::iterator::BatchIterator;
use super::super::pmd::*;
use super::super::interface::Result;
use std::default::Default;
use std::any::Any;

pub struct ContextBatch<T, V>
    where T: Any + Default + Clone,
          V: Batch + BatchIterator + Act
{
    parent: V,
    context: Vec<T>,
    context_size: usize,
}

impl<T, V> ContextBatch<T, V>
    where T: Any + Default + Clone,
          V: Batch + BatchIterator + Act
{
    pub fn new(parent: V) -> ContextBatch<T, V> {
        let capacity = parent.capacity() as usize;
        ContextBatch {
            parent: parent,
            context: vec![Default::default(); capacity],
            context_size: capacity,
        }
    }
}

impl<T, V> Batch for ContextBatch<T, V>
    where T: Any + Default + Clone,
          V: Batch + BatchIterator + Act
{
}

impl<T, V> Act for ContextBatch<T, V>
    where T: Any + Default + Clone,
          V: Batch + BatchIterator + Act
{
    #[inline]
    fn act(&mut self) {
        self.parent.act();
    }

    #[inline]
    fn done(&mut self) {
        self.context = vec![Default::default(); self.context_size];
        self.parent.done();
    }

    #[inline]
    fn send_queue(&mut self, port: &mut PmdPort, queue: i32) -> Result<u32> {
        self.parent.send_queue(port, queue)
    }

    #[inline]
    fn capacity(&self) -> i32 {
        self.parent.capacity()
    }

    #[inline]
    fn drop_packets(&mut self, idxes: Vec<usize>) -> Option<usize> {
        // Need to adjust data
        let mut idx_orig = self.parent.start();
        let mut idx_new = 0;
        let mut remove_idx = 0;
        let end = self.context.len();

        // First go through the list of indexes to be filtered and get rid of them.
        while idx_orig < end && (remove_idx < idxes.len()) {
            let test_idx = idxes[remove_idx];
            assert!(idx_orig <= test_idx);
            if idx_orig == test_idx {
                remove_idx += 1;
            } else {
                self.context.swap(idx_orig, idx_new);
                idx_new += 1;
            }
            idx_orig += 1;
        }
        // Then copy over any left over packets.
        while idx_orig < end {
            self.context.swap(idx_orig, idx_new);
            idx_orig += 1;
            idx_new += 1;
        }
        self.parent.drop_packets(idxes)
    }
}

impl<T, V> BatchIterator for ContextBatch<T, V>
    where T: Any + Default + Clone,
          V: Batch + BatchIterator + Act
{
    #[inline]
    fn start(&mut self) -> usize {
        self.parent.start()
    }

    // FIXME: Really we should be accepting a token (capability) here and only adding context if the token matches.
    #[inline]
    unsafe fn next_address(&mut self, idx: usize) -> Option<(*mut u8, Option<&mut Any>, usize)> {
        match self.parent.next_address(idx) {
            Some((addr, _, iret)) => {
                Some((addr,
                      self.context.get_mut(idx).and_then(|x| Some(x as &mut Any)),
                      iret))
            }
            None => None,
        }
    }

    #[inline]
    unsafe fn next_payload(&mut self, idx: usize) -> Option<(*mut u8, *mut u8, usize, Option<&mut Any>, usize)> {
        match self.parent.next_payload(idx) {
            Some((haddr, paddr, psize, _, iret)) => {
                Some((haddr,
                      paddr,
                      psize,
                      self.context.get_mut(idx).and_then(|x| Some(x as &mut Any)),
                      iret))
            }
            None => None,
        }
    }

    #[inline]
    unsafe fn next_base_address(&mut self, idx: usize) -> Option<(*mut u8, Option<&mut Any>, usize)> {
        match self.parent.next_base_address(idx) {
            Some((addr, _, iret)) => {
                Some((addr,
                      self.context.get_mut(idx).and_then(|x| Some(x as &mut Any)),
                      iret))
            }
            None => None,
        }
    }

    #[inline]
    unsafe fn next_base_payload(&mut self, idx: usize) -> Option<(*mut u8, *mut u8, usize, Option<&mut Any>, usize)> {
        match self.parent.next_base_payload(idx) {
            Some((haddr, paddr, psize, _, iret)) => {
                Some((haddr,
                      paddr,
                      psize,
                      self.context.get_mut(idx).and_then(|x| Some(x as &mut Any)),
                      iret))
            }
            None => None,
        }
    }
}