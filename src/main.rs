use std::boxed::Box;
use std::{thread, time};
use std::sync::Mutex;

#[derive(Clone)]
struct Status<S, V> {
    pub source: Option<S>,
    pub value: Option<V>,
}

impl<S, V> Status<S, V> {
    pub fn new() -> Self {
        Status {
            source: None,
            value: None,
        }
    }
}

#[derive(Clone)]
struct LazyTransformer<S, V, FN> {
    pub status: Mutex<Status<S, V>>,
    pub transform_fn: FN,
}

impl<S: Clone, V: Clone, FN: Fn(S) -> V> LazyTransformer<S, V, FN> {
    pub fn new(transform_fn: FN) -> Self {
        LazyTransformer {
            status: Mutex::new(Status::new()),
            transform_fn,
        }
    }

    pub fn set_source(&self, source: S) {
        let mut status = self.status.lock().unwrap();
        status.source = Some(source);
    }

    pub fn get_transformed(&self) -> Option<V> {
        let mut status = self.status.lock().unwrap();
        if let Some(source) = &status.source {
            let value = (self.transform_fn)(source.clone());
            status.value = Some(value.clone());
            status.source = None;
        }
        return status.value.clone();
    }
}

fn main() {
    let transform_fn = Box::new(|sec| {
        let sec = time::Duration::from_secs(sec);
        thread::sleep(sec);
        println!("sleep for {:?}s.", sec);
        return sec;
    });
    let mut lazy_transformer = LazyTransformer::new(transform_fn);
    let mut lazy_clone = lazy_transformer.clone();
    thread::spawn(move || {
        lazy_clone.set_source(5);
    }).join();

    let value = lazy_transformer.get_transformed();

    println!("{:?}", value);
}
