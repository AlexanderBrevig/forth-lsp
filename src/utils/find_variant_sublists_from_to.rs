use std::mem::{Discriminant, discriminant};

pub trait FindVariantSublistsFromTo<T> {
    fn find_variant_sublists_from_to(
        &self,
        from: Discriminant<T>,
        to: Discriminant<T>,
    ) -> Vec<&[T]>;
}

impl<T> FindVariantSublistsFromTo<T> for Vec<T> {
    fn find_variant_sublists_from_to(
        &self,
        from: Discriminant<T>,
        to: Discriminant<T>,
    ) -> Vec<&[T]> {
        let mut ret = vec![];
        for i in 0..self.len() {
            if discriminant(&self[i]) == from {
                for j in (i + 1)..self.len() {
                    if discriminant(&self[j]) == to {
                        ret.push(&self[i..(j + 1)]);
                        break;
                    }
                }
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
            Three("3"),
            Four("4"),
        ];
        let results = enums.find_variant_sublists_from_to(
            discriminant(&Test::Two("")),
            discriminant(&Test::Four("")),
        );
        let expected: Vec<&[Test]> = vec![&enums[1..4], &enums[5..8]];
        assert_eq!(expected, results);
    }
}
