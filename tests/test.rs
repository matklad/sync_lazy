#[macro_use]
extern crate sync_lazy;
use sync_lazy::Lazy;

use std::mem;
use std::ptr;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::SeqCst;
use std::thread;

fn go<F: FnOnce() -> ()>(mut f: F) {
    struct Yolo<T>(T);
    unsafe impl<T> Send for Yolo<T> {}

    let ptr: *const u8 = &mut f as *const F as *const u8;
    mem::forget(f);
    let yolo = Yolo(ptr);
    thread::spawn(move || {
        let f: F = unsafe { ptr::read(yolo.0 as *const F) };
        f();
    }).join().unwrap();
}

#[test]
fn test_drop() {
    static DROP_CNT: AtomicUsize = AtomicUsize::new(0);
    struct Dropper;
    impl Drop for Dropper {
        fn drop(&mut self) {
            DROP_CNT.fetch_add(1, SeqCst);
        }
    }

    let x = Lazy::new(|| Dropper);
    go(|| {
        Lazy::force(&x);
        assert_eq!(DROP_CNT.load(SeqCst), 0);
    });
    drop(x);
    assert_eq!(DROP_CNT.load(SeqCst), 1);
}

#[test]
fn test_drop_empty() {
    static DROP_CNT: AtomicUsize = AtomicUsize::new(0);
    struct Dropper;
    impl Drop for Dropper {
        fn drop(&mut self) {
            DROP_CNT.fetch_add(1, SeqCst);
        }
    }
    let x = Lazy::new(|| Dropper);
    assert_eq!(DROP_CNT.load(SeqCst), 0);
    drop(x);
    assert_eq!(DROP_CNT.load(SeqCst), 0);
}

#[test]
fn sync_lazy_macro() {
    let called = AtomicUsize::new(0);
    let x = sync_lazy! {
        called.fetch_add(1, SeqCst);
        92
    };

    assert_eq!(called.load(SeqCst), 0);

    go(|| {
        let y = *x - 30;
        assert_eq!(y, 62);
        assert_eq!(called.load(SeqCst), 1);
    });

    let y = *x - 30;
    assert_eq!(y, 62);
    assert_eq!(called.load(SeqCst), 1);
}

#[test]
fn static_lazy() {
    static XS: Lazy<Vec<i32>> = sync_lazy! {
        let mut xs = Vec::new();
        xs.push(1);
        xs.push(2);
        xs.push(3);
        xs
    };
    go(|| {
        assert_eq!(&*XS, &vec![1, 2, 3]);
    });
    assert_eq!(&*XS, &vec![1, 2, 3]);
}

#[test]
fn lazy_is_sync_send() {
    fn assert_traits<T: Send + Sync>() {}
    assert_traits::<Lazy<String>>();
}

#[cfg(feature = "nightly")]
#[test]
fn new_is_const_fn() {
    static XS: Lazy<Vec<i32>> = Lazy::new(|| {
        let mut xs = Vec::new();
        xs.push(1);
        xs.push(2);
        xs.push(3);
        xs
    });
    go(|| {
        assert_eq!(&*XS, &vec![1, 2, 3]);
    });
    assert_eq!(&*XS, &vec![1, 2, 3]);
}
