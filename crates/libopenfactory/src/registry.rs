use std::{collections::HashMap, marker::PhantomData, ops::Index};

#[derive(Debug)]
pub struct Handle<T>(usize, PhantomData<T>);

// Cannot derive because derive requires that `T` be Clone as well.
impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self {
        Self(self.0, self.1)
    }
}

impl<T> Copy for Handle<T> {}

impl<T> PartialEq for Handle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T> Eq for Handle<T> {
    fn assert_receiver_is_total_eq(&self) {}
}

impl<T> ::std::hash::Hash for Handle<T> {
    fn hash<H: ::std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl<T> Handle<T> {
    fn new(ix: usize) -> Self {
        Self(ix, PhantomData)
    }
}

impl<T> Index<Handle<T>> for Vec<T> {
    type Output = <Vec<T> as Index<usize>>::Output;

    fn index(&self, index: Handle<T>) -> &Self::Output {
        &self[index.0]
    }
}

impl<T> Index<&'_ Handle<T>> for Vec<T> {
    type Output = <Vec<T> as Index<usize>>::Output;

    fn index(&self, index: &Handle<T>) -> &Self::Output {
        &self[index.0]
    }
}

pub struct Table<T> {
    list: Vec<T>,
    table: HashMap<String, Handle<T>>,
    inverse_table: HashMap<Handle<T>, String>,
}

impl<T> Table<T> {
    pub fn new() -> Self {
        Self {
            list: vec![],
            table: HashMap::new(),
            inverse_table: HashMap::new(),
        }
    }

    pub fn insert(&mut self, item: T, name: String) -> Handle<T> {
        self.list.push(item);
        let handle = Handle::new(self.list.len() - 1);
        self.table.insert(name.clone(), handle.clone());
        self.inverse_table.insert(handle.clone(), name);
        handle
    }

    pub fn name(&self, handle: &Handle<T>) -> &str {
        &self.inverse_table[handle]
    }

    pub fn get_handle_from_name(&self, name: &str) -> Handle<T> {
        self.table[name]
    }

    pub fn get_ref_and_handle_from_name(&self, name: &str) -> (&T, Handle<T>) {
        let handle = self.table[name];
        let value = &self.list[&handle];

        (value, handle)
    }

    // pub fn get_with_name_unchecked(&self, handle: &Handle<T>) -> (&str, &T) {
    //     (self.name(handle), &self[*handle])
    // }
}

impl<T> Index<Handle<T>> for Table<T> {
    type Output = <Vec<T> as Index<usize>>::Output;

    fn index(&self, index: Handle<T>) -> &Self::Output {
        &self.list[index]
    }
}

impl<T> Index<&'_ Handle<T>> for Table<T> {
    type Output = <Vec<T> as Index<usize>>::Output;

    fn index(&self, index: &Handle<T>) -> &Self::Output {
        &self.list[index]
    }
}

impl<T> Index<String> for Table<T> {
    type Output = <Vec<T> as Index<usize>>::Output;

    fn index(&self, index: String) -> &Self::Output {
        &self.list[self.table[&index]]
    }
}