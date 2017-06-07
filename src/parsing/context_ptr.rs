use super::syntax_definition::*;

use std::rc::{self, Rc};
use std::cell::{self, RefCell, Ref, RefMut};

use std::sync::{self, Arc, RwLock, RwLockReadGuard, RwLockWriteGuard, TryLockError};

use std::ops::{Deref, DerefMut};

/// Syntect uses interior mutability to lazily compile regexes on demand.
/// However, some users of Syntext might need fast single-threaded performance,
/// where others want to parallelize lots of highlighting. `ContextPtr`
/// provides methods common to both usages of interior mutability.
pub trait ContextPtr<'a>: Clone {
    type Weak;
    type ImmutRef: Deref<Target=Context>;
    type MutRef: DerefMut<Target=Context>;
    type BorrowError;
    type BorrowMutError;

    fn new(context: Context) -> Self;
    fn downgrade(&self) -> Self::Weak;
    fn upgrade(this: &Self::Weak) -> Option<Self> where Self: Sized;
    fn borrow(&'a self) -> Self::ImmutRef;
    fn try_borrow(&'a self) -> Result<Self::ImmutRef, Self::BorrowError>;
    fn borrow_mut(&'a self) -> Self::MutRef;
    fn try_borrow_mut(&'a self) -> Result<Self::MutRef, Self::BorrowMutError>;
}

#[derive(Debug, Clone)]
pub struct SerContextPtr(Rc<RefCell<Context>>);
#[derive(Debug, Clone)]
pub struct ParContextPtr(Arc<RwLock<Context>>);

impl<'a> ContextPtr<'a> for SerContextPtr {
    type Weak = rc::Weak<RefCell<Context>>;
    type ImmutRef = Ref<'a, Context>;
    type MutRef = RefMut<'a, Context>;
    type BorrowError = cell::BorrowError;
    type BorrowMutError = cell::BorrowMutError;

    fn new(context: Context) -> Self {
        SerContextPtr(Rc::new(RefCell::new(context)))
    }

    fn downgrade(&self) -> Self::Weak {
        Rc::downgrade(&self.0)
    }

    fn upgrade(this: &Self::Weak) -> Option<Self> {
        this.upgrade().map(SerContextPtr)
    }

    fn borrow(&'a self) -> Self::ImmutRef {
        self.0.borrow()
    }

    fn try_borrow(&'a self) -> Result<Self::ImmutRef, Self::BorrowError> {
        self.0.try_borrow()
    }

    fn borrow_mut(&'a self) -> Self::MutRef {
        self.0.borrow_mut()
    }

    fn try_borrow_mut(&'a self) -> Result<Self::MutRef, Self::BorrowMutError> {
        self.0.try_borrow_mut()
    }
}

impl <'a> ContextPtr<'a> for ParContextPtr {
    type Weak = sync::Weak<RwLock<Context>>;
    type ImmutRef = RwLockReadGuard<'a, Context>;
    type MutRef = RwLockWriteGuard<'a, Context>;
    type BorrowError = TryLockError<RwLockReadGuard<'a, Context>>;
    type BorrowMutError = TryLockError<RwLockWriteGuard<'a, Context>>;

    fn new(context: Context) -> Self {
        ParContextPtr(Arc::new(RwLock::new(context)))
    }

    fn downgrade(&self) -> Self::Weak {
        Arc::downgrade(&self.0)
    }

    fn upgrade(this: &Self::Weak) -> Option<Self> {
        this.upgrade().map(ParContextPtr)
    }

    fn borrow(&'a self) -> Self::ImmutRef {
        self.0.read().unwrap()
    }

    fn try_borrow(&'a self) -> Result<Self::ImmutRef, Self::BorrowError> {
        self.0.try_read()
    }
    
    fn borrow_mut(&'a self) -> Self::MutRef {
        self.0.write().unwrap()
    }

    fn try_borrow_mut(&'a self) -> Result<Self::MutRef, Self::BorrowMutError> {
        self.0.try_write()
    }
}
