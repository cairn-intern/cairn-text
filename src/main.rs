use eframe::egui;

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([900.0, 600.0])
            .with_title("Cairn Text"),
        ..Default::default()
    };
    eframe::run_native(
        "Cairn Text",
        options,
        Box::new(|_cc| Ok(Box::new(MinimalApp))),
    )
}

struct MinimalApp;

impl eframe::App for MinimalApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Cairn Text Minimal Test");
            ui.label("If you see this, basic egui rendering works on WASM.");
        });
    }
}

#[cfg(target_arch = "wasm32")]
fn main() {
    use eframe::wasm_bindgen::JsCast as _;
    std::panic::set_hook(Box::new(|info| {
        let msg = format!("Panic: {}", info);
        if let Some(window) = web_sys::window() {
            if let Some(document) = window.document() {
                if let Some(el) = document.get_element_by_id("loading") {
                    el.set_text_content(Some(&msg));
                }
            }
        }
    }));
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();
    let web_options = eframe::WebOptions::default();
    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window().unwrap().document().unwrap();
        let canvas = document
            .get_element_by_id("cairn-text")
            .unwrap()
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .unwrap();
        match eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|_cc| Ok(Box::new(MinimalApp))),
            )
            .await
        {
            Ok(_) => {
                if let Some(el) = document.get_element_by_id("loading") {
                    el.set_text_content(Some(""));
                }
            }
            Err(e) => {
                if let Some(el) = document.get_element_by_id("loading") {
                    el.set_text_content(Some(&format!("Failed: {:?}", e)));
                }
            }
        }
    });
}
