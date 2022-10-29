use eframe::emath;
use eframe::epaint::CircleShape;
use egui::{Rect, Color32, Shape};
use itertools::join;

use crate::hex_pattern::*;
use crate::parsing::parse_string_to_list;
use crate::rendering::*;

const NODE_SELECT_SQR_RADIUS: f32 = 0.5*0.5; // distance from current node outside which should attempt to connect to next node.

#[derive(PartialEq)]
enum LastNodeState {
	Added,
	Neutral,
	Removed
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct HexeditApp {
    // this how you opt-out of serialization of a member
    #[serde(skip)]
    show_canonical: bool,
		#[serde(skip)]
    are_drawing: bool,

		pattern_text: String,
		canonical_text: String,
		#[serde(skip)]
		drawing_pattern: Option<HexPattern>,
		#[serde(skip)]
		start_draw_node: Option<HexCoord>,
		#[serde(skip)]
		last_draw_node: Option<HexCoord>,
		#[serde(skip)]
		last_node_status: LastNodeState,
}

impl Default for HexeditApp {
    fn default() -> Self {
        Self {
            // Example stuff:
            show_canonical: false,
            are_drawing: false,
						pattern_text: "HexPattern(aqweqad, NORTH_WEST)".to_string(),
						canonical_text: "".to_string(),
						drawing_pattern: None,
						start_draw_node: None,
						last_draw_node: None,
						last_node_status: LastNodeState::Added,
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
	fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
		let Self {
			show_canonical,
			are_drawing,
			pattern_text,
			canonical_text,
			drawing_pattern,
			start_draw_node,
			last_draw_node,
			last_node_status } = self;
		
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
						frame.close();
					}
				});
			});
		});

		egui::SidePanel::left("left_panel").default_width(frame.info().window_info.size.x * 0.142857).show(ctx, |ui| {
			ui.heading("Side Panel");

			// ui.add(egui::Slider::new(value, 0.0..=10.0).text("value"));
			ui.toggle_value(are_drawing, "Draw Pattern");
			
			if ui.toggle_value(show_canonical, "Show Canonical").clicked() && *show_canonical {
				let result = parse_string_to_list(pattern_text.as_str());

				*canonical_text = match result {
					Ok(x) => join(x.iter().map(|r| { r.canonical_text() }), "\n"),
					Err(_) => "".to_string(),
				}
			}

			let mut canonical_str = canonical_text.as_str();

			ui.add(
	egui::TextEdit::multiline(if *show_canonical { &mut canonical_str } else { pattern_text })
					.font(egui::TextStyle::Monospace) // for cursor height
					.code_editor()
					.desired_rows(93)
					.lock_focus(true)
					.desired_width(f32::INFINITY)
			);
		});

		if *are_drawing {
			egui::SidePanel::right("right_panel").default_width(frame.info().window_info.size.x * 0.3).show(ctx, |ui| {
				let (_id, rect) = ui.allocate_space(ui.available_size());
				// make RectTransform
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
				let to_screen = emath::RectTransform::from_to(Rect::from_x_y_ranges(-10.0..=10.0, -10.0..=10.0), m_rect);
				let from_screen = to_screen.inverse();

				let dot_colour = Color32::from_rgb(90,90,90);
				let mut dots: Vec<Shape> = vec![];

				// draw dots
				for i in -10..=10 {
					for j in -10..=10 {
						dots.push(Shape::Circle(CircleShape::filled(to_screen*hex_coord(i, j).to_cartesian(), 4.0, dot_colour)))
					}
				}

				ui.painter().extend(dots);

				let hover_pos = ui.input().pointer.hover_pos();

				let debug_colour0 = Color32::from_rgb(60,60,60);
				let debug_colour1 = Color32::from_rgb(130, 0,0);
				let debug_colour2 = Color32::from_rgb(0, 130,0);
				let debug_colour3 = Color32::from_rgb(0, 0,130);

				if ui.rect_contains_pointer(rect) {
					if ui.input().pointer.primary_clicked() {
						if let Some(hover_pos) = hover_pos {
							ui.painter().circle_filled(hover_pos, 20.0, debug_colour1);
							// if we clicked, and hover_pos returns a useful value, start drawing a pattern. 
							let hex_hover_pos = from_screen * hover_pos;
						
							*start_draw_node = Some(HexCoord::from_cartesian(hex_hover_pos));
							*last_draw_node = Some(HexCoord::from_cartesian(hex_hover_pos));
							*last_node_status = LastNodeState::Added;
						}
					} else if !ui.input().pointer.primary_down() && start_draw_node.is_some() {
						// finished drawing the pattern, do something with it
						*drawing_pattern = None;
						*start_draw_node = None;
						*last_draw_node = None;
						*last_node_status = LastNodeState::Added;
					} else if let Some(hover_pos) = hover_pos {
						ui.painter().circle_filled(hover_pos, 12.0, debug_colour1);
						let hex_hover_pos = from_screen * hover_pos;

						if let Some(inner_start_draw_node) = start_draw_node {
							ui.painter().circle_filled(to_screen*inner_start_draw_node.to_cartesian(), 8.0, debug_colour3);
						}

						if let Some(inner_last_draw_node) = last_draw_node {

							ui.painter().circle_filled(to_screen * inner_last_draw_node.to_cartesian(), 8.0, debug_colour2);


							let nearest = HexCoord::from_cartesian(hex_hover_pos);
							if nearest != *inner_last_draw_node && (hex_hover_pos - nearest.to_cartesian()).length_sq() < NODE_SELECT_SQR_RADIUS {
								// near enough to nearest to add it.
								if let Some(abs_dir) = inner_last_draw_node.dir_to(nearest) {
									// if the nearest is actually adjacent to the last node
									if let Some(inner_drawing_pattern) = drawing_pattern {
										// if a pattern has already been made, add to it.
										inner_drawing_pattern.add_dir(abs_dir)
									} else {
										// otherwise, make a new pattern.
										*drawing_pattern = if let Ok(pattern) = HexPattern::hex_pattern(abs_dir, vec![]) { Some(pattern) } else { None }
									}

									*inner_last_draw_node = nearest;
									*last_node_status = LastNodeState::Added;
								}
							}
							
							let sqr_dist_to_node = (hex_hover_pos - inner_last_draw_node.to_cartesian()).length_sq();
							if sqr_dist_to_node > NODE_EMPTY_SQR_RADIUS {
								if *last_node_status == LastNodeState::Added {
									*last_node_status = LastNodeState::Neutral
								} else if *last_node_status == LastNodeState::Removed {
									*last_node_status = LastNodeState::Neutral;
									// return node to previous node
									if let Some(inner_drawing_pattern) = drawing_pattern {
										*last_draw_node = Some(inner_drawing_pattern.to_coords().pop().unwrap() + start_draw_node.unwrap_or(hex_coord(0,0)))
									} else {
										// if the pattern doesn't exist then just set the last draw node back to the start node I guess.
										*last_draw_node = start_draw_node.clone();
									}
								}
							} else if *last_node_status == LastNodeState::Neutral {
								// if the sqr_dist is less than NODE_SELECT, and the last_node_status is neutral, removing a node
								*last_node_status = LastNodeState::Removed;
								if let Some(inner_drawing_pattern) = drawing_pattern {
									// if a pattern exists, remove a HexDir from it
									if inner_drawing_pattern.pattern_vec.pop().is_none() {
										// if there's no HexDir to remove, delete the whole pattern (since it only has two nodes, removing 1 from it deletes it)
										*drawing_pattern = None
									}
								} else {
									// if there's no HexPattern and we're removing a node, we have to stop drawing all together
									*start_draw_node = None;
									*last_draw_node = None;
								}
							}
						}

						if start_draw_node.is_some() {
							if let Some(inner_draw_pattern) = drawing_pattern {
								inner_draw_pattern.render(ui, to_screen, Some(hover_pos), *start_draw_node)
							} else {
								// draw line from inner_start_draw_node to mouse
							}
						}
					}
				} else if let Some(drawing_pattern) = drawing_pattern {
					// finished drawing the pattern, do something with it
					
				}
			});
		}

		egui::CentralPanel::default().show(ctx, |ui| {
		// The central panel the region left after adding TopPanel's and SidePanel's
			egui::warn_if_debug_build(ui);

			let (_id, rect) = ui.allocate_space(ui.available_size());

			match HexPattern::parse_string(pattern_text.as_str()) {
				Ok(x) => x.render_to_rect(ui, rect),
				Err(_) => (),
			}
		});
	}
}