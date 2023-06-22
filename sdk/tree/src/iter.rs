use crate::error::{TreeError, TreeResult};
use crate::tree::{Read, KV};
use std::cmp::Ordering;
use std::collections::btree_map;
use std::ops::{Bound, RangeBounds};

pub struct Iter<'a, S> {
    parent: &'a S,
    bounds: (Bound<Vec<u8>>, Bound<Vec<u8>>),
    done: bool,
}

impl<'a, S: Read> Iter<'a, S> {
    /// Creates a new iterator over entries in `parent` in the given range
    /// bounds.
    pub fn new(parent: &'a S, bounds: (Bound<Vec<u8>>, Bound<Vec<u8>>)) -> Self {
        Iter {
            parent,
            bounds,
            done: false,
        }
    }
}

impl<'a, S: Read> Iterator for Iter<'a, S> {
    type Item = TreeResult<KV>;

    fn next(&mut self) -> Option<TreeResult<KV>> {
        if self.done {
            return None;
        }

        let maybe_entry = match self.bounds.0 {
            // if entry exists at empty key, emit that. if not, get next entry
            Bound::Unbounded => self.parent.get_next_inclusive(&[]).transpose(),

            // if entry exists at given key, emit that. if not, get next entry
            Bound::Included(ref key) => self.parent.get_next_inclusive(key).transpose(),

            // get next entry
            Bound::Excluded(ref key) => self.parent.get_next(key).transpose(),
        };

        match maybe_entry {
            // bubble up errors
            Some(Err(err)) => Some(Err(err)),

            // got entry
            Some(Ok((key, value))) => {
                // entry is past end of range, mark iterator as done
                if !self.bounds.contains(&key) {
                    self.done = true;
                    return None;
                }

                // advance internal state to next key
                self.bounds.0 = Bound::Excluded(key.clone());
                Some(Ok((key, value)))
            }

            // reached end of iteration, mark iterator as done
            None => {
                self.done = true;
                None
            }
        }
    }
}

pub fn prefix_iterator<D: Read>(d: &D, prefix: Vec<u8>, key: Vec<u8>) -> TreeResult<Option<KV>> {
    let len = prefix.len();
    let prefix_slice = prefix.as_slice();
    let prefixed = concat(prefix_slice, key.as_slice());
    let res = d
        .get_next(prefixed.as_slice())?
        .filter(|(k, _)| k.starts_with(prefix_slice))
        .map(|(k, v)| (k[len..].into(), v));
    Ok(res)
}

pub fn exclusive_range_from(start: &[u8]) -> (Bound<Vec<u8>>, Bound<Vec<u8>>) {
    (Bound::Excluded(start.to_vec()), Bound::Unbounded)
}

pub fn iter_merge_next<'a, S: Read>(
    map_iter: &mut btree_map::Range<Vec<u8>, Option<Vec<u8>>>,
    store_iter: &mut Box<dyn Iterator<Item = TreeResult<KV>> + 'a>,
) -> TreeResult<Option<KV>> {
    let mut map_iter = map_iter.peekable();
    let mut store_iter = store_iter.peekable();

    loop {
        let has_map_entry = map_iter.peek().is_some();
        let has_backing_entry = store_iter.peek().is_some();

        return Ok(match (has_map_entry, has_backing_entry) {
            (false, false) => None,

            (true, false) => match map_iter.next().unwrap() {
                (key, Some(value)) => Some((key.clone(), value.clone())),
                (_, None) => continue,
            },

            (false, true) => store_iter.next().transpose()?,

            (true, true) => {
                let map_key = map_iter.peek().unwrap().0;
                let backing_key = match store_iter.peek().unwrap() {
                    Err(_) => return Err(TreeError::Store("Backing key does not exist".into())),
                    Ok((ref key, _)) => key,
                };
                let key_cmp = map_key.cmp(backing_key);

                if key_cmp == Ordering::Greater {
                    let entry = store_iter.next().unwrap()?;
                    return Ok(Some(entry));
                }

                if key_cmp == Ordering::Equal {
                    store_iter.next();
                }

                match map_iter.next().unwrap() {
                    (key, Some(value)) => Some((key.clone(), value.clone())),
                    (_, None) => continue,
                }
            }
        });
    }
}

#[inline]
pub fn concat(a: &[u8], b: &[u8]) -> Vec<u8> {
    let mut value = Vec::with_capacity(a.len() + b.len());
    value.extend_from_slice(a);
    value.extend_from_slice(b);
    value
}
