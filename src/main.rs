mod cell;
mod refcell;

use crate::cell::Cell;
use crate::refcell::RefCell;

fn main() {
    let cell = Cell::new(32);
    cell.set(3);
    assert_eq!(cell.get(), 3);

    let ref_cell = RefCell::new(32);
    let mut_ref = ref_cell.borrow_mut().unwrap();
    *mut_ref = 100;

    println!("{:?}", mut_ref);
}
