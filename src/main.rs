#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    // Log to stdout (if you run with `RUST_LOG=debug`).

    tracing_subscriber::fmt::init();

    // let mut native_options = eframe::NativeOptions::default();
		// native_options.initial_window_pos = Some(pos2(1000.0, 0.0));
		// native_options.fullscreen = true;
		// let pos = native_options.initial_window_pos;
		// let size = native_options.initial_window_size;
		let native_options = eframe::NativeOptions {
      resizable: true,
      initial_window_size: Some(egui::Vec2 { x: 400.0, y: 400.0 }),
      min_window_size: Some(egui::Vec2 { x: 300.0, y: 300.0 }),
      ..Default::default()
};
		// println!("{pos:?}, {size:?}");
    eframe::run_native(
        "Hexedit",
        native_options,
        Box::new(|cc| Box::new(hexedit::HexeditApp::new(cc))),
    );
}

// when compiling to web using trunk.
#[cfg(target_arch = "wasm32")]
fn main() {
    // Make sure panics are logged using `console.error`.
    console_error_panic_hook::set_once();

    // Redirect tracing to console.log and friends:
    tracing_wasm::set_as_global_default();

    let web_options = eframe::WebOptions::default();
    eframe::start_web(
        "the_canvas_id", // hardcode it
        web_options,
        Box::new(|cc| Box::new(hexedit::HexeditApp::new(cc))),
    )
    .expect("failed to start eframe");
}
