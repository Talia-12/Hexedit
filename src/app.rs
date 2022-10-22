use std::f32::consts::PI;

use eframe::{emath, epaint::CircleShape};
use egui::{Stroke, Color32, Rect, Ui, Shape, Pos2, pos2, Vec2, vec2};

use crate::hex_pattern::*;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct HexeditApp {
    // Example stuff:
    label: String,

    // this how you opt-out of serialization of a member
    #[serde(skip)]
    value: f32,

		pattern_text: String,
}

impl Default for HexeditApp {
    fn default() -> Self {
        Self {
            // Example stuff:
            label: "Hello World!".to_owned(),
            value: 2.7,
						pattern_text: "HexPattern(aqweqad, NORTH_WEST)".to_string(),
        }
    }
}

impl HexeditApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customized the look at feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

				// cc.egui_ctx.set_visuals(Visuals::light());

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl eframe::App for HexeditApp {
	/// Called by the frame work to save state before shutdown.
	fn save(&mut self, storage: &mut dyn eframe::Storage) {
		eframe::set_value(storage, eframe::APP_KEY, self);
	}

	/// Called each time the UI needs repainting, which may be many times per second.
	/// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
	fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
		let Self { label, value, pattern_text } = self;
		
		// Examples of how to create different panels and windows.
		// Pick whichever suits you.
		// Tip: a good default choice is to just keep the `CentralPanel`.
		// For inspiration and more examples, go to https://emilk.github.io/egui

		#[cfg(not(target_arch = "wasm32"))] // no File->Quit on web pages!
		egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
			// The top panel is often a good place for a menu bar:
			egui::menu::bar(ui, |ui| {
				ui.menu_button("File", |ui| {
					if ui.button("Quit").clicked() {
						_frame.close();
					}
				});
			});
		});

		egui::SidePanel::left("side_panel").show(ctx, |ui| {
			ui.heading("Side Panel");

			ui.horizontal(|ui| {
				ui.label("Write something: ");
				ui.text_edit_singleline(label);
			});

			ui.add(egui::Slider::new(value, 0.0..=10.0).text("value"));
			if ui.button("Increment").clicked() {
				*value += 1.0;
			}
			ui.add(
	egui::TextEdit::multiline(pattern_text)
					.font(egui::TextStyle::Monospace) // for cursor height
					.code_editor()
					.desired_rows(10)
					.lock_focus(true)
					.desired_width(f32::INFINITY)
			);
		});

		egui::CentralPanel::default().show(ctx, |ui| {
		// The central panel the region left after adding TopPanel's and SidePanel's
			egui::warn_if_debug_build(ui);

			let (_id, rect) = ui.allocate_space(ui.available_size());

			match HexPattern::parse_string(pattern_text.as_str()) {
				Ok(x) => x.draw_hex_pattern(ui, rect),
				Err(_) => (),
			}
		});
	}
}

impl HexPattern {
	fn draw_hex_pattern(&self, ui: &mut Ui, mut rect: Rect) {
		let diff = rect.height() - rect.width();
		if diff > 0.0 {
			rect.set_top(rect.top() - diff * 0.5);
			rect.set_bottom(rect.bottom() + diff * 0.5);
		}
		else if diff < 0.0 {
			rect.set_left(rect.left() - diff * 0.5);
			rect.set_right(rect.right() + diff * 0.5);
		}

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

		let to_screen = emath::RectTransform::from_to(Rect::from_x_y_ranges(min_x..=max_x, min_y..=max_y), rect);

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

		shapes.push(Shape::line(current_line, Stroke::new(line_thickness, colours[current_colour_index])));

		shapes.append(&mut dots);

		// draw the dot for the start of the pattern.
		shapes.push(Shape::Circle(CircleShape{ center: to_screen * coords[0].to_cartesian(), radius: dot_thickness, fill: colours[0], stroke: Stroke::new(dot_thickness * 0.6, dot_colour) }));

		ui.painter().extend(shapes);			
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