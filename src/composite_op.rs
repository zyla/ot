#![allow(warnings)]

use std::cmp::Ordering::*;

pub type Doc = Vec<u8>;

pub type Chunk = Vec<u8>;

#[derive(Eq, PartialEq, Debug, Clone)]
pub enum Step {
    Skip(usize),
    Insert(Chunk),
    Delete(usize),
}
use Step::*;

type Op = Vec<Step>;

pub fn apply(doc: &mut Doc, op: &[Step]) {
    let mut index = 0;
    for step in op {
        match step {
            Skip(n) => {
                index += n;
            }
            Insert(s) => {
                let old_doc_len = doc.len();
                doc.resize(old_doc_len + s.len(), 0);
                doc.copy_within(index..old_doc_len, index + s.len());
                doc[index..(index + s.len())].copy_from_slice(&s);
                index += s.len()
            }
            Delete(n) => {
                doc.drain(index..(index + n));
                index += n;
            }
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
pub enum Side {
    Left,
    Right,
}

use Side::*;

/// Takes two operations defined on the same initial document,
/// and returns an operation equivalent to `op1` which can be applied after `op2`.
pub fn transform(op1: &Op, op2: &Op, side: Side) -> Op {
    vec![] // TODO
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_insert() {
        let mut doc = b"abc".to_vec();
        apply(&mut doc, &[Skip(1), Insert(b"xyz".to_vec())]);
        assert_eq!(doc, b"axyzbc");
    }

    #[test]
    fn test_apply_delete() {
        let mut doc = b"abcd".to_vec();
        apply(&mut doc, &[Skip(1), Delete(2)]);
        assert_eq!(doc, b"ad");
    }
}
