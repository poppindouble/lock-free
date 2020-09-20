use std::boxed::Box;
use std::{thread, time};
use std::cell::RefCell;

struct LazyTransformer<S, V> {
    pub source: RefCell<Option<S>>,
    pub value: RefCell<Option<V>>,
    pub transform_fn: Box<dyn Fn(S) -> V>,
}

impl<S: Clone, V: Clone> LazyTransformer<S, V> {
    pub fn new(transform_fn: Box<dyn Fn(S) -> V>) -> Self {
        LazyTransformer {
            source: RefCell::new(None),
            value: RefCell::new(None),
            transform_fn,
        }
    }

    pub fn set_source(&self, source: S) {
        *self.source.borrow_mut() = Some(source);
    }

    pub fn get_transformed(&self) -> Option<V> {
        if let Some(source) =  &*self.source.borrow() {
            let value = (self.transform_fn)(source.clone());
            *self.value.borrow_mut() = Some(value);
        }
        *self.source.borrow_mut() = None;
        self.value.borrow().clone()
    }
}

fn main() {
    let transform_fn = Box::new(|sec| {
        let sec = time::Duration::from_secs(sec);
        thread::sleep(sec);
        println!("sleep for {:?}s.", sec);
        return sec;
    });
    let lazy_transformer = LazyTransformer::new(transform_fn);

    lazy_transformer.set_source(5);
    let value = lazy_transformer.get_transformed();

    println!("{:?}", value);
}
