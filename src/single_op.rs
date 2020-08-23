use std::cmp::Ordering::*;

#[derive(Eq, PartialEq, Debug, Clone)]
pub enum Op {
    Insert(usize, u8),
    Delete(usize),
    Noop,
}
use Op::*;

pub type Doc = Vec<u8>;

pub fn apply(doc: &mut Doc, op: &Op) {
    match *op {
        Insert(index, c) => {
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
        Insert(index, c) => {
            let new_index = match *op2 {
                Insert(index2, _) => match (index2.cmp(&index), side) {
                    (Less, _) => index + 1,
                    (Equal, Left) => index,
                    (Equal, Right) => index + 1,
                    (Greater, _) => index,
                },
                Delete(index2) => {
                    if index2 < index {
                        index - 1
                    } else {
                        index
                    }
                }
                Noop => index,
            };
            Insert(new_index, c)
        }
        Delete(index) => {
            let new_index = match *op2 {
                Insert(index2, _) => {
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
        apply(&mut doc, &Insert(1, b'x'));
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
            1 => (0..=doc.len(), any::<u8>()).prop_map(|(index, c)| Insert(index, c)),
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
        #[ignore = "Turns out we don't actually satisfy TP2."]
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
