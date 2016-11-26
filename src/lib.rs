mod map;
mod node;

use map::SplayMap;
use std::cmp::Ordering;
use std::ops::{Add, Neg, Sub};

struct Exclusions {
    bands: SplayMap<Au, Band>,
    inline_size: Au,
}

struct Band {
    left: Au,
    right: Au,
    length: Au,
}

impl Band {
    fn new(left: Au, right: Au, length: Au) -> Band {
        Band {
            left: left,
            right: right,
            length: length,
        }
    }

    fn available_size(&self, inline_size: Au) -> Au {
        inline_size + self.left + self.right
    }
}

struct Point {
    inline: Au,
    block: Au,
}

struct Size {
    inline: Au,
    block: Au,
}

#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
struct Au(i32);

impl Add<Au> for Au {
    type Output = Au;
    fn add(self, other: Au) -> Au {
        Au(self.0 + other.0)
    }
}

impl Sub<Au> for Au {
    type Output = Au;
    fn sub(self, other: Au) -> Au {
        Au(self.0 - other.0)
    }
}

impl Neg for Au {
    type Output = Au;
    fn neg(self) -> Au {
        Au(-self.0)
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Side {
    Left,
    Right,
}

impl Exclusions {
    fn place(&mut self, side: Side, size: &Size) -> Point {
        let block_position = self.bands
                                 .lower_bound_with(|_, band| {
                                     size.inline.cmp(&band.available_size(self.inline_size))
                                 }).expect("Exclusions::place(): Didn't find a band!").0;
        let band = self.bands.get(&block_position).unwrap();
        let inline_position = match side {
            Side::Left => -band.left,
            Side::Right => self.inline_size + band.right - size.inline,
        };
        Point {
            inline: inline_position,
            block: block_position,
        }
    }

    fn exclude(&mut self, side: Side, size: &Size) {
        self.split(side, size);

        let ceiling_block_position = {
            let inline_size = self.inline_size;
            let &mut (ceiling_block_position, ref mut ceiling_band) =
                self.bands
                    .lower_bound_with_mut(|_, band| {
                        size.inline.cmp(&band.available_size(inline_size))
                    }).expect("Exclusions::exclude(): Didn't find the ceiling?!");
            match side {
                Side::Left => ceiling_band.left = size.inline,
                Side::Right => ceiling_band.right = size.inline,
            }
            ceiling_block_position
        };

        while let Some((&block_position, _)) = self.bands.get_with(|block_position, band| {
            if *block_position >= size.block {
                Ordering::Less
            } else if *block_position <= ceiling_block_position {
                Ordering::Greater
            } else {
                Ordering::Equal
            }
        }) {
            self.bands.remove(&block_position); // TODO(pcwalton): Remove without splaying.
        }
    }

    fn split(&mut self, side: Side, size: &Size) {
        let (floor, opposite_inline_size) = {
            let &mut (upper_block_position, ref mut upper_band) =
                self.bands.get_with_mut(|block_position, band| {
                    if size.block < *block_position {
                        Ordering::Less
                    } else if size.block >= *block_position + band.length {
                        Ordering::Greater
                    } else {
                        Ordering::Equal
                    }
                }).expect("Exclusions::split(): Didn't find band to split!");
            let floor = upper_block_position + upper_band.length;
            let opposite_inline_size = match side {
                Side::Left => upper_band.right,
                Side::Right => upper_band.left,
            };
            upper_band.length = size.block - upper_block_position;
            (floor, opposite_inline_size)
        };
        let lower_band_length = floor - size.block;
        let lower_band = match side {
            Side::Left => Band::new(-size.inline, opposite_inline_size, lower_band_length),
            Side::Right => Band::new(opposite_inline_size, -size.inline, lower_band_length),
        };
        self.bands.insert(floor, lower_band);
    }
}

