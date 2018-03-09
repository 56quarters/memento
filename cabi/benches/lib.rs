#![feature(test)]
extern crate memento_cabi;
extern crate test;

use std::ffi::CString;
use test::Bencher;
use memento_cabi::{memento_points_fetch_full, memento_points_free, memento_points_is_error,
                   memento_header_fetch, memento_header_free, memento_header_is_error};



#[bench]
fn benchmark_memento_points_fetch_full(b: &mut Bencher) {
    let path = unsafe { CString::from_vec_unchecked(
        "../tests/upper_01.wsp".as_bytes().to_owned()
    )};

    b.iter(|| {
        let res = memento_points_fetch_full(
            path.as_ptr(),
            1502089980,
            1502259660,
            1502864800,
        );

        assert!(!memento_points_is_error(res));
        memento_points_free(res);
    });
}

#[bench]
fn benchmark_memento_header_fetch(b: &mut Bencher) {
    let path = unsafe { CString::from_vec_unchecked(
        "../tests/upper_01.wsp".as_bytes().to_owned()
    )};

    b.iter(|| {
        let res = memento_header_fetch(path.as_ptr());
        assert!(!memento_header_is_error(res));
        memento_header_free(res);
    });
}
