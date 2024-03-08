//! # BigVec
//!
//! is a data structure that represents a vector capable of dynamically growing without the overhead reallocating memory
//! each time the vector resizes and without memory size limit.
//! It internally manages a collection of smaller vectors, enabling efficient insertion and deletion operations.

use scrypto::prelude::*;
use std::cmp::min;
use std::mem::size_of;
use std::vec::IntoIter;

#[derive(ScryptoSbor)]
pub struct BigVec<
    V: ScryptoEncode + ScryptoDecode + ScryptoDescribe + Categorize<ScryptoCustomValueKind>,
> {
    capacity_per_vec: usize,
    vec_structure: Vec<usize>,
    vec_data: KeyValueStore<usize, Vec<V>>,
}

impl<V: ScryptoEncode + ScryptoDecode + ScryptoDescribe + Categorize<ScryptoCustomValueKind>>
    BigVec<V>
{
    /// Constructs a new, empty `BigVec<V>`.
    pub fn new() -> Self {
        Self {
            capacity_per_vec: 1_000_000 / size_of::<V>(),
            vec_structure: Vec::new(),
            vec_data: KeyValueStore::new(),
        }
    }

    /// Creates a new empty `BigVec` with a specified initial capacity for each internal vector.
    ///
    /// This function initializes a new `BigVec` with the specified `elements_per_vec` as the initial capacity for each
    /// internal vector. This can be useful for optimizing memory usage when the approximate number of elements per
    /// vector is known in advance.
    ///
    /// # Arguments
    ///
    /// * `capacity_per_vec` - The amount of elements to store in each internal vector.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use data_structures::big_vec::BigVec;
    ///
    /// // Create a new BigVec with an initial capacity of 100 elements per vector
    /// let big_vec: BigVec<i32> = BigVec::with_capacity_per_vec(100);
    /// ```
    pub fn with_capacity_per_vec(capacity_per_vec: usize) -> Self {
        Self {
            capacity_per_vec,
            vec_structure: Vec::new(),
            vec_data: KeyValueStore::new(),
        }
    }

    /// Appends an element to the end of the `BigVec`.
    ///  # Examples
    ///
    /// ```no_run
    /// use data_structures::big_vec::BigVec;
    ///
    /// let mut big_vec: BigVec<i32> = BigVec::new();
    ///
    /// big_vec.push(42);
    /// assert_eq!(big_vec.len(), 1);
    /// ```
    pub fn push(&mut self, element: V) {
        if self.vec_structure.is_empty() {
            self.vec_structure.push(1);
            self.vec_data.insert(0, vec![element]);
        } else {
            let vec_length = self.vec_structure.len();
            if self.vec_structure[vec_length - 1] == self.capacity_per_vec {
                self.vec_structure.push(1);
                self.vec_data.insert(vec_length, vec![element]);
            } else {
                self.vec_structure[vec_length - 1] += 1;
                let mut data = self.vec_data.get_mut(&(vec_length - 1)).unwrap();
                data.push(element);
            }
        }
    }

    /// Removes the last element from the `BigVec` and returns it, or `None` if it is empty.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use data_structures::big_vec::BigVec;
    ///
    /// let mut big_vec: BigVec<i32> = BigVec::new();
    ///
    /// big_vec.push(42);
    /// assert_eq!(big_vec.pop(), Some(42));
    /// assert_eq!(big_vec.pop(), None);
    /// ```
    pub fn pop(&mut self) -> Option<V> {
        if self.vec_structure.is_empty() {
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

    /// Inserts an element at a specified index in the `BigVec`.
    ///
    /// # Panics
    ///
    /// Panics if the index is out of bounds.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use data_structures::big_vec::BigVec;
    ///
    /// let mut big_vec: BigVec<i32> = BigVec::new();
    ///
    /// big_vec.push(1);
    /// big_vec.push(3);
    /// big_vec.insert(1, 2);
    /// assert_eq!(big_vec.pop(), Some(3));
    /// assert_eq!(big_vec.pop(), Some(2));
    /// assert_eq!(big_vec.pop(), Some(1));
    /// ```
    pub fn insert(&mut self, index: usize, element: V) {
        let mut data_index: usize = 0;
        let mut vec_index = index;

        // First get the correct vec and position where to insert
        for items_nb in &self.vec_structure {
            if vec_index > *items_nb {
                vec_index -= items_nb;
                data_index += 1;
            } else {
                break;
            }
        }

        // If we exceeded the size, panic
        if (data_index > self.vec_structure.len() && vec_index > 0)
            || vec_index > *self.vec_structure.last().unwrap() + 1
        {
            panic!("Trying to insert to index {index} which is out of bounds!")
        }

        // If we are trying to insert at last position, push item
        if self.vec_structure.get(data_index).is_none() {
            self.vec_structure.push(1);
            self.vec_data.insert(data_index, vec![element]);
            return;
        }

        // Otherwise, insert the item
        let mut data = self
            .vec_data
            .get_mut(&data_index)
            .expect("Something is wrong with this BigVec");

        data.insert(vec_index, element);

        if data.len() <= self.capacity_per_vec {
            *self.vec_structure.get_mut(data_index).unwrap() += 1;
            return;
        }

        let mut to_push = data.pop().unwrap();
        let mut index_to_push = data_index + 1;
        std::mem::drop(data);

        // Restructure everything if needed
        loop {
            match self.vec_structure.get(index_to_push) {
                None => {
                    self.vec_structure.push(1);
                    self.vec_data.insert(index_to_push, vec![to_push]);
                    return;
                }
                Some(amount) => {
                    let mut new_data = self
                        .vec_data
                        .get_mut(&index_to_push)
                        .expect("Something is wrong with this BigVec");

                    new_data.insert(0, to_push);

                    if *amount < self.capacity_per_vec {
                        *self.vec_structure.get_mut(index_to_push).unwrap() += 1;
                        return;
                    } else {
                        to_push = new_data.pop().unwrap();
                        index_to_push += 1;
                    }
                }
            }
        }
    }

    /// Pushes elements from a vector into a BigVec, organizing them into sub-vectors based on the configured capacity.
    /// If the last sub-vector in the BigVec has space remaining, the elements are appended to it.
    /// If not, a new sub-vector is created and the elements are divided accordingly.
    ///
    /// # Arguments
    ///
    /// * `elements` - A vector containing elements of type `V` to be pushed into the BigVec.
    ///
    /// # Example
    ///
    /// ```no_run
    /// let mut big_vec = BigVec::<i32>::new();
    /// let elements = vec![1, 2, 3, 4, 5];
    /// big_vec.push_vec(elements);
    /// ```
    pub fn push_vec(&mut self, mut elements: Vec<V>) {
        let mut start_index = self.vec_structure.len();
        match self.vec_structure.last() {
            None => {}
            Some(vec_size) => {
                let elems_to_push = self.capacity_per_vec - *vec_size;
                let last_index = self.vec_structure.len() - 1;
                if elems_to_push > elements.len() {
                    self.vec_structure[last_index] += elements.len();
                    self.vec_data
                        .get_mut(&(start_index - 1))
                        .unwrap()
                        .append(&mut elements);
                    return;
                } else {
                    self.vec_structure[last_index] = self.capacity_per_vec;
                    self.vec_data
                        .get_mut(&(start_index - 1))
                        .unwrap()
                        .append(&mut elements.drain(..elems_to_push).collect());
                    start_index += 1;
                }
            }
        }

        while !elements.is_empty() {
            let to_drain = min(elements.len(), self.capacity_per_vec);
            let new_elems: Vec<V> = elements.drain(..to_drain).collect();
            self.vec_structure.push(new_elems.len());
            self.vec_data.insert(start_index, new_elems);
            start_index += 1;
        }
    }

    /// Lazily appends the contents of another BigVec to this BigVec.
    /// The sub-vectors and their sizes from the other BigVec are appended to the end of the current BigVec.
    ///
    /// # Arguments
    ///
    /// * `other` - Another BigVec whose contents are to be appended to this BigVec.
    ///
    /// # Example
    ///
    /// ```no_run
    /// let mut big_vec1 = BigVec::<i32>::new();
    /// let mut big_vec2 = BigVec::<i32>::new();
    /// big_vec1.append(big_vec2);
    /// ```
    pub fn append(&mut self, mut other: Self) {
        let mut index_to_push = self.vec_structure.len();
        self.vec_structure.append(&mut other.vec_structure);
        for i in 0..other.vec_structure.len() {
            let vec_data = other.vec_data.remove(&i).unwrap();
            self.vec_data.insert(index_to_push, vec_data);
            index_to_push += 1;
        }
    }

    /// Returns the number of elements in the `BigVec`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use data_structures::big_vec::BigVec;
    ///
    /// let mut big_vec: BigVec<i32> = BigVec::new();
    ///
    /// big_vec.push(1);
    /// big_vec.push(2);
    /// assert_eq!(big_vec.len(), 2);
    /// ```
    pub fn len(&self) -> usize {
        self.vec_structure.iter().sum()
    }

    /// Returns `true` if the `BigVec` is empty, otherwise `false`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use data_structures::big_vec::BigVec;
    ///
    /// let mut big_vec: BigVec<i32> = BigVec::new();
    ///
    /// assert!(big_vec.is_empty());
    /// big_vec.push(1);
    /// assert!(!big_vec.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.vec_structure.is_empty()
    }

    /// Returns the number of vectors internally managed by the `BigVec`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use data_structures::big_vec::BigVec;
    ///
    /// let mut big_vec: BigVec<i32> = BigVec::new();
    ///
    /// assert_eq!(big_vec.vec_nb(), 0);
    /// big_vec.push(1);
    /// assert_eq!(big_vec.vec_nb(), 1);
    /// ```
    pub fn vec_nb(&self) -> usize {
        self.vec_structure.len()
    }

    /// Returns the internal structure of the `BigVec`.
    pub fn structure(&self) -> &Vec<usize> {
        &self.vec_structure
    }

    /// Returns the capacity per vec of the `BigVec`.
    pub fn capacity_per_vec(&self) -> usize {
        self.capacity_per_vec
    }
}

impl<
        V: ScryptoEncode
            + ScryptoDecode
            + ScryptoDescribe
            + Categorize<ScryptoCustomValueKind>
            + Clone,
    > BigVec<V>
{
    /// Returns all the value in the underlying representation.
    /// Should only be used for tests.
    pub fn internal_representation(&self) -> Vec<Vec<V>> {
        let mut ret = vec![];
        for i in 0..self.vec_structure.len() {
            ret.push(self.vec_data.get(&i).unwrap().clone());
        }
        ret
    }
}

impl<V: ScryptoEncode + ScryptoDecode + ScryptoDescribe + Categorize<ScryptoCustomValueKind>>
    Default for BigVec<V>
{
    fn default() -> Self {
        Self::new()
    }
}

impl<V: ScryptoEncode + ScryptoDecode + ScryptoDescribe + Categorize<ScryptoCustomValueKind>>
    From<Vec<V>> for BigVec<V>
{
    fn from(vec: Vec<V>) -> Self {
        let mut big_vec = BigVec::new();
        big_vec.push_vec(vec);
        big_vec
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
                if self.current_vec + 1 >= self.number_of_vec {
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
