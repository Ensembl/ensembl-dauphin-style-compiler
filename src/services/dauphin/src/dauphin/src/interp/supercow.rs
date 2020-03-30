/* SuperCow implements enhanced copy-on-write for interpreter registers.
 *
 * During an instruction, reads and writes may occur to the same register. In this case, we don't want writes to become
 * visible until the operation is over (on "commit", like atomicity in a database). The problem is that the same register
 * may be read from as written to so we can't share register contents BUT we need to avoid unnecessary copies and provide
 * the register contents in an efficiently usable manner. To achieve this, writes are divided into two types. A "write"
 * does not need the old contents of the array and so is provided with a new register which replaces the old at commit.
 * "modify" returns the values of the old register to allow modify in place. If there are NO outstanding reads (such as is
 * the case when the registers are disjoint) then the original array is returned for modification. If there ARE
 * outstanfing reads (which is rare), then a copy is made prior to update. modify must be after any reads or those reads
 * will fail (as the original data has probably been lost). Similarly, there can only be a single modify as the second
 * would also initially require the original contents.
 * 
 * As committing is probably delegated to a transaction-wide process, it is provided in a non-polymorphic trait so that it
 * can be easily queued with other pending commits.
 * 
 * clone() copies a SuperCow such that any outstanding operations and modifications are shared between the clones. This is
 * useful in, for example, returning a handle from the register file to the operation without a mess of reference lifetimes.
 * 
 * In the case when the data is to be copied (eg register-to-register), data_clone is provided. Initially this is a reference
 * to the data in the source but modify forces copying (copy-on-write).
 */

use std::cell::{ Ref, RefCell, RefMut };
use std::rc::Rc;
use owning_ref::{ RefRef, RefMutRefMut };

#[derive(Clone)]
pub struct SuperCow<T> {
    ctor: Rc<RefCell<dyn (Fn() -> T)>>,
    copy: Rc<RefCell<dyn (Fn(&T) -> T)>>,
    set: Option<Rc<RefCell<T>>>,
    get: Option<Rc<RefCell<T>>>
}

impl<T> SuperCow<T> {
    pub fn new<F,G>(ctor: F, copy: G, data: T) -> SuperCow<T> where F: Fn() -> T + Clone + 'static, G: Fn(&T) -> T + Clone + 'static {
        SuperCow {
            ctor: Rc::new(RefCell::new(ctor)),
            copy: Rc::new(RefCell::new(copy)),
            get: Some(Rc::new(RefCell::new(data))),
            set: None
        }
    }

    pub fn copy(&mut self, other: &SuperCow<T>) -> Result<(),String> {
        if let Some(get) = other.get.as_ref() {
            self.set = Some(get.clone());
            Ok(())
        } else {
            Err(format!("Attempt to copy modifying register"))
        }
    }

    pub fn copy_and_modify(&mut self, other: &SuperCow<T>) -> Result<RefMutRefMut<T>,String> {
        if let Some(get) = other.get.as_ref() {
            let x = Rc::new(RefCell::new(self.copy.borrow_mut()(&get.borrow())));
            self.set = Some(x);
            Ok(RefMutRefMut::new(self.set.as_ref().unwrap().borrow_mut()))
        } else {
            Err(format!("Attempt to copy modifying register"))
        }
    }

    pub fn read(&self) -> Result<RefRef<T>,String> {
        Ok(RefRef::new(self.get.as_ref().ok_or_else(|| format!("Attempt to read modifying register"))?.borrow()))
    }

    pub fn write(&mut self) -> RefMutRefMut<T> {
        let new_value : T = (self.ctor.borrow_mut())();
        let write = Rc::new(RefCell::new(new_value));
        self.set = Some(write);
        RefMutRefMut::new(self.set.as_ref().unwrap().borrow_mut())
    }

    pub fn modify(&mut self) -> Result<RefMutRefMut<T>,String> {
        let mut get = self.get.take().ok_or_else(|| format!("Attempt to modify twice"))?;
        if Rc::strong_count(&get) > 1 {
            let x = Rc::new(RefCell::new(self.copy.borrow_mut()(&get.borrow())));
            get = x;
        }
        if get.try_borrow_mut().is_ok() {
            self.set = Some(get);
            Ok(RefMutRefMut::new(self.set.as_ref().unwrap().borrow_mut()))
        } else {
            let v = get.borrow();
            let new_value = Rc::new(RefCell::new(self.copy.borrow_mut()(&v)));
            self.set = Some(new_value);
            Ok(RefMutRefMut::new(self.set.as_ref().unwrap().borrow_mut()))
        }
    }
}

pub trait SuperCowCommit {
    fn commit(&mut self);
}

impl<T> SuperCowCommit for SuperCow<T> {
    fn commit(&mut self) {
        if let Some(set) = self.set.take() {
            self.get = Some(set);
        }
    }
}
