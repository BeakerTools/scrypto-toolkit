use scrypto::prelude::*;

#[blueprint]
mod big_vec {
    use data_structures::big_vec;
    use data_structures::big_vec::BigVec;

    struct BigVecContract {
        vec: BigVec<u32>,
    }

    impl BigVecContract {
        /// /// /// /// /// /// /// /// /// /// /// /// /// /// /// /// /// /// /// /// /// /// ///
        ///                                                                                     ///
        ///  Interface to the `BigVec` data structures for methods that can actually be called  ///
        ///                                                                                     ///
        /// /// /// /// /// /// /// /// /// /// /// /// /// /// /// /// /// /// /// /// /// /// ///

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

        pub fn default() -> Global<BigVecContract> {
            Self {
                vec: BigVec::default(),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .globalize()
        }

        pub fn from(vec: Vec<u32>) -> Global<BigVecContract> {
            Self {
                vec: BigVec::from(vec),
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

        pub fn push_vec(&mut self, elements: Vec<u32>) {
            self.vec.push_vec(elements);
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

        /// /// /// /// /// /// /// /// /// ///  /// ///
        ///                                          ///
        ///  Specific methods for testing purposes   ///
        ///                                          ///
        /// /// /// /// /// /// /// /// /// /// //// ///

        pub fn with_macros() -> Global<BigVecContract> {
            Self {
                vec: big_vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9],
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .globalize()
        }

        pub fn append(&mut self, address: ComponentAddress) {
            let other_big_vec: Global<AnyComponent> = address.into();
            let big_vec = other_big_vec.call("to_big_vec", &None::<u32>);
            self.vec.append(big_vec);
        }

        pub fn full_vec(&self) -> Vec<u32> {
            let mut ret = vec![];
            for elem in &self.vec {
                ret.push(elem);
            }
            ret
        }

        pub fn to_big_vec(&self, _fake: Option<u32>) -> BigVec<u32> {
            BigVec::from(self.vec.into_iter().collect::<Vec<u32>>())
        }
    }
}
