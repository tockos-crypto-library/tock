use core::cell::UnsafeCell;
use core::ptr;

use sam4l::gpio::{PA};

/// A mutable memory location that enforces borrow rules at runtime without
/// possible panics.
///
/// A `InstrumentedTakeCell` is a potential reference to mutable memory. Borrow rules are
/// enforced by forcing clients to either move the memory out of the cell or
/// operate on a borrow within a closure. You can think of a `InstrumentedTakeCell` as a
/// between an `Option` wrapped in a `RefCell` --- attempts to take the value
/// from inside a `InstrumentedTakeCell` may fail by returning `None`.
pub struct InstrumentedTakeCell<T> {
    val: UnsafeCell<Option<T>>,
}

impl<T> InstrumentedTakeCell<T> {
    pub const fn empty() -> InstrumentedTakeCell<T> {
        InstrumentedTakeCell { val: UnsafeCell::new(None) }
    }

    /// Creates a new `InstrumentedTakeCell` containing `value`
    pub const fn new(value: T) -> InstrumentedTakeCell<T> {
        InstrumentedTakeCell { val: UnsafeCell::new(Some(value)) }
    }

    pub fn is_none(&self) -> bool {
        unsafe { (&*self.val.get()).is_none() }
    }

    pub fn is_some(&self) -> bool {
        unsafe { (&*self.val.get()).is_some() }
    }

    /// Takes the value out of the `InstrumentedTakeCell` leaving a `None` in it's place. If
    /// the value has already been taken elsewhere (and not `replace`ed), the
    /// returned `Option` will be empty.
    ///
    /// # Examples
    ///
    /// ```
    /// let cell = InstrumentedTakeCell::new(1234);
    /// let x = &cell;
    /// let y = &cell;
    ///
    /// x.take();
    /// assert_eq!(y.take(), None);
    /// ```
    pub fn take(&self) -> Option<T> {
        unsafe {
            let inner = &mut *self.val.get();
            inner.take()
        }
    }

    pub fn put(&self, val: Option<T>) {
        let _ = self.take();
        let ptr = self.val.get();
        unsafe {
            ptr::replace(ptr, val);
        }
    }

    /// Replaces the contents of the `InstrumentedTakeCell` with `val`. If the cell was not
    /// empty, the previous value is returned, otherwise `None` is returned.
    pub fn replace(&self, val: T) -> Option<T> {
        let prev = self.take();
        let ptr = self.val.get();
        unsafe {
            ptr::replace(ptr, Some(val));
        }
        prev
    }

    /// Allows `closure` to borrow the contents of the `InstrumentedTakeCell` if-and-only-if
    /// it is not `take`n already. The state of the `InstrumentedTakeCell` is unchanged
    /// after the closure completes.
    ///
    /// # Examples
    ///
    /// ```
    /// let cell = InstrumentedTakeCell::new(1234);
    /// let x = &cell;
    /// let y = &cell;
    ///
    /// x.map(|value| {
    ///     // We have mutable access to the value while in the closure
    ///     value += 1;
    /// });
    ///
    /// // After the closure completes, the mutable memory is still in the cell,
    /// // but potentially changed.
    /// assert_eq!(y.take(), Some(1235));
    /// ```
    pub fn map<F, R>(&self, closure: F) -> Option<R>
        where F: FnOnce(&mut T) -> R
    {
        let maybe_val = self.take();
        maybe_val.map(|mut val| {
            let res = closure(&mut val);
            self.replace(val);
            res
        })
    }

    #[inline(never)]
    pub fn instrumented_map<F, R>(&self, closure: F) -> Option<R>
        where F: FnOnce(&mut T) -> R
    {
        unsafe {
            PA[16].clear();
        }
        let maybe_val = self.take();
        unsafe {
            PA[16].set();
            PA[16].clear();
        }
        let ret_val = maybe_val.map(|mut val| {
            unsafe {
                PA[16].set();
                PA[16].clear();
            }
            let res = closure(&mut val);
            unsafe {
                PA[16].set();
                PA[16].clear();
            }
            self.replace(val);
            unsafe {
                PA[16].set();
                PA[16].clear();
            }
            res
        });
        unsafe {
            PA[16].set();
        }

        ret_val
    }


    pub fn map_or<F, R>(&self, default: R, closure: F) -> R
        where F: FnOnce(&mut T) -> R
    {
        let maybe_val = self.take();
        maybe_val.map_or(default, |mut val| {
            let res = closure(&mut val);
            self.replace(val);
            res
        })
    }

    pub fn modify_or_replace<F, G>(&self, modify: F, mkval: G)
        where F: FnOnce(&mut T),
              G: FnOnce() -> T
    {
        let val = match self.take() {
            Some(mut val) => {
                modify(&mut val);
                val
            }
            None => mkval(),
        };
        self.replace(val);
    }
}
