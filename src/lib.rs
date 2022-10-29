#![warn(clippy::all, rust_2018_idioms)]

mod app;
mod hex_pattern;
mod parsing;
mod rendering;
pub use app::HexeditApp;

#[cfg(test)]
mod tests {
    use crate::hex_pattern::*;
    use crate::parsing::*;

    #[test]
    fn parse_to_pattern() {
        let text = "HexPattern(aqweqad, NORTH_WEST)";
				let expected = HexPattern::hex_pattern(HexAbsoluteDir::NorthWest, vec![HexDir::A, HexDir::Q, HexDir::W, HexDir::E, HexDir::Q, HexDir::A, HexDir::D]).unwrap();
        let parsed = HexPattern::parse_string(text);
				assert!(parsed.is_ok());
				assert_eq!(parsed.unwrap(), expected);
    }

		#[test]
		fn parse_to_list() {
			let text = "[HexPattern(a, EAST), HexPattern(aa, EAST)]";
			let expected = vec![
				HexPattern::hex_pattern(HexAbsoluteDir::East, vec![HexDir::A]).unwrap(),
				HexPattern::hex_pattern(HexAbsoluteDir::East, vec![HexDir::A, HexDir::A]).unwrap(),
			];
			let parsed = parse_string_to_list(text);

			assert!(parsed.is_ok());

			for (i, bx) in parsed.unwrap().iter().enumerate() {
				match bx.as_any().downcast_ref::<HexPattern>() {
					Some(x) => assert_eq!(*x, expected[i]),
					None => panic!("Couldn't downcast to HexPattern")
				};		
			}
		}

		#[test]
		fn converting_dirs() {
			for abs_dir in vec![HexAbsoluteDir::East, HexAbsoluteDir::SouthEast, HexAbsoluteDir::SouthWest, HexAbsoluteDir::West, HexAbsoluteDir::NorthWest, HexAbsoluteDir::NorthEast] {
				for rel_dir in vec![HexDir::A, HexDir::Q, HexDir::W, HexDir::E, HexDir::D] {
					assert_eq!(abs_dir.difference(rel_dir.coord_offset(hex_coord(0,0), abs_dir).1).unwrap(), rel_dir)
				}
			}
		}
}