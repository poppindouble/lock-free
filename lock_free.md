# Explore Rust Lock Free Programming

This series blog is a learning path of lock-free programing in Rust. We will start with something really simple, then I will introduce some smart pointers of Rust. Understanding the design of these smart pointers will help us to understand some design pattern of Rust(for example, the relation between `Refcell` and `RWLock`, `MutRef` and `MutexGuard`, etc), and we start to optimize our simple project, eventually we will reach the point of lock-free programming(currently I don't have a flame graph or benchmark to give out a concrete number about performance, because this blog mainly focuses on learning experience). As we go along the path, some other contents besides lock-free programming will also be introduced. Hopefully, at the end of this series blog, you can see some common design patterns in Rust, have a better understanding of Rust's safety feature. If I made any mistakes in the following contents, please let me know. My Email: poppindouble@gmail.com

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

## The Creed

**1. any borrow must last for a scope no greater than that of the owner.**

**2. You can either have one or more immutable references to a specific memory location or you can have only one mutable reference to that memory location**

As we go through this blog, always think about Rust's creed, this creed will be manifested everywhere in the Rust's code.

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
            let new_value = (self.transform_fn)(source.clone());
            self.value = Some(new_value.clone());
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

Nothing fancy here, just a really normal Rust mini project.

## Before we optimize our code.
It is pretty simple example, right? In the `main` function, we assume that the task needs to be executed for 5 seconds. As long as there is no other user sets the `source`, our transformer will always returning the cached value. Functionality wise, it is done. However let's say we have other kinds of transformers, like `SmartTransformer`, `SlowTransformer`, `DumbTransformer`, they all share the same functions above `set_source`, `get_transformed`, we would like to extract the `set_source` and `get_transformed` as a trait.

```rust
use std::boxed::Box;
use std::{thread, time};

trait Transformer<S,V> {
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

```

The interesting part is the trait, the functions inside the trait are `set_source` and `get_transformed`, both these two functions take a `&mut self`. But let's say we have a `DumbTransformer`, it also implement this `Transformer` trait, and since this transformer is pretty dumb, for the `set_source` function, no matter what you pass in, it sets its `source` to a default value, and `get_transformed` function always return a default value as well. In this situation, the `DumbTransformer` just need a `&self` for these two functions, however, our `LazyTransformer` needs a `&mut self`, there is a function signature conflict here. 

### Interior Mutability In Rust

The conflict here is that `LazyTransformer` and `DumbTransformer` share the same trait, but because of the mutability of `&self` is not the same, leads us to the conflict. If we can change the implementation of `set_source` and `get_transformed` in `LazyTransformer` just take a `&self`, then we are good.

How can we use a `&self` to change its internal data? [`RefCell`](https://doc.rust-lang.org/std/cell/struct.RefCell.html) comes to help.

```rust
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
```

Now we can extract the `Transformer` trait with `set_source` and `get_transformed` which takes `&self` as parameters.

Now we have a reason that why we need `Refcell`, but what actually happened inside of it? What is the relation between `Refcell` and `Cell` and `RC`? Is it possible to implement all these types by ourselves? We know that these types are not thread safe, then what is the relation between these types and `RWLock`, `Arc`? At the later of this series blog, we will get to that point, but first, again let's start with something simple.

### Rust building block

[`UnsafeCell`](https://doc.rust-lang.org/std/cell/struct.UnsafeCell.html) is the building block for interior mutability in Rust. It gives you a raw pointer `*mut T` to its contents. It is up to you as the abstraction designer to use that raw pointer correctly. There are two things worth to be noticed.

1. `pub const fn get(&self) -> *mut T`, Gets a mutable pointer to the wrapped value. This can be cast to a pointer of any kind. Ensure that the access is unique (no active references, mutable or not) when casting to &mut T, and ensure that there are no mutations or mutable aliases going on when casting to &T

2. 
```
impl<T> !Sync for UnsafeCell<T>
where
    T: ?Sized, 

```

Remember Rust's creed from above? When we use `UnsafeCell`, we have to obey the creed. The two points from above are basically the same thing, no matter it is single thread or multi-thread, we have to manage the access to the content inside `UnsafeCell` properly. 

But there is a doubt here, I mentioned ***thread safe***, what is actually thread safe means? I will not explain it for now, when we reach the point later in this blog, we will talk about it. For now, just remember the ***CREED***!

### Cell

Now we have a building block, `UnsafeCell`, let's use this building block to build something. [`Cell`](https://doc.rust-lang.org/std/cell/struct.Cell.html) is really similar to [`RefCell`](https://doc.rust-lang.org/std/cell/struct.RefCell.html). The main difference is here:

```rust
// Cell's get method signature
pub fn get(&self) -> T
// Cell's set method signature
pub fn set(&self, val: T)
```

```rust
// RefCell's get_mut signature
pub fn borrow_mut(&self) -> RefMut<T>
// RefCell's get signature
pub fn borrow(&self) -> Ref<T>
```

We will get into the `RefMut` a bit latter. Even though the idea behind `RefMut` is simple, but it is a really common thinking in Rust.

`RefCell` and `Cell` are really similar, but when we do the interior mutability in Rust, `RefCell` provides reference, and again, the creed, either multiple `&T` or single `&mut T`, however, `Cell` gives us ***ownership*** of the value `T`. `Cell` copies or moves contained value, while `RefCell` allows both mutable and immutable reference borrowing. And also `RefCell` checks borrow rules at run time, but Cell checks at compile time.

Let's use `UnsafeCell` to implement our own `Cell` type!

```rust

```







unsafecell -> cell -> refcell(once someone borrow it as &mut, there is no going back to &, so we need something tracking, similar idea as mutext_guard, Ref type and RefMut) -> RC

```
no way: *const T -> &mut T
ok way: *mut T -> &mut T
```



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


