# Explore Rust Lock Free Programming

This series blog is a learning path of lock-free programing in Rust. We will start with something really simple, and we start to optimize our simple project, eventually we will reach the point of lock-free programming(currently I don't have a flame graph or benchmark to give out a concrete number about performance, because this blog mainly focus on learning experience). As we go along the path, some other contents besides lock-free programming will also be introduced. If I made any mistakes in the following contents, please let me know. My Email: poppindouble@gmail.com

## Contents
1. [Let's start with something simple](#lets-start-with-something-simple)
2. [Before we step our toes into optimization](#before-we-optimize-our-code)
3. [First attempt for optimization](#first-attempt-for-optimization)
4. [Second attempt for optimization](#second-attempt-for-optimization)
5. [Foundation of lock-free programming](#foundation-of-lock-free-programming)
6. [Atomic](#atomic)
7. [Memory barrier](#memory-barrier)
8. [What is a lock?](#what-is-a-lock)
9. [Third attempt for optimization](#third-attempt-for-optimization)
10. [Memory management](#memory-management)

## Let's start with something simple

### Description
Assume we need to implement a lazy transformer, the transformer has 3 fields, 
1. A transform function, `transform_fn`, which takes one parameter `source` and does some complicated calculation based on the value of `source`, then this function will return a `new_value`. 
2. Upper user can set the `source` field, which is used for the transform function to do complicated calculation.
3. `cached_value`, once the `new_value` has been calculated in sept 1, we cache this `new_value` to `cached_value`.

Once a new `source` has been set, transformer will calculate the `new_value` and return the `new_value` for the following read request, and also clear the `source`.

### Implementation

```rust
use std::boxed::Box;
use std::{thread, time};

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

    pub fn set_source(&mut self, source: S) {
        self.source = Some(source);
    }

    pub fn get_transformed(&mut self) -> Option<V> {
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
        println!("sleep for {:?}s.", sec);
        return sec;
    });
    let mut lazy_transformer = LazyTransformer::new(transform_fn);

    lazy_transformer.set_source(5);
    let value = lazy_transformer.get_transformed();

    println!("{:?}", value);
}
```

### What is the problem?



## Before we optimize our code.

replace `&mut self` with `&self`, introduce refcell

several comparation:

refcell, cell, 


## First Attempt For Optimization

problem of the current implementation

Object level lock

code

## Second Attempt For Optimization

filed-lock

## Foundation Of Lock-Free Programming

## Atomic

## Memory Barrier

## What Is A Lock?

## Third Attempt For Optimization
lock free

## Memory Management
lock-free(with specific type)
epoch-based memory reclamation


