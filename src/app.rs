use itertools::join;

use crate::hex_pattern::*;
use crate::parsing::parse_string_to_list;
use crate::rendering::*;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct HexeditApp {
    // this how you opt-out of serialization of a member
    #[serde(skip)]
    show_canonical: bool,

		pattern_text: String,
		canonical_text: String,
}

impl Default for HexeditApp {
    fn default() -> Self {
        Self {
            // Example stuff:
            show_canonical: false,
						pattern_text: "HexPattern(aqweqad, NORTH_WEST)".to_string(),
						canonical_text: "".to_string()
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
		let Self { show_canonical, pattern_text, canonical_text } = self;
		
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

			// ui.add(egui::Slider::new(value, 0.0..=10.0).text("value"));
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
				Ok(x) => x.render(ui, rect),
				Err(_) => (),
			}
		});
	}
}