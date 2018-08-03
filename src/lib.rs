extern crate parking_lot;

use std::{
    mem,
    hint,
    cell::UnsafeCell,
};
use parking_lot::{Once, ONCE_INIT};

#[doc(hidden)]
pub use std::cell::UnsafeCell as __UnsafeCell;
#[doc(hidden)]
pub use parking_lot::ONCE_INIT as __ONCE_INIT;

/// A value which is initialized on the first access.
///
/// # Example
/// ```
/// #[macro_use]
/// extern crate sync_lazy;
///
/// use std::collections::HashMap;
/// use sync_lazy::Lazy;
///
/// static GLOBAL: Lazy<HashMap<i32, String>> = sync_lazy! {
///     println!("initializing global");
///     let mut m = HashMap::new();
///     m.insert(13, "Spica".to_string());
///     m.insert(74, "Hoyten".to_string());
///     m
/// };
///
/// fn main() {
///     println!("ready");
///     let xs = vec![1, 2, 3];
///     let local = Lazy::new(move || {
///         println!("initializing local");
///         xs.into_iter().sum::<i32>()
///     });
///
///     ::std::thread::spawn(|| {
///         println!("{:?}", GLOBAL.get(&13));
///     }).join().unwrap();
///     println!("{:?}", GLOBAL.get(&74));
///     println!("{}", Lazy::force(&local));
///     println!("{}", Lazy::force(&local));
///
///     // Prints:
///     //   ready
///     //   initializing global
///     //   Some("Spica")
///     //   Some("Hoyten")
///     //   initializing local
///     //   6
///     //   6
/// }
/// ```
#[derive(Debug)]
pub struct Lazy<T, F = fn() -> T> {
    #[doc(hidden)]
    pub __once: Once,
    #[doc(hidden)]
    pub __state: UnsafeCell<__State<T, F>>,
}

unsafe impl<T: Send, F: Send> Send for Lazy<T, F> {}
// `Send` is important here: a `Lazy<NonSend>` can be created on
// thread A, initialized on thread `B` (which creates a `NonSend` on B),
// and dropped on `A` (which would effectively send a `NonSend`).
unsafe impl<T: Sync + Send, F: Sync + Send> Sync for Lazy<T, F> {}

#[doc(hidden)]
#[derive(Debug)]
pub enum __State<T, F> {
    Init(T),
    Uninit(F),
    Initializing,
}

impl<T, F: FnOnce() -> T> __State<T, F> {
    fn init(&mut self) {
        let f = match mem::replace(self, __State::Initializing) {
            __State::Uninit(f) => f,
            __State::Init(..) | __State::Initializing => {
                /// Once::call_once guarantees that this is indeed unreachable,
                /// even if `f` is reentrant. This is on a cold path, however,
                /// so let's stick to the safe version.
                unreachable!()
            }
        };
        let value = f();
        mem::replace(self, __State::Init(value));
    }
}

impl<T, F: FnOnce() -> T> Lazy<T, F> {
    /// Creates a new lazy value with the given initializing
    /// function.
    pub fn new(f: F) -> Lazy<T, F> {
        Lazy {
            __once: ONCE_INIT,
            __state: UnsafeCell::new(__State::Uninit(f)),
        }
    }

    /// Forces the evaluation of this lazy value and
    /// returns a reference to result. This is equivalent
    /// to the `Deref` impl, but is explicit.
    ///
    /// # Example
    /// ```
    /// use sync_lazy::Lazy;
    ///
    /// let lazy = Lazy::new(|| 92);
    ///
    /// assert_eq!(Lazy::force(&lazy), &92);
    /// assert_eq!(&*lazy, &92);
    /// ```
    pub fn force(this: &Lazy<T, F>) -> &T {
        this.__once.call_once(|| {
            // safe, b/c call_once guarantees exclusive access.
            let state: &mut __State<T, F> = unsafe { &mut *this.__state.get() };
            state.init();
        });
        unsafe {
            let state: &__State<T, F> = &*this.__state.get();
            match state {
                __State::Init(value) => value,
                // safe, b/c we've got past call_once,
                // which sets state to `Init` as the very last step
                _ => hint::unreachable_unchecked(),
            }
        }
    }
}

impl<T, F: FnOnce() -> T> ::std::ops::Deref for Lazy<T, F> {
    type Target = T;
    fn deref(&self) -> &T {
        Lazy::force(self)
    }
}

/// Creates a new lazy value initialized by the given closure block.
/// This macro works in const contexts.
/// If you need a `move` closure, use `Lazy::new` constructor function.
///
/// # Example
/// ```
/// # #[macro_use] extern crate sync_lazy;
/// # fn main() {
/// let hello = "Hello, World!".to_string();
///
/// let lazy = sync_lazy! {
///     hello.to_uppercase()
/// };
///
/// assert_eq!(&*lazy, "HELLO, WORLD!");
/// # }
/// ```
#[macro_export]
macro_rules! sync_lazy {
    ($($block:tt)*) => {
        $crate::Lazy {
            __once: $crate::__ONCE_INIT,
            __state: $crate::__UnsafeCell::new(
                $crate::__State::Uninit(|| { $($block)* })
            ),
        }
    };
}
