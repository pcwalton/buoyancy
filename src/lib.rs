// Copyright 2016 The Servo Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Manages exclusions according to the rules in CSS 2.1 § 9.5.1 and allows objects to be placed
//! adjacent to them.
//!
//! In practice, this algorithm seems to be O(n) for placement of n floats due to the splay tree
//! and aggressive merging of bands. Worst case, it is O(n²); however, this is very rare.

#![cfg_attr(test, feature(test))]

extern crate app_units;
#[cfg(test)]
#[macro_use]
extern crate quickcheck;
#[cfg(test)]
extern crate rand;
#[cfg(test)]
extern crate test as rust_test;

pub mod exclusions;
mod map;
mod node;
#[cfg(test)]
mod bench;
#[cfg(test)]
mod test;

