// Any copyright is dedicated to the Public Domain.
// http://creativecommons.org/publicdomain/zero/1.0/

use quickcheck::{Arbitrary, StdGen};
use rand;
use rust_test::Bencher;
use test::{self, Exclusion, InlineSize};

fn generate_test_cases(count: usize) -> Vec<(InlineSize, Vec<Exclusion>)> {
    let mut test_cases: Vec<(InlineSize, Vec<Exclusion>)> = vec![];
    let mut generator = StdGen::new(rand::thread_rng(), 100);
    for _ in 0..count {
        test_cases.push(Arbitrary::arbitrary(&mut generator))
    }
    test_cases
}

fn bench(count: usize, bencher: &mut Bencher) {
    let test_cases = generate_test_cases(count);
    bencher.iter(|| {
        for &(inline_size, ref exclusions) in test_cases.iter() {
            test::place(inline_size, (*exclusions).clone());
        }
    });
}

#[bench]
fn bench_1(bencher: &mut Bencher) { bench(1, bencher) }
#[bench]
fn bench_4(bencher: &mut Bencher) { bench(4, bencher) }
#[bench]
fn bench_16(bencher: &mut Bencher) { bench(16, bencher) }
#[bench]
fn bench_128(bencher: &mut Bencher) { bench(128, bencher) }
#[bench]
fn bench_1024(bencher: &mut Bencher) { bench(1024, bencher) }


