use crate::{hex_pattern::*, rendering::Renderable};

// #[derive(Debug)]
// pub struct ParseError {
// 	errors_on: Vec<u32>
// }

pub fn parse_string_to_list(string: &str) -> Result<Vec<Box<dyn Renderable>>, ParseError> {
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
		let parsed_pattern = HexPattern::parse_string(entry);
		if parsed_pattern.is_ok() {
			return Box::new(parsed_pattern.unwrap())
		};

		return Box::new("UNKNOWN".to_string());
	});

	return Ok(Vec::from_iter(out))
}