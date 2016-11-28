use exclusions::Au;
use quickcheck::{Arbitrary, Gen, StdGen};
use rand;
use rust_test::Bencher;
use test::{self, Exclusion};

const EXCLUSION_COUNT: usize = 128;

fn generate_test_cases() -> Vec<(Au, Vec<Exclusion>)> {
    let mut test_cases: Vec<(Au, Vec<Exclusion>)> = vec![];
    let mut generator = StdGen::new(rand::thread_rng(), 100);
    for _ in 0..EXCLUSION_COUNT {
        test_cases.push(Arbitrary::arbitrary(&mut generator))
    }
    test_cases
}

#[bench]
fn bench(bencher: &mut Bencher) {
    let test_cases = generate_test_cases();
    bencher.iter(|| {
        for &(inline_size, ref exclusions) in test_cases.iter() {
            test::place(inline_size, (*exclusions).clone());
        }
    });
}


