// Copyright (c) 2012 The Rust Project Developers
// Copyright (c) 2015 Guillaume Gomez
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! C-string manipulation and management
//!
//! This modules provides the basic methods for creating and manipulating
//! null-terminated strings for use with FFI calls (back to C). Most C APIs require
//! that the string being passed to them is null-terminated, and by default rust's
//! string types are *not* null terminated.
//!
//! The other problem with translating Rust strings to C strings is that Rust
//! strings can validly contain a null-byte in the middle of the string (0 is a
//! valid Unicode codepoint). This means that not all Rust strings can actually be
//! translated to C strings.
//!
//! # Creation of a C string
//!
//! A C string is managed through the `CString` type defined in this module. It
//! "owns" the internal buffer of characters and will automatically deallocate the
//! buffer when the string is dropped. The `ToCStr` trait is implemented for `&str`
//! and `&[u8]`, but the conversions can fail due to some of the limitations
//! explained above.
//!
//! This also means that currently whenever a C string is created, an allocation
//! must be performed to place the data elsewhere (the lifetime of the C string is
//! not tied to the lifetime of the original string/data buffer). If C strings are
//! heavily used in applications, then caching may be advisable to prevent
//! unnecessary amounts of allocations.
//!
//! Be carefull to remember that the memory is managed by C allocator API and not
//! by Rust allocator API.
//! That means that the CString pointers should be freed with C allocator API
//! if you intend to do that on your own, as the behaviour if you free them with
//! Rust's allocator API is not well defined
//!

#![feature(core, collections, libc, convert)]

extern crate libc;

use std::string::String;
use std::ffi::CString;
use std::{mem, slice};

/// A generic trait for converting a *const c_str to another Rust type
pub trait FromCStr {
    /// Copy the c_str into the returned type
    unsafe fn from_c_str(c_str: *const libc::c_char) -> Self;
    /// the same as from_c_str but for old code compatibility
    unsafe fn from_raw_buf(c_str: *const u8) -> Self;
}

/// A generic trait for converting a value to a CString.
pub trait ToCStr {
    /// Copy the receiver into a CString.
    ///
    /// # Panics
    ///
    /// Panics the task if the receiver has an interior null.
    fn to_c_str(&self) -> CString;

    /// Unsafe variant of `to_c_str()` that doesn't check for nulls.
    unsafe fn to_c_str_unchecked(&self) -> CString;

    /// Work with a temporary CString constructed from the receiver.
    /// The provided `*libc::c_char` will be freed immediately upon return.
    ///
    /// # Example
    ///
    /// ```rust
    /// extern crate libc;
    ///
    /// use std::c_str::ToCStr;
    ///
    /// fn main() {
    ///     let s = "PATH".with_c_str(|path| unsafe {
    ///         libc::getenv(path)
    ///     });
    /// }
    /// ```
    ///
    /// # Panics
    ///
    /// Panics the task if the receiver has an interior null.
    #[inline]
    fn with_c_str<T, F>(&self, f: F) -> T where
        F: FnOnce(*const libc::c_char) -> T,
    {
        let c_str = self.to_c_str();
        f(c_str.as_ptr())
    }

    /// Unsafe variant of `with_c_str()` that doesn't check for nulls.
    #[inline]
    unsafe fn with_c_str_unchecked<T, F>(&self, f: F) -> T where
        F: FnOnce(*const libc::c_char) -> T,
    {
        let c_str = self.to_c_str_unchecked();
        f(c_str.as_ptr())
    }
}

impl ToCStr for str {
    #[inline]
    fn to_c_str(&self) -> CString {
        self.as_bytes().to_c_str()
    }

    #[inline]
    unsafe fn to_c_str_unchecked(&self) -> CString {
        self.as_bytes().to_c_str_unchecked()
    }

    #[inline]
    fn with_c_str<T, F>(&self, f: F) -> T where
        F: FnOnce(*const libc::c_char) -> T,
    {
        self.as_bytes().with_c_str(f)
    }

    #[inline]
    unsafe fn with_c_str_unchecked<T, F>(&self, f: F) -> T where
        F: FnOnce(*const libc::c_char) -> T,
    {
        self.as_bytes().with_c_str_unchecked(f)
    }
}

impl ToCStr for String {
    #[inline]
    fn to_c_str(&self) -> CString {
        self.as_bytes().to_c_str()
    }

    #[inline]
    unsafe fn to_c_str_unchecked(&self) -> CString {
        self.as_bytes().to_c_str_unchecked()
    }

    #[inline]
    fn with_c_str<T, F>(&self, f: F) -> T where
        F: FnOnce(*const libc::c_char) -> T,
    {
        self.as_bytes().with_c_str(f)
    }

    #[inline]
    unsafe fn with_c_str_unchecked<T, F>(&self, f: F) -> T where
        F: FnOnce(*const libc::c_char) -> T,
    {
        self.as_bytes().with_c_str_unchecked(f)
    }
}

impl FromCStr for String {
    #[inline]
    unsafe fn from_c_str(c_str: *const libc::c_char) -> String {
        let mut count = 0isize;

        loop {
            let tmp = ::std::intrinsics::offset(c_str, count);

            if *tmp == 0i8 {
                break;
            }
            count += 1;
        }
        if count == 0 {
            String::new()
        } else {
            let v : Vec<u8> = Vec::from_raw_buf(c_str as *const u8, count as usize);

            String::from_utf8_unchecked(v)
        }
    }

    #[inline]
    unsafe fn from_raw_buf(c_str: *const u8) -> String {
        FromCStr::from_c_str(c_str as *const libc::c_char)
    }
}

impl FromCStr for CString {
    #[inline]
    unsafe fn from_c_str(c_str: *const libc::c_char) -> CString {
        let mut count = 0isize;

        loop {
            let tmp = ::std::intrinsics::offset(c_str, count);

            if *tmp == 0i8 {
                break;
            }
            count += 1;
        }
        if count == 0 {
            CString::new("\0").unwrap()
        } else {
            let v : Vec<u8> = Vec::from_raw_buf(c_str as *const u8, count as usize);

            CString::new(v).unwrap()
        }
    }

    #[inline]
    unsafe fn from_raw_buf(c_str: *const u8) -> CString {
        FromCStr::from_c_str(c_str as *const libc::c_char)
    }
}

// The length of the stack allocated buffer for `vec.with_c_str()`
const BUF_LEN: usize = 128;

impl ToCStr for [u8] {
    fn to_c_str(&self) -> CString {
        let cs = unsafe { self.to_c_str_unchecked() };

        check_for_null(self, cs.as_ptr() as *mut libc::c_char);
        cs
    }

    unsafe fn to_c_str_unchecked(&self) -> CString {
        CString::new(self).unwrap()
    }

    fn with_c_str<T, F>(&self, f: F) -> T where
        F: FnOnce(*const libc::c_char) -> T,
    {
        unsafe { with_c_str(self, true, f) }
    }

    unsafe fn with_c_str_unchecked<T, F>(&self, f: F) -> T where
        F: FnOnce(*const libc::c_char) -> T,
    {
        with_c_str(self, false, f)
    }
}

impl<'a, T: ToCStr> ToCStr for &'a T {
    #[inline]
    fn to_c_str(&self) -> CString {
        (**self).to_c_str()
    }

    #[inline]
    unsafe fn to_c_str_unchecked(&self) -> CString {
        (**self).to_c_str_unchecked()
    }

    #[inline]
    fn with_c_str<U, F>(&self, f: F) -> U where
        F: FnOnce(*const libc::c_char) -> U,
    {
        (**self).with_c_str(f)
    }

    #[inline]
    unsafe fn with_c_str_unchecked<U, F>(&self, f: F) -> U where
        F: FnOnce(*const libc::c_char) -> U,
    {
        (**self).with_c_str_unchecked(f)
    }
}

// Unsafe function that handles possibly copying the &[u8] into a stack array.
unsafe fn with_c_str<T, F>(v: &[u8], checked: bool, f: F) -> T where
    F: FnOnce(*const libc::c_char) -> T,
{
    let c_str = if v.len() < BUF_LEN {
        let mut buf: [u8; BUF_LEN] = mem::uninitialized();
        let mut copy_: Vec<u8> = Vec::from(v);
        slice::bytes::copy_memory(&mut buf, copy_.as_mut_slice());
        buf[v.len()] = 0;

        let buf = buf.as_mut_ptr();
        if checked {
            check_for_null(v, buf as *mut libc::c_char);
        }

        return f(buf as *const libc::c_char)
    } else if checked {
        v.to_c_str()
    } else {
        v.to_c_str_unchecked()
    };

    f(c_str.as_ptr())
}

#[inline]
fn check_for_null(v: &[u8], buf: *mut libc::c_char) {
    for i in 0..v.len() {
        unsafe {
            let p = buf.offset(i as isize);
            assert!(*p != 0);
        }
    }
}

/// External iterator for a CString's bytes.
///
/// Use with the `std::iter` module.
/*#[allow(raw_pointer_deriving)]
#[derive(Clone)]
pub struct CChars<'a> {
    ptr: *const libc::c_char,
    marker: marker::ContravariantLifetime<'a>,
}

impl<'a> Iterator for CChars<'a> {
    type Item = libc::c_char;

    fn next(&mut self) -> Option<libc::c_char> {
        let ch = unsafe { *self.ptr };
        if ch == 0 {
            None
        } else {
            self.ptr = unsafe { self.ptr.offset(1) };
            Some(ch)
        }
    }
}*/

/// Parses a C "multistring", eg windows env values or
/// the req->ptr result in a uv_fs_readdir() call.
///
/// Optionally, a `count` can be passed in, limiting the
/// parsing to only being done `count`-times.
///
/// The specified closure is invoked with each string that
/// is found, and the number of strings found is returned.
pub unsafe fn from_c_multistring<F>(buf: *const libc::c_char,
                                    count: Option<usize>,
                                    mut f: F)
                                    -> usize where
    F: FnMut(&CString),
{

    let mut curr_ptr: usize = buf as usize;
    let mut ctr = 0;
    let (limited_count, limit) = match count {
        Some(limit) => (true, limit),
        None => (false, 0)
    };
    while ((limited_count && ctr < limit) || !limited_count)
          && *(curr_ptr as *const libc::c_char) != 0 as libc::c_char {
        let mut v : Vec<u8> = Vec::new();
        let mut decal = 0isize;

        loop {
            let tmp : u8 = *::std::intrinsics::offset(curr_ptr as *const libc::c_uchar, decal);
            if tmp == 0u8 {
                break;
            }
            v.push(tmp);
            decal += 1;
        }
        let cstr = CString::new(v).unwrap();
        f(&cstr);
        curr_ptr += cstr.as_bytes().len() + 1;
        ctr += 1;
    }
    return ctr;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::Thread;
    use libc;
    use std::ptr;
    use std::ffi::CString;

    #[test]
    fn test_str_multistring_parsing() {
        unsafe {
            let input = b"zero\0one\0\0";
            let ptr = input.as_ptr();
            let expected = ["zero", "one"];
            let mut it = expected.iter();
            let result = from_c_multistring(ptr as *const libc::c_char, None, |c| {
                let cbytes = c.as_bytes_no_nul();
                assert_eq!(cbytes, it.next().unwrap().as_bytes());
            });
            assert_eq!(result, 2);
            assert!(it.next().is_none());
        }
    }

    #[test]
    fn test_str_to_c_str() {
        let c_str = "".to_c_str();
        unsafe {
            assert_eq!(*c_str.as_ptr().offset(0), 0);
        }

        let c_str = "hello".to_c_str();
        let buf = c_str.as_ptr();
        unsafe {
            assert_eq!(*buf.offset(0), 'h' as libc::c_char);
            assert_eq!(*buf.offset(1), 'e' as libc::c_char);
            assert_eq!(*buf.offset(2), 'l' as libc::c_char);
            assert_eq!(*buf.offset(3), 'l' as libc::c_char);
            assert_eq!(*buf.offset(4), 'o' as libc::c_char);
            assert_eq!(*buf.offset(5), 0);
        }
    }

    #[test]
    fn test_vec_to_c_str() {
        let b: &[u8] = &[];
        let c_str = b.to_c_str();
        unsafe {
            assert_eq!(*c_str.as_ptr().offset(0), 0);
        }

        let c_str = b"hello".to_c_str();
        let buf = c_str.as_ptr();
        unsafe {
            assert_eq!(*buf.offset(0), 'h' as libc::c_char);
            assert_eq!(*buf.offset(1), 'e' as libc::c_char);
            assert_eq!(*buf.offset(2), 'l' as libc::c_char);
            assert_eq!(*buf.offset(3), 'l' as libc::c_char);
            assert_eq!(*buf.offset(4), 'o' as libc::c_char);
            assert_eq!(*buf.offset(5), 0);
        }

        let c_str = b"foo\xFF".to_c_str();
        let buf = c_str.as_ptr();
        unsafe {
            assert_eq!(*buf.offset(0), 'f' as libc::c_char);
            assert_eq!(*buf.offset(1), 'o' as libc::c_char);
            assert_eq!(*buf.offset(2), 'o' as libc::c_char);
            assert_eq!(*buf.offset(3), 0xffu8 as libc::c_char);
            assert_eq!(*buf.offset(4), 0);
        }
    }

    #[test]
    fn test_unwrap() {
        let c_str = "hello".to_c_str();
        unsafe { libc::free(c_str.into_inner() as *mut libc::c_void) }
    }

    #[test]
    fn test_as_ptr() {
        let c_str = "hello".to_c_str();
        let len = unsafe { libc::strlen(c_str.as_ptr()) };
        assert_eq!(len, 5);
    }

    #[test]
    fn test_iterator() {
        let c_str = "".to_c_str();
        let mut iter = c_str.iter();
        assert_eq!(iter.next(), None);

        let c_str = "hello".to_c_str();
        let mut iter = c_str.iter();
        assert_eq!(iter.next(), Some('h' as libc::c_char));
        assert_eq!(iter.next(), Some('e' as libc::c_char));
        assert_eq!(iter.next(), Some('l' as libc::c_char));
        assert_eq!(iter.next(), Some('l' as libc::c_char));
        assert_eq!(iter.next(), Some('o' as libc::c_char));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_to_c_str_fail() {
        assert!(Thread::spawn(move|| { "he\x00llo".to_c_str() }).join().is_err());
    }

    #[test]
    fn test_to_c_str_unchecked() {
        unsafe {
            let c_string = "he\x00llo".to_c_str_unchecked();
            let buf = c_string.as_ptr();
            assert_eq!(*buf.offset(0), 'h' as libc::c_char);
            assert_eq!(*buf.offset(1), 'e' as libc::c_char);
            assert_eq!(*buf.offset(2), 0);
            assert_eq!(*buf.offset(3), 'l' as libc::c_char);
            assert_eq!(*buf.offset(4), 'l' as libc::c_char);
            assert_eq!(*buf.offset(5), 'o' as libc::c_char);
            assert_eq!(*buf.offset(6), 0);
        }
    }

    #[test]
    fn test_as_bytes() {
        let c_str = "hello".to_c_str();
        assert_eq!(c_str.as_bytes(), b"hello\0");
        let c_str = "".to_c_str();
        assert_eq!(c_str.as_bytes(), b"\0");
        let c_str = b"foo\xFF".to_c_str();
        assert_eq!(c_str.as_bytes(), b"foo\xFF\0");
    }

    #[test]
    fn test_as_bytes_no_nul() {
        let c_str = "hello".to_c_str();
        assert_eq!(c_str.as_bytes_no_nul(), b"hello");
        let c_str = "".to_c_str();
        let exp: &[u8] = &[];
        assert_eq!(c_str.as_bytes_no_nul(), exp);
        let c_str = b"foo\xFF".to_c_str();
        assert_eq!(c_str.as_bytes_no_nul(), b"foo\xFF");
    }

    #[test]
    fn test_as_str() {
        let c_str = "hello".to_c_str();
        assert_eq!(c_str.as_str(), Some("hello"));
        let c_str = "".to_c_str();
        assert_eq!(c_str.as_str(), Some(""));
        let c_str = b"foo\xFF".to_c_str();
        assert_eq!(c_str.as_str(), None);
    }

    #[test]
    #[should_panic]
    fn test_new_fail() {
        let _c_str = unsafe { CString::new(ptr::null(), false) };
    }

    #[test]
    fn test_clone() {
        let a = "hello".to_c_str();
        let b = a.clone();
        assert!(a == b);
    }

    #[test]
    fn test_clone_noleak() {
        fn foo<F>(f: F) where F: FnOnce(&CString) {
            let s = "test".to_string();
            let c = s.to_c_str();
            // give the closure a non-owned CString
            let mut c_ = unsafe { CString::new(c.as_ptr(), false) };
            f(&c_);
            // muck with the buffer for later printing
            unsafe { *c_.as_mut_ptr() = 'X' as libc::c_char }
        }

        let mut c_: Option<CString> = None;
        foo(|c| {
            c_ = Some(c.clone());
            c.clone();
            // force a copy, reading the memory
            c.as_bytes().to_vec();
        });
        let c_ = c_.unwrap();
        // force a copy, reading the memory
        c_.as_bytes().to_vec();
    }
}

#[cfg(test)]
mod bench {
    extern crate test;

    //use prelude::v1::*;
    use self::test::Bencher;
    use libc;

    #[inline]
    fn check(s: &str, c_str: *const libc::c_char) {
        let s_buf = s.as_ptr();
        for i in 0..s.len() {
            unsafe {
                assert_eq!(
                    *s_buf.offset(i as isize) as libc::c_char,
                    *c_str.offset(i as isize));
            }
        }
    }

    static S_SHORT: &'static str = "Mary";
    static S_MEDIUM: &'static str = "Mary had a little lamb";
    static S_LONG: &'static str = "\
        Mary had a little lamb, Little lamb
        Mary had a little lamb, Little lamb
        Mary had a little lamb, Little lamb
        Mary had a little lamb, Little lamb
        Mary had a little lamb, Little lamb
        Mary had a little lamb, Little lamb";

    fn bench_to_string(b: &mut Bencher, s: &str) {
        b.iter(|| {
            let c_str = s.to_c_str();
            check(s, c_str.as_ptr());
        })
    }

    #[bench]
    fn bench_to_c_str_short(b: &mut Bencher) {
        bench_to_string(b, S_SHORT)
    }

    #[bench]
    fn bench_to_c_str_medium(b: &mut Bencher) {
        bench_to_string(b, S_MEDIUM)
    }

    #[bench]
    fn bench_to_c_str_long(b: &mut Bencher) {
        bench_to_string(b, S_LONG)
    }

    fn bench_to_c_str_unchecked(b: &mut Bencher, s: &str) {
        b.iter(|| {
            let c_str = unsafe { s.to_c_str_unchecked() };
            check(s, c_str.as_ptr())
        })
    }

    #[bench]
    fn bench_to_c_str_unchecked_short(b: &mut Bencher) {
        bench_to_c_str_unchecked(b, S_SHORT)
    }

    #[bench]
    fn bench_to_c_str_unchecked_medium(b: &mut Bencher) {
        bench_to_c_str_unchecked(b, S_MEDIUM)
    }

    #[bench]
    fn bench_to_c_str_unchecked_long(b: &mut Bencher) {
        bench_to_c_str_unchecked(b, S_LONG)
    }

    fn bench_with_c_str(b: &mut Bencher, s: &str) {
        b.iter(|| {
            s.with_c_str(|c_str_buf| check(s, c_str_buf))
        })
    }

    #[bench]
    fn bench_with_c_str_short(b: &mut Bencher) {
        bench_with_c_str(b, S_SHORT)
    }

    #[bench]
    fn bench_with_c_str_medium(b: &mut Bencher) {
        bench_with_c_str(b, S_MEDIUM)
    }

    #[bench]
    fn bench_with_c_str_long(b: &mut Bencher) {
        bench_with_c_str(b, S_LONG)
    }

    fn bench_with_c_str_unchecked(b: &mut Bencher, s: &str) {
        b.iter(|| {
            unsafe {
                s.with_c_str_unchecked(|c_str_buf| check(s, c_str_buf))
            }
        })
    }

    #[bench]
    fn bench_with_c_str_unchecked_short(b: &mut Bencher) {
        bench_with_c_str_unchecked(b, S_SHORT)
    }

    #[bench]
    fn bench_with_c_str_unchecked_medium(b: &mut Bencher) {
        bench_with_c_str_unchecked(b, S_MEDIUM)
    }

    #[bench]
    fn bench_with_c_str_unchecked_long(b: &mut Bencher) {
        bench_with_c_str_unchecked(b, S_LONG)
    }
}
