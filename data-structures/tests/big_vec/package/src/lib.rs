use data_structures::big_vec::BigVec;
use scrypto::prelude::*;

#[blueprint]
mod big_vec {
    struct BigVecContract {
        vec: BigVec<u32>,
    }

    impl BigVecContract {
        pub fn new() -> Global<BigVecContract> {
            Self { vec: BigVec::new() }
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .globalize()
        }

        pub fn with_capacity_per_vec(capacity_per_vec: usize) -> Global<BigVecContract> {
            Self {
                vec: BigVec::with_capacity_per_vec(capacity_per_vec),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .globalize()
        }

        pub fn push(&mut self, element: u32) {
            self.vec.push(element);
        }

        pub fn pop(&mut self) -> Option<u32> {
            self.vec.pop()
        }

        pub fn insert(&mut self, index: usize, element: u32) {
            self.vec.insert(index, element);
        }

        pub fn len(&self) -> usize {
            self.vec.len()
        }

        pub fn is_empty(&self) -> bool {
            self.vec.is_empty()
        }

        pub fn vec_nb(&self) -> usize {
            self.vec.vec_nb()
        }

        pub fn structure(&self) -> Vec<usize> {
            self.vec.structure().clone()
        }

        pub fn capacity_per_vec(&self) -> usize {
            self.vec.capacity_per_vec()
        }

        pub fn internal_representation(&self) -> Vec<Vec<u32>> {
            self.vec.internal_representation()
        }

        pub fn full_vec(&self) -> Vec<u32> {
            let mut ret = vec![];
            for elem in &self.vec {
                ret.push(elem);
            }
            ret
        }
    }
}
