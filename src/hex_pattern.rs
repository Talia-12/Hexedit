use std::ops::Add;

use num_derive::{FromPrimitive, ToPrimitive};    
use num_traits::{FromPrimitive, ToPrimitive};

use egui::{Pos2, pos2};

#[derive(serde::Deserialize, serde::Serialize)]
pub struct HexPattern {
	pub start_dir: HexAbsoluteDir,
	pub pattern_vec: Vec<HexDir>
}

impl HexPattern {
	pub fn to_coords(&self) -> Vec<HexCoord> {
		let mut coords = vec![hex_coord(0, 0)];

		let mut prev_coord = self.start_dir.coord_offset();
		let mut prev_dir = self.start_dir;

		coords.push(prev_coord);

		for rel_dir in &self.pattern_vec {
				(prev_coord, prev_dir) = rel_dir.coord_offset(prev_coord, prev_dir);
				coords.push(prev_coord);
		}

		return coords;
	}
}

#[derive(serde::Deserialize, serde::Serialize)]
pub enum HexDir {
		A,
		Q,
		W,
		E,
		D
}

impl HexDir {
	/// Takes in the absolute direction that the line was going, and returns the next coord as well as the new absolute direction.
	fn coord_offset(&self, prev_coord: HexCoord, prev_dir: HexAbsoluteDir) -> (HexCoord, HexAbsoluteDir) {
		let new_dir = match *self {
			HexDir::A => prev_dir.turn(-2),
			HexDir::Q => prev_dir.turn(-1),
			HexDir::W => prev_dir,
			HexDir::E => prev_dir.turn(1),
			HexDir::D => prev_dir.turn(2),
		};

		return (prev_coord + new_dir.coord_offset(), new_dir);
	}
}

#[derive(serde::Deserialize, serde::Serialize, ToPrimitive, FromPrimitive, Clone, Copy)]
pub enum HexAbsoluteDir {
	East,
	SouthEast,
	SouthWest,
	West,
	NorthWest,
	NorthEast
}

impl HexAbsoluteDir {
	fn coord_offset(&self) -> HexCoord {
		match *self {
			HexAbsoluteDir::East => hex_coord(1, 0),
			HexAbsoluteDir::SouthEast => hex_coord(0, 1),
			HexAbsoluteDir::SouthWest => hex_coord(-1, 1),
			HexAbsoluteDir::West => hex_coord(-1, 0),
			HexAbsoluteDir::NorthWest => hex_coord(0, -1),
			HexAbsoluteDir::NorthEast => hex_coord(1, -1),
		}
	}

	fn turn(&self, amount: i16) -> HexAbsoluteDir {
		// cursed code to convert the current HexAbsoluteDir to an int, then add amount and modulo 6.
		match FromPrimitive::from_i16(((*ToPrimitive::to_i16(self).get_or_insert(0) + amount) % 6 + 6) % 6) {
				Some(d2) => d2,
				None => FromPrimitive::from_u8(0).unwrap(),
		}
	}
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HexCoord {
	pub q: i32,
	pub r: i32
}

impl HexCoord {
	pub fn to_cartesian(&self) -> Pos2 {
		let q = self.q as f32;
		let r = self.r as f32;
		return pos2(3.0_f32.sqrt() * q + 3.0_f32.sqrt()/2.0 * r, 3.0/2.0 * r);
	}
}

impl Add for HexCoord {
    type Output = HexCoord;

    fn add(self, rhs: Self) -> Self::Output {
        return hex_coord(self.q + rhs.q, self.r + rhs.r)
    }
}

pub fn hex_coord(q: i32, r: i32) -> HexCoord {
	return HexCoord{ q, r }
}