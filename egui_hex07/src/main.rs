#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    //env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    //temp: set log level
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 300.0])
            .with_min_inner_size([300.0, 220.0])
            .with_icon(
                // NOTE: Adding an icon is optional
                eframe::icon_data::from_png_bytes(&include_bytes!("../assets/icon-256.png")[..])
                    .expect("Failed to load icon"),
            ),
        ..Default::default()
    };
    eframe::run_native(
        "eframe template",
        native_options,
        Box::new(|cc| Box::new(egui_hex07::HexApp::new(cc))),
    )
}

// When compiling to web using trunk:
#[cfg(target_arch = "wasm32")]
fn main() {
    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        // Initialize the thread pool with the maximum available threads
        let num_threads = get_num_threads();
        
        // Using then/catch pattern with a JsFuture since Promise doesn't implement Future
        let promise = egui_hex07::init_thread_pool(num_threads);
        let result = wasm_bindgen_futures::JsFuture::from(promise).await;
        
        match result {
            Ok(_) => log::info!("Thread pool initialized with {} threads", num_threads),
            Err(e) => log::error!("Failed to initialize thread pool: {:?}", e),
        }

        let start_result = eframe::WebRunner::new()
            .start(
                "the_canvas_id", // hardcode it
                web_options,
                Box::new(|cc| Box::new(egui_hex07::HexApp::new(cc))),
            )
            .await;
        let loading_text = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id("loading_text"));
        match start_result {
            Ok(_) => {
                loading_text.map(|e| e.remove());
            }
            Err(e) => {
                loading_text.map(|e| {
                    e.set_inner_html(
                        "<p> The app has crashed. See the developer console for details. </p>",
                    )
                });
                panic!("failed to start eframe: {e:?}");
            }
        }
    });
}

#[cfg(target_arch = "wasm32")]
fn get_num_threads() -> usize {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_namespace = self)]
        fn navigator() -> JsValue;
    }
    
    // Access the hardwareConcurrency property with web_sys
    let nav = web_sys::window()
        .expect("no window")
        .navigator();
    
    nav.hardware_concurrency() as usize
}
