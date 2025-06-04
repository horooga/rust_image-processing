use druid::{
    commands,
    piet::ImageFormat,
    widget::{
        Align, Button, Container, CrossAxisAlignment, Flex, Image, Label, MainAxisAlignment,
        RadioGroup, Slider, ViewSwitcher,
    },
    Color, Data, FileDialogOptions, FileSpec, ImageBuf, Lens, UnitPoint, Widget, WidgetExt,
};
use image::{ImageReader, RgbImage};
use std::{
    path::Path,
    sync::{Arc, Mutex},
};

use crate::AppState;

static MAX_UNDOS_LEN: u8 = 3;

#[derive(Clone, Data, PartialEq)]
pub enum ProcessingOption {
    Dithering,
    Color,
}

#[derive(Clone, Data, Lens)]
pub struct DitheringParams {
    pub threshold: f64,
}

#[derive(Clone, Data, Lens)]
pub struct ColorParams {
    pub brightness: f64,
    pub contrast: f64,
}

struct FileOpenController;

impl<W: druid::Widget<AppState>> druid::widget::Controller<AppState, W> for FileOpenController {
    fn event(
        &mut self,
        child: &mut W,
        ctx: &mut druid::EventCtx,
        event: &druid::Event,
        data: &mut AppState,
        env: &druid::Env,
    ) {
        if let druid::Event::Command(cmd) = event {
            if let Some(file_info) = cmd.get(commands::OPEN_FILE) {
                if let Some(img_buf) = image_from_path(file_info.path()) {
                    if let Some(curr_img) = &data.img {
                        update_undos(&mut data.undos, curr_img.clone());
                    }
                    data.img = Some(img_buf.clone());
                    ctx.request_update();
                }
                ctx.set_handled();
                return;
            }
        }
        child.event(ctx, event, data, env);
    }
}

fn update_undos(undos: &mut Arc<Mutex<Vec<ImageBuf>>>, img: ImageBuf) {
    let mut images_lock = undos.lock().unwrap();
    images_lock.push(img);
    if images_lock.len() > MAX_UNDOS_LEN as usize {
        images_lock.remove(0);
    }
}

fn image_from_path<P: AsRef<Path>>(path: P) -> Option<ImageBuf> {
    let img = ImageReader::open(path).ok()?.decode().ok()?;
    let rgb_img: RgbImage = img.to_rgb8();
    let (width, height) = rgb_img.dimensions();
    let raw_bytes = rgb_img.into_raw();

    Some(ImageBuf::from_raw(
        raw_bytes,
        ImageFormat::Rgb,
        width as usize,
        height as usize,
    ))
}

pub fn build_ui() -> impl Widget<AppState> {
    let file_button = Button::new("Open Image").on_click(|ctx, _data: &mut AppState, _env| {
        let options = FileDialogOptions::new()
            .allowed_types(vec![FileSpec::new(
                "Image",
                &["png", "jpg", "jpeg", "webp"],
            )])
            .default_type(FileSpec::new("JPG", &["jpg"]));
        ctx.submit_command(commands::SHOW_OPEN_PANEL.with(options));
    });

    let processing_dropdown = RadioGroup::column(vec![
        ("Dithering".to_string(), ProcessingOption::Dithering),
        ("Color".to_string(), ProcessingOption::Color),
    ])
    .lens(AppState::selected_option);

    let params_view = ViewSwitcher::new(
        |data: &AppState, _env| data.selected_option.clone(),
        |selected, _data, _env| match selected {
            ProcessingOption::Dithering => dithering_params_ui()
                .lens(AppState::dithering_params)
                .boxed(),
            ProcessingOption::Color => color_params_ui().lens(AppState::color_params).boxed(),
        },
    );

    let image_view = ViewSwitcher::new(
        |data: &AppState, _env| data.img.clone(),
        |image_opt, _data, _env| {
            if let Some(img) = image_opt {
                Image::new(img.clone()).fix_size(350.0, 350.0).boxed()
            } else {
                Label::new("No image loaded").boxed()
            }
        },
    );

    let undo_button = Button::new("Undo").on_click(|_ctx, data: &mut AppState, _env| {
        let mut images_lock = data.undos.lock().unwrap();
        if let Some(prev_img) = images_lock.pop() {
            data.img = Some(prev_img);
        }
    });

    let right_col = Flex::column()
        .with_flex_child(Align::new(UnitPoint::CENTER, image_view), 1.0)
        .with_spacer(10.0)
        .with_flex_child(undo_button, 1.0);

    let left_col = Flex::column()
        .with_child(file_button)
        .with_spacer(10.0)
        .with_child(Label::new("Select Processing Option:"))
        .with_child(processing_dropdown)
        .with_spacer(10.0)
        .with_child(params_view);

    Container::new(
        Flex::row()
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .main_axis_alignment(MainAxisAlignment::Center)
            .with_flex_child(left_col, 1.0)
            .with_flex_child(right_col, 1.0)
            .controller(FileOpenController),
    )
    .background(Color::BLACK)
    .padding((30., 200.))
}

fn dithering_params_ui() -> impl Widget<DitheringParams> {
    Flex::column()
        .with_child(Label::new("Dithering Parameters"))
        .with_spacer(5.0)
        .with_child(Label::new("Threshold:").padding((0., 0., 0., 5.)))
        .with_child(
            Slider::new()
                .with_range(0.0, 1.0)
                .lens(DitheringParams::threshold),
        )
}

fn color_params_ui() -> impl Widget<ColorParams> {
    Flex::column()
        .with_child(Label::new("Color Parameters"))
        .with_spacer(5.0)
        .with_child(Label::new("Brightness:").padding((0., 0., 0., 5.)))
        .with_child(
            Slider::new()
                .with_range(0.0, 2.0)
                .lens(ColorParams::brightness),
        )
        .with_spacer(5.0)
        .with_child(Label::new("Contrast:").padding((0., 0., 0., 5.)))
        .with_child(
            Slider::new()
                .with_range(0.0, 2.0)
                .lens(ColorParams::contrast),
        )
}
