use radix_engine_common::prelude::{
    ScryptoCustomValueKind, ScryptoDecode, ScryptoDescribe, ScryptoEncode,
};
use sbor::Categorize;
use scrypto::prelude::{KeyValueStore, ScryptoSbor};
use std::mem::size_of;
use std::vec::IntoIter;

#[derive(ScryptoSbor)]
pub struct BigVec<
    V: ScryptoEncode + ScryptoDecode + ScryptoDescribe + Categorize<ScryptoCustomValueKind>,
> {
    items_per_vec: usize,
    vec_structure: Vec<usize>,
    vec_data: KeyValueStore<usize, Vec<V>>,
}

impl<V: ScryptoEncode + ScryptoDecode + ScryptoDescribe + Categorize<ScryptoCustomValueKind>>
    BigVec<V>
{
    pub fn new() -> Self {
        Self {
            items_per_vec: 1_000_000 / size_of::<V>(),
            vec_structure: Vec::new(),
            vec_data: KeyValueStore::new(),
        }
    }

    pub fn push(&mut self, element: V) {
        if self.vec_structure.len() == 0 {
            self.vec_structure.push(1);
            self.vec_data.insert(0, vec![element]);
        } else {
            let vec_length = self.vec_structure.len();
            if self.vec_structure[vec_length - 1] == self.items_per_vec {
                self.vec_structure.push(1);
                self.vec_data.insert(vec_length - 1, vec![element]);
            } else {
                self.vec_structure[vec_length - 1] += 1;
                let mut data = self.vec_data.get_mut(&(vec_length - 1)).unwrap();
                data.push(element);
            }
        }
    }

    pub fn pop(&mut self) -> Option<V> {
        if self.vec_structure.len() == 0 {
            None
        } else {
            let vec_length = self.vec_structure.len();
            self.vec_structure[vec_length - 1] -= 1;
            if self.vec_structure[vec_length - 1] == 0 {
                let mut data = self.vec_data.remove(&(vec_length - 1)).unwrap();
                self.vec_structure.pop();
                data.pop()
            } else {
                let mut data = self.vec_data.get_mut(&(vec_length - 1)).unwrap();
                data.pop()
            }
        }
    }

    pub fn insert(&mut self, mut index: usize, element: V) {
        let mut data_index: usize = 0;
        for items_nb in &self.vec_structure {
            if index > *items_nb {
                index -= items_nb
            } else {
                let mut data = self
                    .vec_data
                    .get_mut(&data_index)
                    .expect("Something is wrong with this BigVec");

                data.insert(index, element);
                self.vec_structure.insert(data_index, items_nb + 1);
                return;
            }
            data_index += 1;
        }

        panic!("Trying to insert to index {index} which is out of bounds!")
    }

    pub fn len(&self) -> usize {
        self.vec_structure.iter().sum()
    }

    pub fn is_empty(&self) -> bool {
        self.vec_structure.is_empty()
    }

    pub fn vec_nb(&self) -> usize {
        self.vec_structure.len()
    }
}

pub struct BigVecIntoIterator<
    'a,
    V: ScryptoEncode + ScryptoDecode + ScryptoDescribe + Categorize<ScryptoCustomValueKind> + Clone,
> {
    pub number_of_vec: usize,
    pub current_vec: usize,
    pub current_vec_iterator: IntoIter<V>,
    pub vec_data: &'a KeyValueStore<usize, Vec<V>>,
}
impl<
        'a,
        V: ScryptoEncode
            + ScryptoDecode
            + ScryptoDescribe
            + Categorize<ScryptoCustomValueKind>
            + Clone,
    > IntoIterator for &'a BigVec<V>
{
    type Item = V;
    type IntoIter = BigVecIntoIterator<'a, V>;

    fn into_iter(self) -> Self::IntoIter {
        let current_vec = match self.vec_data.get(&0) {
            None => Vec::new(),
            Some(vec) => vec.clone(),
        };
        Self::IntoIter {
            number_of_vec: self.vec_structure.len(),
            current_vec: 0,
            current_vec_iterator: current_vec.into_iter(),
            vec_data: &self.vec_data,
        }
    }
}

impl<
        'a,
        V: ScryptoEncode
            + ScryptoDecode
            + ScryptoDescribe
            + Categorize<ScryptoCustomValueKind>
            + Clone,
    > Iterator for BigVecIntoIterator<'a, V>
{
    type Item = V;

    fn next(&mut self) -> Option<Self::Item> {
        match self.current_vec_iterator.next() {
            Some(item) => Some(item),
            None => {
                if self.current_vec == self.number_of_vec {
                    None
                } else {
                    self.current_vec += 1;
                    self.current_vec_iterator = match self.vec_data.get(&self.current_vec) {
                        None => {
                            panic!("The iterator is wrongly formed")
                        }
                        Some(vec) => <Vec<V> as Clone>::clone(&vec).into_iter(),
                    };

                    self.current_vec_iterator.next()
                }
            }
        }
    }
}
