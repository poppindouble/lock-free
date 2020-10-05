use crate::cell::Cell;

struct RcInner<T> {
    value: T,
    refcount: Cell<u64>,
}

pub struct Rc<T> {
    inner: *const RcInner<T>,
}

impl<T> Rc<T> {
    pub fn new(v: T) -> Self {
        let inner = Box::new(RcInner {
            value: v,
            refcount: Cell::new(1),
        });

        Rc {
            // SAFETY: Box does not give us a null pointer.
            inner: Box::into_raw(inner),
        }
    }
}

impl<T> std::ops::Deref for Rc<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        // SAFETY: self.inner is a Box that is only deallocated when the last Rc goes away.
        // we have an Rc, therefore the Box has not been deallocated, so deref is fine.
        unsafe { &(*self.inner).value }
    }
}

impl<T> Clone for Rc<T> {
    fn clone(&self) -> Self {
        unsafe {
            let c = (*self.inner).refcount.get();
            (*self.inner).refcount.set(c + 1);
        }
        Rc { inner: self.inner }
    }
}

impl<T> Drop for Rc<T> {
    fn drop(&mut self) {
        let c = unsafe { (*self.inner).refcount.get() };
        if c == 1 {
            // SAFETY: we are the _only_ Rc left, and we are being dropped.
            // therefore, after us, there will be no Rc's, and no references to T.
            let mut_inner = self.inner as *mut RcInner<T>;
            let _ = unsafe { Box::from_raw(mut_inner) };
        } else {
            // there are other Rcs, so don't drop the Box!
            let mut_inner = self.inner as *mut RcInner<T>;
            unsafe {
                (*mut_inner).refcount.set(c - 1);
            }
        }
    }
}
