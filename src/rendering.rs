use std::f32::consts::PI;
use std::any::Any;

use eframe::{emath::{self, RectTransform}, epaint::CircleShape};
use egui::{Stroke, Color32, Rect, Ui, Shape, Pos2, pos2, Vec2, vec2};

use crate::hex_pattern::*;

pub trait Renderable {
	fn render_to_rect(&self, ui: &mut Ui, rect: Rect);

	fn canonical_text(&self) -> String;

	fn as_any(&self) -> &dyn Any;
}

impl Renderable for String {
	fn render_to_rect(&self, ui: &mut Ui, _rect: Rect) {
		ui.label(self);
	}

	fn canonical_text(&self) -> String {
		self.clone()
	}

	fn as_any(&self) -> &dyn Any {
		self
	}
}

impl HexPattern {
	pub(crate) fn render(&self, ui: &mut Ui, to_screen: RectTransform, cursor_position: Option<Pos2>, offset: Option<HexCoord>) {
		let colours = if ui.visuals().dark_mode {
			vec![
				Color32::from_rgb(0xff, 0x6b, 0xff),
				Color32::from_rgb(0xa8, 0x1e, 0xe3),
				Color32::from_rgb(0x64, 0x90, 0xed),
				Color32::from_rgb(0xb1, 0x89, 0xc7),
			]
		} else {
			vec![
				Color32::from_rgb(0xff, 0x6b, 0xff),
				Color32::from_rgb(0xa8, 0x1e, 0xe3),
				Color32::from_rgb(0x64, 0x90, 0xed),
				Color32::from_rgb(0xb1, 0x89, 0xc7),
			]
		};

		let dot_colour = if ui.visuals().dark_mode {
			Color32::from_rgb(255, 255, 255)
		} else {
			Color32::from_rgb(0, 0, 0)
		};

		let line_thickness = 5.0 as f32;
		let dot_thickness = 10.0 as f32;
		let arrow_thickness = 25.0 as f32;
		
		let coords = Vec::from_iter(self.to_coords().clone().iter().map(|coord| { if let Some(offset) = offset { *coord + offset } else { *coord } }));

		let mut shapes = vec![];
		let mut dots: Vec<Shape> = vec![];

		let mut current_colour_index = 0;
		let mut current_line: Vec<Pos2> = vec![to_screen * coords[0].to_cartesian()];

		let mut visited_vertex_colours: Vec<(HexCoord, usize)> = vec![(coords[0], current_colour_index)];

		for index in 1..coords.len() {
			let start_coord = coords[index - 1];
			let end_coord = coords[index];

			let start_screen = to_screen * start_coord.to_cartesian();
			let end_screen = to_screen * end_coord.to_cartesian();


			if visited_vertex_colours.contains(&(end_coord, current_colour_index)) {
				let midway = pos2(start_screen.x * 0.5 + end_screen.x * 0.5, start_screen.y * 0.5 + end_screen.y * 0.5);
				
				current_line.push(midway);
				shapes.push(Shape::line(current_line, Stroke::new(line_thickness, colours[current_colour_index])));
				
				current_colour_index = (current_colour_index + 1) % colours.len();
				
				shapes.push(arrow(&midway, &start_screen, arrow_thickness, colours[current_colour_index]));
				current_line = vec![midway, end_screen];
			} else {
				current_line.push(end_screen);
			}

			visited_vertex_colours.push((end_coord, current_colour_index));

			dots.push(Shape::circle_filled(end_screen, dot_thickness, dot_colour))
		}

		if let Some(cursor_position) = cursor_position {
			current_line.push(cursor_position)
		}

		shapes.push(Shape::line(current_line, Stroke::new(line_thickness, colours[current_colour_index])));

		shapes.append(&mut dots);

		// draw the dot for the start of the pattern.
		shapes.push(Shape::Circle(CircleShape{ center: to_screen * coords[0].to_cartesian(), radius: dot_thickness, fill: colours[0], stroke: Stroke::new(dot_thickness * 0.6, dot_colour) }));

		ui.painter().extend(shapes);
	}
}

impl Renderable for HexPattern {
	fn render_to_rect(&self, ui: &mut Ui, rect: Rect) {
		let mut m_rect = rect;

		let diff = m_rect.height() - m_rect.width();
		if diff > 0.0 {
			m_rect.set_top(m_rect.top() + diff * 0.5);
			m_rect.set_bottom(m_rect.bottom() - diff * 0.5);
		}
		else if diff < 0.0 {
			m_rect.set_left(m_rect.left() - diff * 0.5);
			m_rect.set_right(m_rect.right() + diff * 0.5);
		}

		let coords = self.to_coords();

		let mut min_x = f32::MAX;
		let mut max_x = f32::MIN;
		let mut min_y = f32::MAX;
		let mut max_y = f32::MIN;

		coords.iter().map(|coord| {coord.to_cartesian()}).for_each(|pos| {
			if pos.x < min_x {
				min_x = pos.x;
			}
			if pos.x > max_x {
				max_x = pos.x;
			}
			if pos.y < min_y {
				min_y = pos.y;
			}
			if pos.y > max_y {
				max_y = pos.y;
			}
		});

		let dx = max_x - min_x;
		let dy = max_y - min_y;
		let dd = dx - dy;

		// this section makes it so that the drawn region is square, so that the drawn pattern isn't distorted if it
		// is longer than it is tall. 
		if dd > 0.0 {
			min_y -= dd/2.0;
			max_y += dd/2.0;
		} else {
			min_x -= -dd/2.0;
			max_x += -dd/2.0;
		}

		let to_screen = emath::RectTransform::from_to(Rect::from_x_y_ranges(min_x..=max_x, min_y..=max_y), m_rect);

		self.render(ui, to_screen, None, None)
	}

	fn canonical_text(&self) -> String {
		let angles_str: String = self.pattern_vec.iter().map(|d| { hex_dir_char(d) }).collect();
		let start_dir_str = hex_absolute_dir_str(self.start_dir);
		format!("HexPattern({angles_str}, {start_dir_str})")
	}

	fn as_any(&self) -> &dyn Any {
		self
	}
}

fn hex_dir_char(dir: &HexDir) -> char {
	match dir {
    HexDir::A => 'a',
    HexDir::Q => 'q',
    HexDir::W => 'w',
    HexDir::E => 'e',
    HexDir::D => 'd',
	}
}

fn hex_absolute_dir_str<'a>(dir: HexAbsoluteDir) -> &'a str {
	match dir {
    HexAbsoluteDir::East => "EAST",
    HexAbsoluteDir::SouthEast => "SOUTH_EAST",
    HexAbsoluteDir::SouthWest => "SOUTH_WEST",
    HexAbsoluteDir::West => "WEST",
    HexAbsoluteDir::NorthWest => "NORTH_WEST",
    HexAbsoluteDir::NorthEast => "NORTH_EAST",
}
}

fn arrow(tip_at: &Pos2, from: &Pos2, side_len: f32, colour: Color32) -> Shape {
	let dir = (*from - *tip_at).normalized() * side_len;

	let p0 = rotate(&dir, -PI/6.0);
	let p1 = rotate(&dir, PI/6.0);

	// return Shape::circle_filled(*tip_at + p0, side_len, colour);
	return Shape::convex_polygon(vec![*tip_at - dir*0.5, *tip_at + p0 - dir*0.5, *tip_at + p1 - dir*0.5], colour, Stroke::none());
}

fn rotate(vec: &Vec2, t: f32) -> Vec2 {
	let angle = vec.angle() + t;
	let mag = vec.length();
	return vec2(mag * angle.cos(), mag * angle.sin());
}