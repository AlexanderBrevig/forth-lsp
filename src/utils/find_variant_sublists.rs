use std::mem::{discriminant, Discriminant};

#[allow(dead_code)]
pub trait FindVariantSublists<T> {
    fn find_variant_sublists(&self, find: Vec<Discriminant<T>>) -> Vec<&[T]>;
}

impl<T> FindVariantSublists<T> for Vec<T> {
    fn find_variant_sublists(&self, find: Vec<Discriminant<T>>) -> Vec<&[T]> {
        let mut ret = vec![];
        for i in 0..self.len() {
            let disc = (0..find.len())
                .filter(|j| i + j < self.len())
                .all(|j| discriminant(&self[i + j]) == find[j]);
            if disc {
                ret.push(&self[i..(i + find.len())])
            }
        }
        ret
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq, PartialOrd)]
    enum Test {
        One(&'static str),
        Two(&'static str),
        Three(&'static str),
        Four(&'static str),
    }

    #[test]
    fn test() {
        use Test::*;
        let enums = vec![
            One("one"),
            Two("two"),
            Three("three"),
            Four("four"),
            One("1"),
            Two("2"),
        ];
        let results = enums.find_variant_sublists(vec![
            discriminant(&Test::One("")),
            discriminant(&Test::Two("")),
        ]);
        let expected: Vec<&[Test]> = vec![&enums[0..2], &enums[4..6]];
        assert_eq!(expected, results);
    }
}
