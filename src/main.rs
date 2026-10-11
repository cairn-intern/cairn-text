use eframe::egui;
use std::path::PathBuf;

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
        Box::new(|_cc| Ok(Box::new(CairnEditor::default()))),
    )
}

#[cfg(target_arch = "wasm32")]
fn main() {
    use eframe::wasm_bindgen::JsCast as _;
    // Custom panic hook that shows errors in the DOM
    std::panic::set_hook(Box::new(|info| {
        let msg = format!("Panic: {}", info);
        if let Some(window) = web_sys::window() {
            if let Some(document) = window.document() {
                if let Some(el) = document.get_element_by_id("loading") {
                    el.set_text_content(Some(&msg));
                    // Make sure it's visible
                    if let Some(_) = el.dyn_ref::<web_sys::HtmlElement>() {
                        let style = web_sys::window()
                            .unwrap()
                            .document()
                            .unwrap()
                            .get_element_by_id("loading")
                            .unwrap();
                        let _ = js_sys::Reflect::set(
                            &style,
                            &"style".into(),
                            &"display:block;color:red;z-index:9999;".into(),
                        );
                    }
                }
            }
        }
    }));
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();
    let web_options = eframe::WebOptions::default();
    wasm_bindgen_futures::spawn_local(async {
        let show_error = |msg: &str| {
            if let Some(window) = web_sys::window() {
                if let Some(document) = window.document() {
                    if let Some(el) = document.get_element_by_id("loading") {
                        el.set_text_content(Some(msg));
                    }
                }
            }
        };

        let document = match web_sys::window().and_then(|w| w.document()) {
            Some(d) => d,
            None => {
                show_error("Error: no document");
                return;
            }
        };
        let canvas = match document
            .get_element_by_id("cairn-text")
            .and_then(|el| el.dyn_into::<web_sys::HtmlCanvasElement>().ok())
        {
            Some(c) => c,
            None => {
                show_error("Error: canvas not found");
                return;
            }
        };
        match eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|_cc| Ok(Box::new(CairnEditor::default()))),
            )
            .await
        {
            Ok(_) => {
                if let Some(el) = document.get_element_by_id("loading") {
                    el.set_text_content(Some(""));
                }
            }
            Err(e) => {
                show_error(&format!("Failed to start: {:?}. WebGL2 may not be available.", e));
            }
        }
    });
}

struct Tab {
    path: Option<PathBuf>,
    content: String,
    dirty: bool,
}

impl Tab {
    fn new() -> Self {
        Self {
            path: None,
            content: String::new(),
            dirty: false,
        }
    }

    fn title(&self) -> String {
        let name = self
            .path
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("Untitled");
        if self.dirty {
            format!("{} *", name)
        } else {
            name.to_string()
        }
    }
}

struct CairnEditor {
    tabs: Vec<Tab>,
    active: usize,
    status: String,
}

impl Default for CairnEditor {
    fn default() -> Self {
        Self {
            tabs: vec![Tab::new()],
            active: 0,
            status: String::new(),
        }
    }
}

impl CairnEditor {
    fn active_tab_mut(&mut self) -> &mut Tab {
        &mut self.tabs[self.active]
    }

    fn open_file(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(path) = rfd::FileDialog::new().pick_file() {
            match std::fs::read_to_string(&path) {
                Ok(content) => {
                    self.tabs.push(Tab {
                        path: Some(path),
                        content,
                        dirty: false,
                    });
                    self.active = self.tabs.len() - 1;
                    self.status = "File opened.".to_string();
                }
                Err(e) => self.status = format!("Open failed: {}", e),
            }
        }
        #[cfg(target_arch = "wasm32")]
        {
            self.status = "File open not supported in browser.".to_string();
        }
    }

    fn save_file(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let path_opt = self.active_tab_mut().path.clone();
            if let Some(path) = path_opt {
                let content = self.active_tab_mut().content.clone();
                match std::fs::write(&path, &content) {
                    Ok(()) => {
                        self.active_tab_mut().dirty = false;
                        self.status = format!("Saved {}.", path.display());
                    }
                    Err(e) => self.status = format!("Save failed: {}", e),
                }
            } else {
                self.save_file_as();
            }
        }
        #[cfg(target_arch = "wasm32")]
        {
            self.status = "File save not supported in browser.".to_string();
        }
    }

    fn save_file_as(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(path) = rfd::FileDialog::new().save_file() {
            let content = self.active_tab_mut().content.clone();
            match std::fs::write(&path, &content) {
                Ok(()) => {
                    let tab = self.active_tab_mut();
                    tab.path = Some(path.clone());
                    tab.dirty = false;
                    self.status = format!("Saved {}.", path.display());
                }
                Err(e) => self.status = format!("Save failed: {}", e),
            }
        }
        #[cfg(target_arch = "wasm32")]
        {
            self.status = "File save not supported in browser.".to_string();
        }
    }

    fn new_tab(&mut self) {
        self.tabs.push(Tab::new());
        self.active = self.tabs.len() - 1;
    }

    fn close_tab(&mut self) {
        if self.tabs.len() > 1 {
            self.tabs.remove(self.active);
            if self.active >= self.tabs.len() {
                self.active = self.tabs.len() - 1;
            }
        } else {
            self.tabs[0] = Tab::new();
            self.active = 0;
        }
    }
}

impl eframe::App for CairnEditor {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("menu").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New Tab").clicked() {
                        self.new_tab();
                        ui.close();
                    }
                    if ui.button("Open...").clicked() {
                        self.open_file();
                        ui.close();
                    }
                    if ui.button("Save").clicked() {
                        self.save_file();
                        ui.close();
                    }
                    if ui.button("Save As...").clicked() {
                        self.save_file_as();
                        ui.close();
                    }
                    if ui.button("Close Tab").clicked() {
                        self.close_tab();
                        ui.close();
                    }
                });
            });
        });

        egui::TopBottomPanel::top("tabs").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let mut close_idx: Option<usize> = None;
                let mut new_active: Option<usize> = None;
                for (i, tab) in self.tabs.iter().enumerate() {
                    let selected = i == self.active;
                    if ui.selectable_label(selected, tab.title()).clicked() {
                        new_active = Some(i);
                    }
                    if ui.small_button("x").clicked() {
                        close_idx = Some(i);
                    }
                }
                if let Some(i) = new_active {
                    self.active = i;
                }
                if let Some(idx) = close_idx {
                    if self.tabs.len() > 1 {
                        self.tabs.remove(idx);
                        if self.active >= self.tabs.len() {
                            self.active = self.tabs.len() - 1;
                        }
                    }
                }
            });
        });

        egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(&self.status);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let tab = self.active_tab_mut();
            let response = ui.add_sized(
                ui.available_size(),
                egui::TextEdit::multiline(&mut tab.content)
                    .font(egui::TextStyle::Monospace)
                    .code_editor()
                    .lock_focus(true),
            );
            if response.changed() {
                tab.dirty = true;
                self.status.clear();
            }
        });

        ctx.input_mut(|i| {
            if i.consume_key(egui::Modifiers::COMMAND, egui::Key::S) {
                self.save_file();
            }
            if i.consume_key(egui::Modifiers::COMMAND, egui::Key::O) {
                self.open_file();
            }
            if i.consume_key(egui::Modifiers::COMMAND, egui::Key::T) {
                self.new_tab();
            }
            if i.consume_key(egui::Modifiers::COMMAND, egui::Key::W) {
                self.close_tab();
            }
        });
    }
}
