use regex::Regex;

use crate::{hex_pattern::*, rendering::Renderable};

// #[derive(Debug)]
// pub struct ParseError {
// 	errors_on: Vec<u32>
// }

pub fn parse_to_list(string: &str) -> Result<Vec<Box<dyn Renderable>>, HexError> {
	let mut str = string.to_string().to_ascii_uppercase();

	let separated = if str.contains("HEX_PATTERN") || str.contains("HEXPATTERN") {
		// assume that it uses ',' to separate patterns, and that all are HE
		let mut indent = 0;
		str = str.chars().map(|c| {
			if c == '(' {
				indent += 1;
			} else if c == ')' {
				indent -= 1;
				indent = i32::min(indent, 0)
			}

			match c {
				',' => if indent == 0 { ',' } else { ';' },
				';' => if indent >= 0 { ';' } else { ',' },
				_ => c
			}
		}).collect();
		str.split(',')
	} else {
		// assume that it uses '\n' to separate patterns
		str.split('\n')
	};

	let out = separated.map(|entry| -> Box<dyn Renderable> {
		if let Ok(parsed_pattern) = parse_to_hex_pattern(entry) {
			return Box::new(parsed_pattern)
		};

		return Box::new("UNKNOWN".to_string());
	});

	return Ok(Vec::from_iter(out))
}

const ANGLE_CHARS: [char; 10] = ['a', 'q', 'w', 'e', 'd', 'A', 'Q', 'W', 'E', 'D'];

pub fn parse_to_hex_pattern(string: &str) -> Result<HexPattern, HexError> {
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

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_parse_to_pattern() {
			let text = "HexPattern(aqweqad, NORTH_WEST)";
			let expected = HexPattern::hex_pattern(HexAbsoluteDir::NorthWest, vec![HexDir::A, HexDir::Q, HexDir::W, HexDir::E, HexDir::Q, HexDir::A, HexDir::D]).unwrap();
			let parsed = parse_to_hex_pattern(text);
			assert!(parsed.is_ok());
			assert_eq!(parsed.unwrap(), expected);
	}

	#[test]
	fn test_parse_to_list() {
		let text = "[HexPattern(a, EAST), HexPattern(aa, EAST)]";
		let expected = vec![
			HexPattern::hex_pattern(HexAbsoluteDir::East, vec![HexDir::A]).unwrap(),
			HexPattern::hex_pattern(HexAbsoluteDir::East, vec![HexDir::A, HexDir::A]).unwrap(),
		];
		let parsed = parse_to_list(text);

		assert!(parsed.is_ok());

		for (i, bx) in parsed.unwrap().iter().enumerate() {
			match bx.as_any().downcast_ref::<HexPattern>() {
				Some(x) => assert_eq!(*x, expected[i]),
				None => panic!("Couldn't downcast to HexPattern")
			};		
		}
	}
}