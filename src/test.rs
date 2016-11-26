use quickcheck::{Arbitrary, Gen, StdGen};
use std::i32;

#[derive(Clone, Copy, Debug)]
struct Exclusion {
    side: Side,
    size: Size,
}

impl Arbitrary for Exclusion {
    fn arbitrary<G: Gen>(gen: &mut G) -> Exclusion {
        Exclusion {
            side: Arbitrary::arbitrary(gen),
            size: i32::abs(Arbitrary::arbitrary(gen)),
        }
    }
}

impl Arbitrary for Size {
    fn arbitrary<G: Gen>(gen: &mut G) -> Size {
        Size {
            inline: Arbitrary::arbitrary(gen),
            block: Arbitrary::arbitrary(gen),
        }
    }
}

impl Arbitrary for Au {
    fn arbitrary<G: Gen>(gen: &mut G) -> Au {
        Au(Arbitrary::arbitrary(gen))
    }
}

fn check_overflow(inline_size: Au, exclusions: Vec<Exclusion>) {

}
