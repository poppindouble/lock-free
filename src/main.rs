use std::boxed::Box;
use std::{thread, time};

trait Transformer<S, V> {
    fn set_source(&mut self, source: S);
    fn get_transformed(&mut self) -> Option<V>;
}

struct LazyTransformer<S, V> {
    pub source: Option<S>,
    pub value: Option<V>,
    pub transform_fn: Box<dyn Fn(S) -> V>,
}

impl<S: Clone, V: Clone> LazyTransformer<S, V> {
    pub fn new(transform_fn: Box<dyn Fn(S) -> V>) -> Self {
        LazyTransformer {
            source: None,
            value: None,
            transform_fn,
        }
    }
}

impl<S: Clone, V: Clone> Transformer<S, V> for LazyTransformer<S, V> {
    fn set_source(&mut self, source: S) {
        self.source = Some(source);
    }

    fn get_transformed(&mut self) -> Option<V> {
        if let Some(source) = &self.source {
            let value = (self.transform_fn)(source.clone());
            self.value = Some(value.clone());
            self.source = None;
            return Some(value);
        }
        return self.value.clone();
    }
}

fn main() {
    let transform_fn = Box::new(|sec| {
        let sec = time::Duration::from_secs(sec);
        thread::sleep(sec);
        println!("sleep for {:?}.", sec);
        return sec;
    });
    let mut lazy_transformer = LazyTransformer::new(transform_fn);

    lazy_transformer.set_source(5);
    let value = lazy_transformer.get_transformed();

    println!("{:?}", value);
}
