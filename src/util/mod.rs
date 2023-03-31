use bevy::prelude::{Component, ReflectComponent};
use bevy::reflect::Reflect;



pub(crate) mod scripting;




#[derive(Reflect, Default, Debug, Component)]
#[reflect(Component)]
pub struct Buffer {
    pub vec: Vec<u32>,
    head: usize, 
}

impl Buffer {
    pub fn with_capacity(capacity: usize) -> Buffer {
        Self {
            vec: vec![0; capacity],
            head: 0,
        }
    }

    pub fn get(&self, index: usize) -> Option<&u32> {
        let raw = self.head + index;
        let new_index = if raw > (self.vec.len() - 1) {
            0
        }
        else {
            raw
        };
        self.vec.get(new_index)
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut u32> {
        let raw = self.head + index;
        let new_index = if raw > (self.vec.len() - 1) {
            0
        }
        else {
            raw
        };
        self.vec.get_mut(new_index)
    }

    pub fn insert(&mut self, value: u32) {
        self.head = self.head.checked_sub(1).unwrap_or(self.vec.len() - 1);
        if let Some(head) = self.get_mut(0) {
            *head = value;
        }
        else {
            panic!()
        }
    }

    pub fn iter(&self) -> BufferIter {
        let ring = self.vec.as_slice();
        let head = self.head;
        let tail = self.head.checked_sub(1).unwrap_or(self.vec.len() - 1);

        BufferIter::new(ring, tail, head)
    }
}



pub struct BufferIter<'a> {
    ring: &'a[u32],
    tail: usize,
    head: usize,
    check: bool
}

impl<'a> BufferIter<'a> {
    pub fn new(ring: &'a[u32], tail: usize, head: usize) -> Self {
        Self { ring, tail, head, check: false }
    }
}

impl<'a> Iterator for BufferIter<'a> {
    type Item = &'a u32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.check {
            return None
        }
        if self.tail == self.head  {
            self.check = true;
        }

        let head = self.head;
        self.head = if self.head < self.ring.len() - 1 {
            self.head + 1
        }
        else {
           0
        };

        self.ring.get(head)
    }
}