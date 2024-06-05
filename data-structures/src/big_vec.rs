//! # BigVec
//!
//! is a data structure that represents a vector capable of dynamically growing without the overhead reallocating memory
//! each time the vector resizes and without memory size limit.
//! It internally manages a collection of smaller vectors, enabling efficient insertion and deletion operations.

use scrypto::prelude::*;
use std::cmp::min;
use std::mem::size_of;
use std::ops::{Deref, DerefMut};
use std::vec::IntoIter;

pub trait BigVecElement:
    ScryptoEncode + ScryptoDecode + ScryptoDescribe + Categorize<ScryptoCustomValueKind>
{
}
impl<T: ScryptoEncode + ScryptoDecode + ScryptoDescribe + Categorize<ScryptoCustomValueKind>>
    BigVecElement for T
{
}

#[derive(ScryptoSbor)]
#[sbor(categorize_types = "V")]
pub struct BigVec<V: BigVecElement> {
    pub start_index: usize,
    pub capacity_per_vec: usize,
    pub vec_structure: Vec<usize>,
    pub vec_data: KeyValueStore<usize, Vec<V>>,
}

impl<V: BigVecElement> BigVec<V> {
    /// Constructs a new, empty `BigVec<V>`.
    pub fn new() -> Self {
        Self {
            start_index: 0,
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
            start_index: 0,
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
            self.vec_data.insert(self.start_index, vec![element]);
        } else {
            let vec_length = self.vec_structure.len();
            if self.vec_structure[vec_length - 1] == self.capacity_per_vec {
                self.vec_structure.push(1);
                self.vec_data
                    .insert(vec_length + self.start_index, vec![element]);
            } else {
                self.vec_structure[vec_length - 1] += 1;
                let mut data = self
                    .vec_data
                    .get_mut(&(vec_length + self.start_index - 1))
                    .unwrap();
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

    /// Retrieves an immutable reference to an item in the `BigVec`.
    ///
    /// This method takes a reference to an index and returns an `Option` containing a
    /// reference to the item at the specified index, if it exists. If the index is out
    /// of bounds,, it returns `None`.
    ///
    /// # Arguments
    ///
    /// * `index` - A reference to the index of the item to retrieve.
    ///
    /// # Returns
    ///
    /// An `Option` containing a reference to the item at the specified index, if it exists.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use data_structures::big_vec::BigVec;
    ///
    /// let mut big_vec: BigVec<i32> = BigVec::new();
    /// big_vec.push(10);
    ///
    /// assert_eq!(big_vec.get(&0).as_deref(), Some(&10));
    /// assert_eq!(big_vec.get(&1).as_deref(), None);
    /// ```
    pub fn get(&self, index: &usize) -> Option<BigVecItemRef<'_, V>> {
        self.get_correct_indexes(index)
            .map(|indexes| BigVecItemRef::new(self.vec_data.get(&indexes.0).unwrap(), indexes.1))
    }

    /// Retrieves a mutable reference to an item in the `BigVec`.
    ///
    /// This method takes a reference to an index and returns an `Option` containing a
    /// mutable reference to the item at the specified index, if it exists. If the index
    /// is out of bounds, it returns `None`.
    ///
    /// # Arguments
    ///
    /// * `index` - A reference to the index of the item to retrieve.
    ///
    /// # Returns
    ///
    /// An `Option` containing a mutable reference to the item at the specified index, if it exists.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use data_structures::big_vec::BigVec;
    ///
    /// let mut big_vec: BigVec<i32> = BigVec::new();
    /// big_vec.push(10);
    ///
    /// if let Some(mut item) = big_vec.get_mut(&0) {
    ///     *item = 20;
    /// }
    ///
    /// assert_eq!(big_vec.get(&0).as_deref(), Some(&20));
    /// ```
    pub fn get_mut(&mut self, index: &usize) -> Option<BigVecItemRefMut<'_, V>> {
        match self.get_correct_indexes(index) {
            None => None,
            Some(indexes) => Some(BigVecItemRefMut::new(
                self.vec_data.get_mut(&indexes.0).unwrap(),
                indexes.1,
            )),
        }
    }

    /// Inserts an element at a specified index in the `BigVec`.
    ///
    /// # Safety
    ///
    /// This method is marked as unsafe because it allows inserting elements into the `BigVec` at a
    /// specific index, which could potentially lead to exceed the fee limit during a transaction.
    /// The caller is responsible for ensuring that inserting elements using this method does
    /// not result exceeding fee limits.
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
    /// unsafe { big_vec.insert(1, 2); }
    /// assert_eq!(big_vec.pop(), Some(3));
    /// assert_eq!(big_vec.pop(), Some(2));
    /// assert_eq!(big_vec.pop(), Some(1));
    /// ```
    pub unsafe fn insert(&mut self, index: usize, element: V) {
        let mut data_index = index / self.capacity_per_vec;
        let vec_index = index % self.capacity_per_vec;

        if data_index > self.vec_structure.len()
            || (data_index == self.vec_structure.len() && vec_index > 0)
            || (data_index + 1 == self.vec_structure.len()
                && vec_index >= *self.vec_structure.last().unwrap())
        {
            panic!("Trying to insert to index {index} which is out of bounds!")
        }

        data_index += self.start_index;

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

    /// Removes and returns the first vector of elements from a `BigVec`.
    ///
    /// This method removes and returns the first vector of elements from the `BigVec`.
    /// If the `BigVec` is empty, it returns `None`.
    ///
    /// # Returns
    ///
    /// An `Option` containing the first vector of elements from the `BigVec`, if it exists.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use data_structures::big_vec::BigVec;
    ///
    /// let mut big_vec: BigVec<i32> = BigVec::with_capacity_per_vec(2);
    /// big_vec.push(1);
    /// big_vec.push(2);
    /// big_vec.push(3);
    ///
    /// assert_eq!(big_vec.pop_first_vec(), Some(vec![1,2]));
    /// assert_eq!(big_vec.pop_first_vec(), Some(vec![3]));
    /// assert_eq!(big_vec.pop_first_vec(), None)
    /// ```
    pub fn pop_first_vec(&mut self) -> Option<Vec<V>> {
        match self.vec_structure.first() {
            None => None,
            Some(_) => {
                self.vec_structure.remove(0);
                let ret = self.vec_data.remove(&self.start_index);
                self.start_index += 1;
                ret
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
    /// use data_structures::big_vec::BigVec;
    ///
    /// let mut big_vec = BigVec::<i32>::new();
    /// let elements = vec![1, 2, 3, 4, 5];
    /// big_vec.push_vec(elements);
    /// ```
    pub fn push_vec(&mut self, mut elements: Vec<V>) {
        let start_index = self.vec_structure.len();
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
                }
            }
        }

        self.push_vec_raw(elements);
    }

    /// Pushes elements from a vector into a BigVec, organizing them into sub-vectors based on the configured capacity.
    /// It does it without trying to fill the current last vector.
    ///
    /// # Arguments
    ///
    /// * `elements`: A vector of elements to be pushed into the internal data structure.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use data_structures::big_vec::BigVec;
    ///
    /// let mut big_vec = BigVec::<i32>::new();
    /// let elements = vec![1, 2, 3, 4, 5, 6];
    /// big_vec.push_vec_raw(elements);
    /// ```
    pub fn push_vec_raw(&mut self, mut elements: Vec<V>) {
        let mut start_index = self.vec_structure.len();

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
    /// use data_structures::big_vec::BigVec;
    ///
    /// let mut big_vec1 = BigVec::<i32>::new();
    /// let mut big_vec2 = BigVec::<i32>::new();
    /// big_vec1.append(big_vec2);
    /// ```
    pub fn append(&mut self, mut other: Self) {
        if other.capacity_per_vec == self.capacity_per_vec {
            let mut index_to_push = self.vec_structure.len();
            self.vec_structure.append(&mut other.vec_structure);
            for i in 0..other.vec_structure.len() {
                let vec_data = other.vec_data.remove(&i).unwrap();
                self.vec_data.insert(index_to_push, vec_data);
                index_to_push += 1;
            }
        } else {
            panic!("Cannot append from a BigVec with a different structure")
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

    fn get_correct_indexes(&self, index: &usize) -> Option<(usize, usize)> {
        let data_index = *index / self.capacity_per_vec;
        let vec_index = *index % self.capacity_per_vec;

        // If we exceeded the size return None
        if data_index >= self.vec_structure.len()
            || (data_index + 1 == self.vec_structure.len()
                && vec_index >= *self.vec_structure.last().unwrap())
        {
            None
        } else {
            Some((data_index + self.start_index, vec_index))
        }
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

impl<V: BigVecElement> Default for BigVec<V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<V: BigVecElement> From<Vec<V>> for BigVec<V> {
    fn from(vec: Vec<V>) -> Self {
        let mut big_vec = BigVec::new();
        big_vec.push_vec(vec);
        big_vec
    }
}

pub struct BigVecItemRef<'a, V: BigVecElement> {
    sub_vec: KeyValueEntryRef<'a, Vec<V>>,
    item_index: usize,
}

impl<'a, V: BigVecElement> BigVecItemRef<'a, V> {
    pub fn new(sub_vec: KeyValueEntryRef<'a, Vec<V>>, item_index: usize) -> Self {
        Self {
            sub_vec,
            item_index,
        }
    }
}

impl<'a, V: BigVecElement> Deref for BigVecItemRef<'a, V> {
    type Target = V;

    fn deref(&self) -> &Self::Target {
        self.sub_vec.get(self.item_index).unwrap()
    }
}

pub struct BigVecItemRefMut<'a, V: BigVecElement> {
    sub_vec: KeyValueEntryRefMut<'a, Vec<V>>,
    item_index: usize,
}
impl<'a, V: BigVecElement> BigVecItemRefMut<'a, V> {
    pub fn new(sub_vec: KeyValueEntryRefMut<'a, Vec<V>>, item_index: usize) -> Self {
        Self {
            sub_vec,
            item_index,
        }
    }
}

impl<'a, V: BigVecElement> Deref for BigVecItemRefMut<'a, V> {
    type Target = V;

    fn deref(&self) -> &Self::Target {
        self.sub_vec.get(self.item_index).unwrap()
    }
}

impl<'a, V: BigVecElement> DerefMut for BigVecItemRefMut<'a, V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.sub_vec.get_mut(self.item_index).unwrap()
    }
}

pub struct BigVecIntoIterator<'a, V: BigVecElement + Clone> {
    pub number_of_vec: usize,
    pub current_vec: usize,
    pub current_vec_iterator: IntoIter<V>,
    pub vec_data: &'a KeyValueStore<usize, Vec<V>>,
}
impl<'a, V: BigVecElement + Clone> IntoIterator for &'a BigVec<V> {
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

impl<'a, V: BigVecElement + Clone> Iterator for BigVecIntoIterator<'a, V> {
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
                            panic!("The iterator is wrongly formed! Could not find element at index {}", self.current_vec);
                        }
                        Some(vec) => <Vec<V> as Clone>::clone(&vec).into_iter(),
                    };

                    self.current_vec_iterator.next()
                }
            }
        }
    }
}
