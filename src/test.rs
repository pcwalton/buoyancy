// Any copyright is dedicated to the Public Domain.
// http://creativecommons.org/publicdomain/zero/1.0/

use app_units::Au;
use exclusions::{Exclusions, Point, Side, Size};
use quickcheck::{Arbitrary, Gen};
use std::cmp;
use std::i32;

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct InlineSize(Au);

impl Arbitrary for InlineSize {
    fn arbitrary<G: Gen>(gen: &mut G) -> InlineSize {
        InlineSize(Au(i32::abs(Arbitrary::arbitrary(gen))))
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Exclusion {
    side: Side,
    size: Size,
}

impl Arbitrary for Exclusion {
    fn arbitrary<G: Gen>(gen: &mut G) -> Exclusion {
        Exclusion {
            side: Arbitrary::arbitrary(gen),
            size: Size {
                inline: Au(i32::abs(Arbitrary::arbitrary(gen))),
                block: Au(i32::abs(Arbitrary::arbitrary(gen))),
            },
        }
    }
}

impl Arbitrary for Size {
    fn arbitrary<G: Gen>(gen: &mut G) -> Size {
        Size {
            inline: Au(Arbitrary::arbitrary(gen)),
            block: Au(Arbitrary::arbitrary(gen)),
        }
    }
}

impl Arbitrary for Side {
    fn arbitrary<G: Gen>(gen: &mut G) -> Side {
        if Arbitrary::arbitrary(gen) {
            Side::Left
        } else {
            Side::Right
        }
    }
}

#[derive(Clone, Debug)]
pub struct ExcludedArea {
    exclusion: Exclusion,
    origin: Point,
}

impl ExcludedArea {
    fn new(exclusion: &Exclusion, origin: &Point) -> ExcludedArea {
        ExcludedArea {
            exclusion: *exclusion,
            origin: *origin,
        }
    }

    fn intersects(&self, other: &ExcludedArea) -> bool {
        let this_left = self.origin.inline;
        let other_left = other.origin.inline;
        let this_right = this_left + self.exclusion.size.inline;
        let other_right = other_left + other.exclusion.size.inline;
        if (this_left <= other_left && this_right <= other_left) ||
                (this_left >= other_left && this_left >= other_right) {
            return false
        }

        let this_top = self.origin.block;
        let other_top = other.origin.block;
        let this_bottom = this_top + self.exclusion.size.block;
        let other_bottom = other_top + other.exclusion.size.block;
        if (this_top <= other_top && this_bottom <= other_top) ||
                (this_top >= other_top && this_bottom >= other_bottom) {
            return false
        }

        true
    }
}

pub fn place(inline_size: InlineSize, mut exclusion_info: Vec<Exclusion>) -> Vec<ExcludedArea> {
    let mut areas = Vec::with_capacity(exclusion_info.len());
    let mut exclusions = Exclusions::new(inline_size.0);
    for exclusion in &mut exclusion_info {
        exclusion.size.inline = cmp::min(exclusion.size.inline, inline_size.0);
        let origin = exclusions.place(exclusion.side, &exclusion.size).origin;
        let exclusion_inline_size = match exclusion.side {
            Side::Left => origin.inline + exclusion.size.inline,
            Side::Right => inline_size.0 - origin.inline,
        };
        exclusions.exclude(exclusion.side,
                           &Size::new(exclusion_inline_size, origin.block + exclusion.size.block));
        areas.push(ExcludedArea::new(exclusion, &origin))
    }
    areas
}

quickcheck! {
    fn check_overflow(inline_size: InlineSize, exclusions: Vec<Exclusion>) -> bool {
        let areas = place(inline_size, exclusions);
        for area in areas {
            assert!(area.origin.block >= Au(0));
            assert!(area.origin.inline >= Au(0));
            assert!(area.origin.inline + area.exclusion.size.inline <= inline_size.0);
        }
        true
    }

    fn check_overlap(inline_size: InlineSize, exclusions: Vec<Exclusion>) -> bool {
        let areas = place(inline_size, exclusions);
        for (i, a) in areas.iter().enumerate() {
            for b in &areas[(i + 1)..] {
                if a.intersects(b) {
                    panic!("illegal overlap! {:#?} vs {:#?}", a, b)
                }
                assert!(!a.intersects(b))
            }
        }
        true
    }

    fn check_vertical_packing(inline_size: InlineSize, exclusions: Vec<Exclusion>) -> bool {
        let areas = place(inline_size, exclusions);
        for (i, a) in areas.iter().enumerate().rev() {
            if a.origin.block == Au(0) {
                continue
            }
            assert!(areas[0..i].iter().any(|b| {
                a.origin.block == b.origin.block + b.exclusion.size.block
            }))
        }
        true
    }

    fn check_left_float_rules(inline_size: InlineSize, exclusions: Vec<Exclusion>) -> bool {
        let areas = place(inline_size, exclusions);
        for (i, a) in areas.iter().enumerate() {
            if a.exclusion.side != Side::Left {
                continue
            }
            for b in &areas[(i + 1)..] {
                if b.exclusion.side != Side::Left {
                    continue
                }
                assert!(b.origin.inline >= a.origin.inline + a.exclusion.size.inline ||
                        b.origin.block >= a.origin.block + a.exclusion.size.block)
            }
        }
        true
    }

    fn check_right_float_rules(inline_size: InlineSize, exclusions: Vec<Exclusion>) -> bool {
        let areas = place(inline_size, exclusions);
        for (i, a) in areas.iter().enumerate() {
            if a.exclusion.side != Side::Right {
                continue
            }
            for b in &areas[(i + 1)..] {
                if b.exclusion.side != Side::Right {
                    continue
                }
                assert!(b.origin.inline + b.exclusion.size.inline <= a.origin.inline ||
                        b.origin.block >= a.origin.block + a.exclusion.size.block)
            }
        }
        true
    }
}

