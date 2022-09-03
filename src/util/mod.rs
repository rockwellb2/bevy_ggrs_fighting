
use std::ops::Index;

use bevy::reflect::FromReflect;
use bevy::reflect::Reflect;

pub(crate) mod scripting;

#[derive(Reflect, Default, Debug)]
pub struct Buffer<T: Reflect + FromReflect> {
    pub vec: Vec<T>,
    head: usize, 
    tail: usize,
    capacity: usize,
}

impl<T: Reflect + FromReflect + Default> Buffer<T> {
    pub fn with_capacity(capacity: usize) -> Buffer<T> {
        let vec = Vec::with_capacity(capacity);
        let head = 0;
        let tail = capacity - 1;

        Self {
            vec,
            head,
            tail,
            capacity,
        }
    }

    pub fn from_vec(vec: Vec<T>) -> Buffer<T> {
        let head = 0;
        let tail = vec.len() - 1;
        let capacity = vec.capacity();

        Self {
            vec,
            head,
            tail,
            capacity
        }
    }

    pub fn push(&mut self, value: T) {
        if self.capacity != self.vec.len() {
            self.vec.push(value);
            self.tail += 1;
        }
        else {
            if let Some(elem) = self.vec.get_mut(self.head) {
                *elem = value;

                if self.head < self.capacity - 1 {
                    self.head += 1;
                }
                else {
                    self.head = 0;
                }

                if self.tail < self.capacity - 1 {
                    self.tail += 1;
                }
                else {
                    self.tail = 0;
                }
            }
        }
        

    }

    pub fn last(&self) -> Option<&T> {
        self.vec.get(self.tail)
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        let baseline = self.head + index;
        if let Some(overflow) = baseline.checked_sub(self.capacity) {
            self.vec.get(overflow)
        }
        else {
            self.vec.get(baseline)
        }
    }



}

impl<T: Reflect + FromReflect + Default> Index<usize> for Buffer<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        let baseline = self.head + index;
        if let Some(overflow) = baseline.checked_sub(self.capacity) {
            &self.vec[overflow]
        }
        else {
            &self.vec[baseline]
        }
    }
}

pub struct BufferIter<'a, T: Reflect + FromReflect + Default> {
    buffer: &'a Buffer<T>,
    index: usize,
    check: bool
}

impl<'a, T: Reflect + FromReflect + Default> IntoIterator for &'a Buffer<T> {
    type Item = &'a T;
    type IntoIter = BufferIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
       BufferIter {
        buffer: self,
        index: self.tail,
        check: false
       }
    }
}



impl<'a, T: Reflect + FromReflect + Default> Iterator for BufferIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.check {
            return None
        }
        
        let n = self.buffer.get(self.index);
        if self.index == self.buffer.head {
            self.check = true;
        }
        else if self.index == 0 {
            self.index = self.buffer.capacity - 1;
        }
        else {
            self.index -= 1;
        }

        return n

    }
}

#[cfg(test)]
pub mod tests {
    use super::Buffer;

    #[test]
    pub fn index_check() {
        let vector: Vec<i32> = vec![1, 2, 3];
        let buffer = Buffer::from_vec(vector);

        assert_eq!(buffer[0], 1);
        assert_eq!(buffer[2], 3);

    }

    #[test]
    fn push_test() {
        let vector: Vec<i32> = vec![4, 3, 6, 7, 10];
        let mut buffer = Buffer::from_vec(vector);
        buffer.push(1);
        assert_eq!(buffer[0], 3);
    }

    #[test]
    fn with_capacity_test() {
        let mut buffer: Buffer<i32> = Buffer::with_capacity(10);
        buffer.push(5);

        assert_eq!(buffer[0], 5);

        for n in 0..9 {
            println!("N is equal to {}", n);
            buffer.push(n);
            println!("Vec: {:?}", buffer.vec);
        }



        assert_eq!(buffer[9], 8);
        buffer.push(11);
        println!("Vec: {:?}", buffer.vec);

        assert_eq!(buffer[0], 0);
        buffer.push(-3);
        assert_eq!(buffer[9], -3);
        assert_eq!(buffer[0], 1);

    }

    #[test]
    fn iter_test() {
        let buffer = Buffer::from_vec(vec![0, 1, 2]);
        let iter = &mut buffer.into_iter();

       assert_eq!(Some(&2), iter.next());
       assert_eq!(Some(&1), iter.next());
       assert_eq!(Some(&0), iter.next());
       assert_eq!(None, iter.next());


       
    }

}