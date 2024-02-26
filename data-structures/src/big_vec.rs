use std::mem::size_of;
use std::ops::Deref;
use radix_engine_common::prelude::{ScryptoDecode, ScryptoDescribe, ScryptoEncode};
use scrypto::prelude::{KeyValueStore, ScryptoSbor};

#[derive(ScryptoSbor)]
pub struct BigVec<V: ScryptoEncode + ScryptoDecode + ScryptoDescribe>{
    items_per_vec: usize,
    vec_structure: Vec<usize>,
    vec_data: KeyValueStore<usize, Vec<V>>
}

impl<V: ScryptoEncode + ScryptoDecode + ScryptoDescribe> BigVec<V>{

    pub fn new() -> Self {
        Self{
            items_per_vec: 1_000_000/size_of::<V>(),
            vec_structure: Vec::new(),
            vec_data: KeyValueStore::new()
        }
    }

    pub fn push(&mut self, item: V){
        if self.vec_structure.len() == 0{
            self.vec_structure.push(1);
            self.vec_data.insert(0, vec![item]);
        }
        else{
            if self.vec_structure[self.vec_structure.len() - 1] == self.items_per_vec{
                self.vec_structure.push(1);
                self.vec_data.insert(self.vec_structure.len() - 1, vec![item]);
            }
            else{
                self.vec_structure[self.vec_structure.len() - 1] += 1;
                let mut data = self.vec_data.get_mut(&(self.vec_structure.len() - 1)).unwrap();
                data.push(item);
            }
        }
    }

    pub fn pop(&mut self) -> Option<V>{
        if self.vec_structure.len() == 0{
            None
        }
        else{
            self.vec_structure[self.vec_structure.len() - 1] -= 1;
            if self.vec_structure[self.vec_structure.len() - 1] == 0 {
                let mut data = self.vec_data.remove(&(self.vec_structure.len() - 1)).unwrap();
                self.vec_structure.pop();
                data.pop()
            }
            else{
                let mut data = self.vec_data.get_mut(&(self.vec_structure.len() - 1)).unwrap();
                data.pop()
            }
        }
    }
}