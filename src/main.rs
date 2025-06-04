mod image_procs;
mod types_n_convs;
mod ui;

use druid::piet::ImageBuf;
use druid::{AppLauncher, Data, Lens, LocalizedString, WindowDesc};
use std::sync::{Arc, Mutex};
use ui::{ColorParams, DitheringParams, ProcessingOption};

#[derive(Clone, Data, Lens)]
pub struct AppState {
    pub img: Option<ImageBuf>,
    pub undos: Arc<Mutex<Vec<ImageBuf>>>,

    pub selected_option: ProcessingOption,

    pub dithering_params: DitheringParams,
    pub color_params: ColorParams,
}

fn main() {
    let main_window = WindowDesc::new(ui::build_ui())
        .title(LocalizedString::new("Image Processing"))
        .window_size((800.0, 500.0));

    let initial_state = AppState {
        img: None,
        undos: Arc::new(Mutex::new(Vec::new())),
        selected_option: ProcessingOption::Dithering,
        dithering_params: DitheringParams { threshold: 0.5 },
        color_params: ColorParams {
            brightness: 1.0,
            contrast: 1.0,
        },
    };

    AppLauncher::with_window(main_window)
        .launch(initial_state)
        .expect("Failed to launch");
}
