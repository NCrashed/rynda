use modular_bitfield::{specifiers::*, *};
use std::fmt;
use num_traits::identities::Zero;
use core::ops::Add;

/// Amount of bytes RgbVoxel takes in memory
pub const RGB_VOXEL_SIZE: usize = 2;

/// 16-bit RGB voxel with 5 bits for red and blue colors and 6 bits for green color. Human eye is considered
/// more sensitive to green tones. Zero values in all components are considered as an empty voxel.
#[repr(packed(2))]
#[derive(Clone, Copy)]
#[bitfield]
pub struct RgbVoxel {
    pub red: B5,
    pub green: B6,
    pub blue: B5,
}

impl PartialEq for RgbVoxel {
    fn eq(&self, other: &Self) -> bool {
        self.red() == other.red() && self.green() == other.green() && self.blue() == other.blue()
    }
}

impl Eq for RgbVoxel {}

impl fmt::Debug for RgbVoxel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RgbVoxel")
            .field("red", &self.red())
            .field("green", &self.green())
            .field("blue", &self.blue())
            .finish()
    }
}

impl Add for RgbVoxel {
    type Output = RgbVoxel;

    fn add(self, other: Self) -> Self {
        RgbVoxel::rgb(self.red() + other.red(), self.green() + other.green(), self.blue() + other.blue())
    }
}

impl Zero for RgbVoxel {
    fn zero() -> Self {
        Self::empty()
    }

    fn is_zero(&self) -> bool {
        self.is_empty()
    }
}

impl RgbVoxel {
    /// Empty voxel is voxel with all values set to zero.
    pub fn empty() -> Self {
        RgbVoxel::from_bytes([0u8, 0])
    }

    /// Empty voxel is voxel with all values set to zero.
    pub fn is_empty(&self) -> bool {
        let bytes = self.into_bytes();
        bytes[0] == 0 && bytes[1] == 0
    }

    /// Shortcut for making color from red green blue bytes.
    pub fn rgb(red: u8, green: u8, blue: u8) -> Self {
        RgbVoxel::new()
            .with_red(red)
            .with_green(green)
            .with_blue(blue)
    }

    /// Shortcut for making only red color shade
    pub fn only_red(red: u8) -> Self {
        Self::empty().with_red(red)
    }

    /// Shortcut for making only green color shade
    pub fn only_green(green: u8) -> Self {
        Self::empty().with_green(green)
    }

    /// Shortcut for making only blue color shade
    pub fn only_blue(blue: u8) -> Self {
        Self::empty().with_blue(blue)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_tests() {
        assert!(RgbVoxel::empty().is_empty(), "Empty voxel is not empty");
        assert_eq!(
            RgbVoxel::rgb(31, 0, 0).into_bytes(),
            [0b00011111, 0b00000000],
            "Red bits are not in expected place"
        );
        assert_eq!(
            RgbVoxel::rgb(0, 63, 0).into_bytes(),
            [0b11100000, 0b00000111],
            "Green bits are not in expected place"
        );
        assert_eq!(
            RgbVoxel::rgb(0, 0, 31).into_bytes(),
            [0b00000000, 0b11111000],
            "Blue bits are not in expected place"
        );
    }
}
