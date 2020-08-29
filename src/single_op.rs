use std::cmp::Ordering::*;

#[derive(Eq, PartialEq, Debug, Clone)]
pub enum Op {
    Insert(usize, usize, u8),
    Delete(usize),
    Noop,
}
use Op::*;

pub type Doc = Vec<u8>;

pub fn apply(doc: &mut Doc, op: &Op) {
    match *op {
        Insert(index, _, c) => {
            doc.insert(index, c);
        }
        Delete(index) => {
            doc.remove(index);
        }
        Noop => {}
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
///
/// Satisfies TP1:
///
/// ```ignore
/// { apply(doc, op1); apply(doc, transform(op2, op1, Right)); }
/// ```
/// is equivalent to
///
/// ```ignore
/// { apply(doc, op2); apply(doc, transform(op1, op2, Left)); }
/// ```
pub fn transform(op1: &Op, op2: &Op, side: Side) -> Op {
    match *op1 {
        Insert(index, num_deletes, c) => {
            let mut num_deletes = num_deletes;
            let new_index = match *op2 {
                Insert(index2, num_deletes_2, _) => match ((index2 + num_deletes_2).cmp(&(index + num_deletes)), side) {
                    (Less, _) => index + 1,
                    (Equal, Left) => index,
                    (Equal, Right) => index + 1,
                    (Greater, _) => index,
                },
                Delete(index2) => {
                    if index2 < index {
                        num_deletes += 1;
                        index - 1
                    } else {
                        index
                    }
                }
                Noop => index,
            };
            Insert(new_index, num_deletes, c)
        }
        Delete(index) => {
            let new_index = match *op2 {
                Insert(index2, _, _) => {
                    if index2 <= index {
                        index + 1
                    } else {
                        index
                    }
                }
                Delete(index2) => {
                    match index2.cmp(&index) {
                        Less => index - 1,
                        Equal => {
                            // Both ops deleted the same character
                            return Noop;
                        }
                        Greater => index,
                    }
                }
                Noop => index,
            };
            Delete(new_index)
        }
        Noop => Noop,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_insert() {
        let mut doc = b"abc".to_vec();
        apply(&mut doc, &Insert(1, 0, b'x'));
        assert_eq!(doc, b"axbc");
    }

    #[test]
    fn test_apply_delete() {
        let mut doc = b"abc".to_vec();
        apply(&mut doc, &Delete(1));
        assert_eq!(doc, b"ac");
    }

    use proptest::prelude::*;

    fn valid_op_for(doc: &Doc) -> impl Strategy<Value = Op> {
        prop_oneof![
            // Note: we always generate 0 for num_deletes. The reasoning is: if two operations are
            // made against the same document, they should be affected by the same deletes, I
            // guess?
            //
            // But this could possibly break in the more general case, where we can generate new
            // operations from arbitrary fork points. We _could_ give them proper num_deletes, but
            // that would actually require tombstones...
            1 => (0..=doc.len(), any::<u8>()).prop_map(|(index, c)| Insert(index, 0, c)),
            (doc.len() > 0) as u32 => (0..doc.len()).prop_map(|index| Delete(index)),
        ]
    }

    fn doc_and_two_valid_ops() -> impl Strategy<Value = (Doc, Op, Op)> {
        any::<Doc>().prop_flat_map(|doc| {
            (valid_op_for(&doc), valid_op_for(&doc))
                .prop_map(move |(op1, op2)| (doc.clone(), op1, op2))
        })
    }

    fn doc_and_3_valid_ops() -> impl Strategy<Value = (Doc, Op, Op, Op)> {
        any::<Doc>().prop_flat_map(|doc| {
            (valid_op_for(&doc), valid_op_for(&doc), valid_op_for(&doc))
                .prop_map(move |(op1, op2, op3)| (doc.clone(), op1, op2, op3))
        })
    }

    proptest! {
        #[test]
        fn transform_property_1((doc, op1, op2) in doc_and_two_valid_ops()) {
            let mut doc1 = doc.clone();
            let transformed_op2 = transform(&op2, &op1, Right);
            apply(&mut doc1, &op1);
            apply(&mut doc1, &transformed_op2);

            let mut doc2 = doc.clone();
            let transformed_op1 = transform(&op1, &op2, Left);
            apply(&mut doc2, &op2);
            apply(&mut doc2, &transformed_op1);

            prop_assert_eq!(doc1, doc2, "\ntransformed_op1 = {:?},\ntransformed_op2 = {:?}\n", transformed_op1, transformed_op2);
        }

        #[test]
        fn transform_property_2((doc, op1, op2, op3) in doc_and_3_valid_ops()) {
            let mut doc1 = doc.clone();
            let transformed_op2 = transform(&op2, &op1, Right);
            apply(&mut doc1, &op1);
            apply(&mut doc1, &transformed_op2);

            let mut doc2 = doc.clone();
            let transformed_op1 = transform(&op1, &op2, Left);
            apply(&mut doc2, &op2);
            apply(&mut doc2, &transformed_op1);

            let op3_transformed_by_1_2 = transform(&transform(&op3, &op1, Right), &transformed_op2, Right);
            apply(&mut doc1, &op3_transformed_by_1_2);
            let op3_transformed_by_2_1 = transform(&transform(&op3, &op2, Right), &transformed_op1, Right);
            apply(&mut doc2, &op3_transformed_by_2_1);

            prop_assert_eq!(
                doc1,
                doc2,
                "\nops1 = {:?}\nops2 = {:?}\n",
                &[op1, transformed_op2, op3_transformed_by_1_2],
                &[op2, transformed_op1, op3_transformed_by_2_1],
            );
        }
    }
}
