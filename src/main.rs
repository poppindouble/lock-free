use std::{thread, time};
use std::sync::{Arc, Mutex, RwLock};

#[derive(Clone)]
struct LazyTransformer<S, V, FN: Fn(S) -> V> {
    pub source: Arc<Mutex<Option<S>>>,
    pub value: Arc<RwLock<Option<V>>>,
    pub transform_fn: FN,
}

impl<S: Clone+Copy, V: Clone+Copy, FN: Fn(S) -> V> LazyTransformer<S, V, FN> {
    pub fn new(transform_fn: FN) -> Self {
        LazyTransformer {
            source: Arc::new(Mutex::new(None)),
            value: Arc::new(RwLock::new(None)),
            transform_fn,
        }
    }

    pub fn set_source(&self, source: S) {
        let mut source_guard = self.source.lock().unwrap();
        *source_guard = Some(source);
    }

    pub fn get_transformed(&self) -> Option<V> {
        let mut source_guard = self.source.lock().unwrap();
        if let Some(source) = &*source_guard {
            let new_value = (self.transform_fn)(source.clone());
            let mut value_write_guard = self.value.write().unwrap();
            *value_write_guard = Some(new_value);
            *source_guard = None;
            return Some(new_value);
        } else {
            return *self.value.read().unwrap();
        }
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
