mod cell;
mod rc;
//mod refcell;

use crate::cell::Cell;
//use crate::refcell::RefCell;
use crate::rc::Rc;

fn main() {
    let cell = Cell::new(32);
    let rc_cell = Rc::new(cell);

    let rc1 = rc_cell.clone();

    {
        let rc2 = rc_cell.clone();
        rc2.set(3);
    }

    let value = rc1.get();

    println!("{:?}", value);
}
