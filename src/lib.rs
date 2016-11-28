#![cfg_attr(test, feature(test))]

#[cfg(test)]
#[macro_use]
extern crate quickcheck;
#[cfg(test)]
extern crate rand;
#[cfg(test)]
extern crate test as rust_test;

mod exclusions;
mod map;
mod node;
#[cfg(test)]
mod bench;
#[cfg(test)]
mod test;

