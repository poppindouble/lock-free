use crate::cell::Cell;
use std::cell::UnsafeCell;
use std::ops::{Deref, DerefMut};

#[derive(Copy, Clone)]
enum RefState {
    Unshared,
    Shared(i32),
    Exclusive,
}

pub struct RefGuard<'a, T> {
    refcell: &'a RefCell<T>,
}

impl<'a, T> RefGuard<'a, T> {
    pub fn new(value: &'a RefCell<T>) -> Self {
        RefGuard { refcell: value }
    }
}

impl<'a, T> Drop for RefGuard<'a, T> {
    fn drop(&mut self) {
        match self.refcell.state.get() {
            RefState::Exclusive | RefState::Unshared => unreachable!(),
            RefState::Shared(1) => {
                self.refcell.state.set(RefState::Unshared);
            }
            RefState::Shared(shared) => {
                self.refcell.state.set(RefState::Shared(shared - 1));
            }
        }
    }
}

impl<'a, T> Deref for RefGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        let ptr = self.refcell.value.get();
        unsafe { &*ptr }
    }
}

pub struct MutRefGuard<'a, T> {
    refcell: &'a RefCell<T>,
}

impl<'a, T> MutRefGuard<'a, T> {
    pub fn new(value: &'a RefCell<T>) -> Self {
        MutRefGuard { refcell: value }
    }
}

impl<'a, T> Drop for MutRefGuard<'a, T> {
    fn drop(&mut self) {
        match self.refcell.state.get() {
            RefState::Exclusive => {
                self.refcell.state.set(RefState::Unshared);
            }
            _ => unreachable!(),
        }
    }
}

impl<'a, T> Deref for MutRefGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        let ptr = self.refcell.value.get();
        unsafe { &*ptr }
    }
}

impl<'a, T> DerefMut for MutRefGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        let ptr = self.refcell.value.get();
        unsafe { &mut *ptr }
    }
}

pub struct RefCell<T> {
    value: UnsafeCell<T>,
    state: Cell<RefState>,
}

impl<T> RefCell<T> {
    pub fn new(value: T) -> Self {
        RefCell {
            value: UnsafeCell::new(value),
            state: Cell::new(RefState::Unshared),
        }
    }

    pub fn borrow(&self) -> Option<RefGuard<T>> {
        match self.state.get() {
            RefState::Unshared => {
                let state = RefState::Shared(1);
                self.state.set(state);
                return Some(RefGuard::new(self));
            }
            RefState::Shared(shared) => {
                self.state.set(RefState::Shared(shared + 1));
                return Some(RefGuard::new(&self));
            }
            RefState::Exclusive => None,
        }
    }

    pub fn borrow_mut(&self) -> Option<MutRefGuard<T>> {
        match self.state.get() {
            RefState::Unshared => {
                self.state.set(RefState::Exclusive);

                return Some(MutRefGuard::new(&self));
            }
            _ => None,
        }
    }
}
