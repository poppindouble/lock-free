use std::cell::UnsafeCell;

pub struct Cell<T> {
    value: UnsafeCell<T>,
}

impl<T> Cell<T> {
    pub fn new(value: T) -> Self {
        Cell {
            value: UnsafeCell::new(value),
        }
    }

    pub fn set(&self, new_value: T) {
        let ptr = self.value.get();
        // SAFETY: It is single thread because of !Sync of UnsafeCell, so no one is mutating self.value at the same time.
        // SAFETY: we know we're not invalidating any references, because we never give any out
        unsafe { *ptr = new_value };
    }

    pub fn get(&self) -> T
    where
        T: Copy,
    {
        let ptr = self.value.get();
        // SAFETY: It is single thread because of !Sync of UnsafeCell, so no one is mutating self.value at the same time.
        unsafe { *ptr }
    }
}
