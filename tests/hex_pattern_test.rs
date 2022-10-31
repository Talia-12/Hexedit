use hexedit::hex_pattern::*;

#[test]
fn converting_dirs() {
	for abs_dir in vec![HexAbsoluteDir::East, HexAbsoluteDir::SouthEast, HexAbsoluteDir::SouthWest, HexAbsoluteDir::West, HexAbsoluteDir::NorthWest, HexAbsoluteDir::NorthEast] {
		for rel_dir in vec![HexDir::A, HexDir::Q, HexDir::W, HexDir::E, HexDir::D] {
			assert_eq!(abs_dir.difference(rel_dir.coord_offset(hex_coord(0,0), abs_dir).1).unwrap(), rel_dir)
		}
	}
}