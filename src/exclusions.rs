// Copyright 2016 The Servo Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! The algorithm that manages exclusions and places objects according to the rules in CSS 2.1 §
//! 9.5.1.

use app_units::Au;
use map::SplayMap;
use std::cmp::Ordering;
use std::fmt::{self, Debug, Formatter};
use std::i32;
use std::iter;

const MAX_AU: Au = Au(i32::MAX);

/// Tracks exclusions and allows objects to be placed adjacent to them.
#[derive(Clone)]
pub struct Exclusions {
    bands: SplayMap<Au, Band>,
    inline_size: Au,
}

#[derive(Clone, Copy, Debug)]
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

    fn get(&self, side: Side) -> Au {
        match side {
            Side::Left => self.left,
            Side::Right => self.right,
        }
    }

    fn set(&mut self, side: Side, inline_size: Au) {
        match side {
            Side::Left => self.left = inline_size,
            Side::Right => self.right = inline_size,
        }
    }
}

/// A logical point.
#[derive(Clone, Copy, Debug)]
pub struct Point {
    /// The origin in the inline direction (normally horizontal).
    pub inline: Au,
    /// The origin in the block direction (normally vertical).
    pub block: Au,
}

impl Point {
    /// Creates a new logical point.
    pub fn new(inline: Au, block: Au) -> Point {
        Point {
            inline: inline,
            block: block,
        }
    }
}

/// A logical size.
#[derive(Clone, Copy, Debug)]
pub struct Size {
    /// The size in the inline direction (normally horizontal).
    pub inline: Au,
    /// The size in the block direction (normally vertical).
    pub block: Au,
}

impl Size {
    /// Creates a new logical size.
    pub fn new(inline: Au, block: Au) -> Size {
        Size {
            inline: inline,
            block: block,
        }
    }
}

/// Where an object should be placed to avoid overlapping any excluded area.
#[derive(Clone, Copy, Debug)]
pub struct Placement {
    /// The distance from the top left of the zone to the top left of the object.
    pub origin: Point,
    /// How much space is available for the object without overlapping any exclusions.
    pub available_inline_size: Au,
}

impl Placement {
    fn new(origin: &Point, available_inline_size: Au) -> Placement {
        Placement {
            origin: *origin,
            available_inline_size: available_inline_size,
        }
    }
}

/// Left or right.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Side {
    Left,
    Right,
}

impl Debug for Exclusions {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), fmt::Error> {
        try!(writeln!(formatter, "Exclusions(inline_size={:?}): bands:", self.inline_size));
        for (block_position, band) in self.bands.clone().into_iter() {
            try!(writeln!(formatter, "    {:?} {:?}", block_position, band));
        }
        Ok(())
    }
}

impl Exclusions {
    /// Creates a new rectangular exclusion zone infinitely long in the block direction with the
    /// given inline size.
    ///
    /// The zone starts out with no exclusions in it.
    pub fn new(inline_size: Au) -> Exclusions {
        Exclusions {
            bands: iter::once((Au(0), Band::new(Au(0), Au(0), MAX_AU))).collect(),
            inline_size: inline_size,
        }
    }

    /// Places an object so that it does not overlap any exclusions according to the CSS float
    /// placement rules.
    ///
    /// The object is aligned either to the left or right, depending on the size.
    pub fn place(&mut self, alignment: Side, size: &Size) -> Placement {
        let block_position =
            self.bands
                .lower_bound_with(|&band_block_start, band| {
                    compare_inline_size(band_block_start, band, size, self.inline_size)
                }).expect("Exclusions::place(): Didn't find a band!").0;
        let band = self.bands.get(&block_position).unwrap();
        let inline_position = match alignment {
            Side::Left => -band.left,
            Side::Right => self.inline_size + band.right - size.inline,
        };
        let origin = Point::new(inline_position, block_position);
        Placement::new(&origin, band.available_size(self.inline_size))
    }

    /// Excludes a rectangular area of the given size, preventing any objects from being placed
    /// within it.
    ///
    /// The excluded area touches the top left or top right of the zone, depending on the side.
    pub fn exclude(&mut self, side: Side, size: &Size) {
        if size.inline == Au(0) || size.block == Au(0) {
            return
        }

        self.split(size.block);

        let (mut last_block_position, mut last_band): (Au, Option<Band>) = (size.block, None);
        loop {
            let mut band_to_delete = None;
            match self.bands.get_with_mut(|block_position, band| {
                if last_block_position <= *block_position {
                    Ordering::Less
                } else if last_block_position > *block_position + band.length {
                    Ordering::Greater
                } else {
                    Ordering::Equal
                }
            }) {
                Some(&mut (block_position, ref mut band)) if -band.get(side) <= size.inline => {
                    // Extend this band.
                    //
                    //  ┌────────────────┐
                    //  │                │
                    //  ├───────┬────┬───┘
                    //  │       │ ─→ ┆
                    //  ├───┬───┴┄┄┄┄┘
                    //  │   │
                    //  ├───┘
                    //  │
                    band.set(side, -size.inline);

                    // Merge with the next band if we can.
                    //
                    //  ┌───────────────┐
                    //  │               │
                    //  ├───────────┬───┘
                    //  │           │
                    //  ├┄┄┄┄┄┄┄┄┄┄┄┤
                    //  │     ↓     │
                    //  ├───┬───────┘
                    //  │   │
                    //  ├───┘
                    //  │
                    if let Some(ref last_band) = last_band {
                        if band.left == last_band.left && band.right == last_band.right {
                            band.length = band.length + last_band.length;
                            band_to_delete = Some(last_block_position)
                        }
                    }

                    last_block_position = block_position;
                    last_band = Some(*band);
                }
                Some(_) | None => break,
            }

            // Delete the old band if we merged bands above.
            //
            //  ┌───────────────┐     ┌───────────────┐
            //  │               │     │               │
            //  ├────────────┬──┘     ├────────────┬──┘
            //  │            │        │            │
            //  ├┄┄┄┄┄┄┄┄┄┄┄┄┤     →  │            │
            //  │  (delete)  │        │            │
            //  ├───┬────────┘        ├───┬────────┘
            //  │   │                 │   │
            //  ├───┘                 ├───┘
            //  │                     │
            if let Some(band_to_delete) = band_to_delete {
                self.bands.remove(&band_to_delete);
            }
        }
    }

    /// Splits the band spanning the given block position in two at that point.
    ///
    ///  ┌───────────────┐     ┌───────────────┐
    ///  │               │     │               │
    ///  ├────────────┬──┘     ├────────────┬──┘
    ///  │            │        │            │
    ///  ├┄┄┄┄┄┄┄┄┄┄┄┄┼┄┄┄  →  ├────────────┤
    ///  │            │        │            │
    ///  ├───┬────────┘        ├───┬────────┘
    ///  │   │                 │   │
    ///  ├───┘                 ├───┘
    ///  │                     │
    fn split(&mut self, block_position: Au) {
        let (floor, left_size, right_size);
        {
            let &mut (upper_block_position, ref mut upper_band) =
                self.bands.get_with_mut(|&band_block_position, band| {
                    if block_position < band_block_position {
                        Ordering::Less
                    } else if block_position >= band_block_position + band.length {
                        Ordering::Greater
                    } else {
                        Ordering::Equal
                    }
                }).expect("Exclusions::split(): Didn't find band to split!");
            floor = upper_block_position + upper_band.length;
            upper_band.length = block_position - upper_block_position;
            left_size = upper_band.left;
            right_size = upper_band.right
        }
        let lower_band = Band::new(left_size, right_size, floor - block_position);
        self.bands.insert(block_position, lower_band);
    }
}

fn compare_inline_size(band_block_start: Au,
                       band: &Band,
                       exclusion_size: &Size,
                       inline_size: Au)
                       -> Ordering {
    match exclusion_size.inline.cmp(&band.available_size(inline_size)) {
        Ordering::Less | Ordering::Equal => Ordering::Less,
        Ordering::Greater if band_block_start + band.length == MAX_AU => Ordering::Equal,
        Ordering::Greater => Ordering::Greater,
    }
}


