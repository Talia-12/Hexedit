use std::ops::Add;

use num_derive::{FromPrimitive, ToPrimitive};    
use num_traits::{FromPrimitive, ToPrimitive};

use egui::{Pos2, pos2};
use regex::Regex;

const ANGLE_CHARS: [char; 10] = ['a', 'q', 'w', 'e', 'd', 'A', 'Q', 'W', 'E', 'D'];

#[derive(PartialEq, Debug)]
pub enum HexError {
	Overlap,
	InvalidString
}

#[derive(serde::Deserialize, serde::Serialize, Debug, PartialEq)]
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

	pub fn parse_string(string: &str) -> Result<HexPattern, HexError> {
		let splitter_re = Regex::new("[\\s:;,()]").unwrap();
		let out = splitter_re.split(string);

		let mut angles: Option<Vec<HexDir>> = None;
		let mut start_dir: Option<HexAbsoluteDir> = None;

		for part in out {
			if part.len() == 0 {
				continue;
			}

			if part.chars().all(|c| { for char in ANGLE_CHARS { if c == char { return true } } return false }) {
				angles = Some(Vec::from_iter(part.chars().map(|c| {
					match c.to_uppercase().nth(0) {
						Some('A') => HexDir::A,
						Some('Q') => HexDir::Q,
						Some('W') => HexDir::W,
						Some('E') => HexDir::E,
						Some('D') => HexDir::D,
						_ => panic!("Checked above that all chars were AQWED and now they aren't??")
					}
				 })));

				 continue
			}

			let mut matching = part.to_ascii_uppercase();
			matching.retain(|c| { c != '_' });

			start_dir = match &matching.as_str() {
				&"EAST" => Some(HexAbsoluteDir::East),
				&"SOUTHEAST" => Some(HexAbsoluteDir::SouthEast),
				&"SOUTHWEST" => Some(HexAbsoluteDir::SouthWest),
				&"WEST" => Some(HexAbsoluteDir::West),
				&"NORTHWEST" => Some(HexAbsoluteDir::NorthWest),
				&"NORTHEAST" => Some(HexAbsoluteDir::NorthEast),
				_ => start_dir
			}
		};

		if start_dir.is_none() {
			return Err(HexError::InvalidString)
		};

		return HexPattern::hex_pattern(start_dir.unwrap(), angles.unwrap_or(vec![]))
	}
	
	pub fn hex_pattern(start_dir: HexAbsoluteDir, pattern_vec: Vec<HexDir>) -> Result<HexPattern, HexError> {
		let pattern = HexPattern { start_dir, pattern_vec };
		
		if HexPattern::check_for_overlap(&pattern.to_coords()) {
			return Err(HexError::Overlap)
		}
	
		return Ok(pattern)
	}
	
	fn check_for_overlap(coords: &Vec<HexCoord>) -> bool {
		let mut visited_edges: Vec<(HexCoord, HexCoord)> = vec![];
	
		for index in 1..coords.len() {
			let start_coord = coords[index - 1];
			let end_coord = coords[index];
	
			if visited_edges.contains(&(end_coord, start_coord)) || visited_edges.contains(&(start_coord, end_coord)) {
				return true
			}
			visited_edges.push((start_coord, end_coord));
		}
	
		return false
	}
}

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone, Copy)]
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

#[derive(serde::Deserialize, serde::Serialize, PartialEq, ToPrimitive, FromPrimitive, Clone, Copy, Debug)]
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