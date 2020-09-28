use crate::cell::Cell;
use std::cell::UnsafeCell;

#[derive(Copy, Clone)]
enum RefStatus {
    Unshared,
    Shared(i32),
    Exclusive,
}

pub struct RefCell<T> {
    value: UnsafeCell<T>,
    state: Cell<RefStatus>,
}

impl<T> RefCell<T> {
    pub fn new(value: T) -> Self {
        RefCell {
            value: UnsafeCell::new(value),
            state: Cell::new(RefStatus::Unshared),
        }
    }

    pub fn borrow(&self) -> Option<&T> {
        match self.state.get() {
            RefStatus::Unshared => {
                let state = RefStatus::Shared(1);
                self.state.set(state);

                unsafe {
                    let ptr = self.value.get();
                    let reference = &*ptr;
                    return Some(reference);
                }
            }
            RefStatus::Shared(shared) => {
                self.state.set(RefStatus::Shared(shared + 1));

                unsafe {
                    let ptr = self.value.get();
                    let reference = &*ptr;
                    return Some(reference);
                }
            }
            RefStatus::Exclusive => None,
        }
    }

    pub fn borrow_mut(&self) -> Option<&mut T> {
        match self.state.get() {
            RefStatus::Unshared => {
                self.state.set(RefStatus::Unshared);

                unsafe {
                    let ptr = self.value.get();
                    let mut_reference = &mut *ptr;
                    return Some(mut_reference);
                }
            },
            _ => None,
        }
    }
}
