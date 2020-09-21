use std::boxed::Box;
use std::{thread, time};
use std::sync::{Mutex, Arc};

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
    pub status: Arc<Mutex<Status<S, V>>>,
    pub transform_fn: FN,
}

impl<S: Clone, V: Clone, FN: Fn(S) -> V> LazyTransformer<S, V, FN> {
    pub fn new(transform_fn: FN) -> Self {
        LazyTransformer {
            status: Arc::new(Mutex::new(Status::new())),
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
    let transform_fn = Box::new(|hold_val| {
        let sec = time::Duration::from_secs(5);
        println!("executing transform for {:?}.", sec);
        thread::sleep(sec);
        return hold_val;
    });
    let lazy_transformer = LazyTransformer::new(transform_fn);
    let mut handles = vec![];

    for i in 0..1000 {
        let lazy_clone = lazy_transformer.clone();
        let handle = thread::spawn(move || {
            let sec = time::Duration::from_millis(100 * i);
            thread::sleep(sec);
            let value = lazy_clone.get_transformed();
            println!("getting value {:?}", value);
        });
        handles.push(handle);
    }

    println!("launched all readers");

    for i in 0..10 {
        let lazy_clone = lazy_transformer.clone();
        let handle = thread::spawn(move || {
            let sec = time::Duration::from_secs(i);
            thread::sleep(sec);
            println!("setting source {:?}", i);
            lazy_clone.set_source(i);
        });
        handles.push(handle);
    }

    println!("launched all setters");

    for handle in handles {
        handle.join().unwrap();
    }
}
