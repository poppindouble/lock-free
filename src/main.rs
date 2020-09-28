mod cell;
mod refcell;

use crate::cell::Cell;
use crate::refcell::RefCell;

fn main() {
    let cell = Cell::new(32);
    cell.set(3);
    assert_eq!(cell.get(), 3);

    let ref_cell = RefCell::new(32);
    {
        let mut mut_ref_guard = ref_cell.borrow_mut().unwrap();
        *mut_ref_guard = 100;
        println!("{:?}", *mut_ref_guard);
    }
    let reference = ref_cell.borrow().unwrap();
    println!("{:?}", *reference);
}
