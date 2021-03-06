use std::slice;

fn slice_of_zst() {
    fn foo<T>(v: &[T]) -> Option<&[T]> {
        let mut it = v.iter();
        for _ in 0..5 {
            it.next();
        }
        Some(it.as_slice())
    }

    fn foo_mut<T>(v: &mut [T]) -> Option<&mut [T]> {
        let mut it = v.iter_mut();
        for _ in 0..5 {
            it.next();
        }
        Some(it.into_slice())
    }

    // In a slice of zero-size elements the pointer is meaningless.
    // Ensure iteration still works even if the pointer is at the end of the address space.
    let slice: &[()] = unsafe { slice::from_raw_parts(-5isize as *const (), 10) };
    assert_eq!(slice.len(), 10);
    assert_eq!(slice.iter().count(), 10);

    // .nth() on the iterator should also behave correctly
    let mut it = slice.iter();
    assert!(it.nth(5).is_some());
    assert_eq!(it.count(), 4);

    // Converting Iter to a slice should never have a null pointer
    assert!(foo(slice).is_some());

    // Test mutable iterators as well
    let slice: &mut [()] = unsafe { slice::from_raw_parts_mut(-5isize as *mut (), 10) };
    assert_eq!(slice.len(), 10);
    assert_eq!(slice.iter_mut().count(), 10);

    {
        let mut it = slice.iter_mut();
        assert!(it.nth(5).is_some());
        assert_eq!(it.count(), 4);
    }

    assert!(foo_mut(slice).is_some())
}

fn test_iter_ref_consistency() {
    use std::fmt::Debug;

    fn test<T : Copy + Debug + PartialEq>(x : T) {
        let v : &[T] = &[x, x, x];
        let v_ptrs : [*const T; 3] = match v {
            [ref v1, ref v2, ref v3] => [v1 as *const _, v2 as *const _, v3 as *const _],
            _ => unreachable!()
        };
        let len = v.len();

        // nth(i)
        for i in 0..len {
            assert_eq!(&v[i] as *const _, v_ptrs[i]); // check the v_ptrs array, just to be sure
            let nth = v.iter().nth(i).unwrap();
            assert_eq!(nth as *const _, v_ptrs[i]);
        }
        assert_eq!(v.iter().nth(len), None, "nth(len) should return None");

        // stepping through with nth(0)
        {
            let mut it = v.iter();
            for i in 0..len {
                let next = it.nth(0).unwrap();
                assert_eq!(next as *const _, v_ptrs[i]);
            }
            assert_eq!(it.nth(0), None);
        }

        // next()
        {
            let mut it = v.iter();
            for i in 0..len {
                let remaining = len - i;
                assert_eq!(it.size_hint(), (remaining, Some(remaining)));

                let next = it.next().unwrap();
                assert_eq!(next as *const _, v_ptrs[i]);
            }
            assert_eq!(it.size_hint(), (0, Some(0)));
            assert_eq!(it.next(), None, "The final call to next() should return None");
        }

        // next_back()
        {
            let mut it = v.iter();
            for i in 0..len {
                let remaining = len - i;
                assert_eq!(it.size_hint(), (remaining, Some(remaining)));

                let prev = it.next_back().unwrap();
                assert_eq!(prev as *const _, v_ptrs[remaining-1]);
            }
            assert_eq!(it.size_hint(), (0, Some(0)));
            assert_eq!(it.next_back(), None, "The final call to next_back() should return None");
        }
    }

    fn test_mut<T : Copy + Debug + PartialEq>(x : T) {
        let v : &mut [T] = &mut [x, x, x];
        let v_ptrs : [*mut T; 3] = match v {
            [ref v1, ref v2, ref v3] =>
              [v1 as *const _ as *mut _, v2 as *const _ as *mut _, v3 as *const _ as *mut _],
            _ => unreachable!()
        };
        let len = v.len();

        // nth(i)
        for i in 0..len {
            assert_eq!(&mut v[i] as *mut _, v_ptrs[i]); // check the v_ptrs array, just to be sure
            let nth = v.iter_mut().nth(i).unwrap();
            assert_eq!(nth as *mut _, v_ptrs[i]);
        }
        assert_eq!(v.iter().nth(len), None, "nth(len) should return None");

        // stepping through with nth(0)
        {
            let mut it = v.iter();
            for i in 0..len {
                let next = it.nth(0).unwrap();
                assert_eq!(next as *const _, v_ptrs[i]);
            }
            assert_eq!(it.nth(0), None);
        }

        // next()
        {
            let mut it = v.iter_mut();
            for i in 0..len {
                let remaining = len - i;
                assert_eq!(it.size_hint(), (remaining, Some(remaining)));

                let next = it.next().unwrap();
                assert_eq!(next as *mut _, v_ptrs[i]);
            }
            assert_eq!(it.size_hint(), (0, Some(0)));
            assert_eq!(it.next(), None, "The final call to next() should return None");
        }

        // next_back()
        {
            let mut it = v.iter_mut();
            for i in 0..len {
                let remaining = len - i;
                assert_eq!(it.size_hint(), (remaining, Some(remaining)));

                let prev = it.next_back().unwrap();
                assert_eq!(prev as *mut _, v_ptrs[remaining-1]);
            }
            assert_eq!(it.size_hint(), (0, Some(0)));
            assert_eq!(it.next_back(), None, "The final call to next_back() should return None");
        }
    }

    // Make sure iterators and slice patterns yield consistent addresses for various types,
    // including ZSTs.
    test(0u32);
    test(());
    test([0u32; 0]); // ZST with alignment > 0
    test_mut(0u32);
    test_mut(());
    test_mut([0u32; 0]); // ZST with alignment > 0
}

fn main() {
    slice_of_zst();
    test_iter_ref_consistency();
}
