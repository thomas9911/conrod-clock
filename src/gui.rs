//! This crate is used for sharing a few items between the conrod examples.
//!
//! The module contains:
//!
//! - `pub struct DemoApp` as a demonstration of some state we want to change.
//! - `pub fn gui` as a demonstration of all widgets, some of which mutate our `DemoApp`.
//! - `pub struct Ids` - a set of all `widget::Id`s used in the `gui` fn.
//!
//! By sharing these items between these examples, we can test and ensure that the different events
//! and drawing backends behave in the same manner.
#![allow(dead_code)]

extern crate rand;

// use my_widgets;

pub const WIN_W: u32 = 600;
pub const WIN_H: u32 = 420;

/// A demonstration of some application state we want to control with a conrod GUI.
pub struct Clock(chrono::DateTime<chrono::Local>);

impl Clock {
    pub fn new() -> Clock {
        Clock {
            0: chrono::Local::now(),
        }
    }
    pub fn update(&mut self) {
        self.0 = chrono::Local::now()
    }
}

impl std::fmt::Display for Clock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.format("%k:%M:%S"))
    }
}

/// A set of reasonable stylistic defaults that works for the `gui` below.
pub fn theme() -> conrod_core::Theme {
    use conrod_core::position::{Align, Direction, Padding, Position, Relative};
    conrod_core::Theme {
        name: "Demo Theme".to_string(),
        padding: Padding::none(),
        x_position: Position::Relative(Relative::Align(Align::Start), None),
        y_position: Position::Relative(Relative::Direction(Direction::Backwards, 20.0), None),
        background_color: conrod_core::color::DARK_CHARCOAL,
        shape_color: conrod_core::color::LIGHT_CHARCOAL,
        border_color: conrod_core::color::BLACK,
        border_width: 0.0,
        label_color: conrod_core::color::WHITE,
        font_id: None,
        font_size_large: 26,
        font_size_medium: 18,
        font_size_small: 12,
        widget_styling: conrod_core::theme::StyleMap::default(),
        mouse_drag_threshold: 0.0,
        double_click_threshold: std::time::Duration::from_millis(500),
    }
}

// Generate a unique `WidgetId` for each widget.
conrod_core::widget_ids! {
    pub struct Ids {
        // The scrollable canvas.
        canvas,
        // The title and introduction widgets.
        title,
        card_info1,
        card_info2,
        card_info3,
        temperature,
        fan_speed,
        memory_used,
        memory_free,
        memory_total,
        memory_utilization,
        gpu_utilization,
        power_usage,
        circle,
    }
}

macro_rules! str_line {
    ($k:ident) => {
        if $k < 100000 {
            format!("{}: {}", stringify!($k).split('_').collect::<Vec<&str>>().join(" "), $k)
        } else {
            format!("{}: {:.4e}", stringify!($k).split('_').collect::<Vec<&str>>().join(" "), $k as f64)
        }
    };
    ($k:ident, $u:expr) => {
        if $k < 100000 {
            format!("{}: {}{}", stringify!($k).split('_').collect::<Vec<&str>>().join(" "), $k, $u)
        } else {
            format!("{}: {:.4e}{}", stringify!($k).split('_').collect::<Vec<&str>>().join(" "), $k as f64, $u)
        }
    };
}

/// Instantiate a GUI demonstrating every widget available in conrod.
pub fn gui(ui: &mut conrod_core::UiCell, ids: &Ids, app: &mut Clock, card: &nvml_wrapper::Device) {
    use conrod_core::{widget, Positionable, Widget};

    const MARGIN: conrod_core::Scalar = 30.0;
    const TITLE_SIZE: conrod_core::FontSize = 120;
    const FONT_SIZE: conrod_core::FontSize = 24;

    // `Canvas` is a widget that provides some basic functionality for laying out children widgets.
    // By default, its size is the size of the window. We'll use this as a background for the
    // following widgets, as well as a scrollable container for the children widgets.
    // const TITLE: &'static str = "All Widgets";

    widget::Canvas::new()
        .pad(MARGIN)
        .scroll_kids_vertically()
        .set(ids.canvas, ui);
    ////////////////
    ///// TEXT /////
    ////////////////

    app.update();
    // We'll demonstrate the `Text` primitive widget by using it to draw a title and an
    // introduction to the example.
    let time = format!("{}", app);
    widget::Text::new(&time)
        .font_size(TITLE_SIZE)
        // .mid_top_of(ids.canvas)
        .middle_of(ids.canvas)
        .set(ids.title, ui);

    let memory_clock = card
        .clock_info(nvml_wrapper::enum_wrappers::device::Clock::Memory)
        .unwrap_or(0);
    let graphics_clock = card
        .clock_info(nvml_wrapper::enum_wrappers::device::Clock::Graphics)
        .unwrap_or(0);
    let video_clock = card
        .clock_info(nvml_wrapper::enum_wrappers::device::Clock::Video)
        .unwrap_or(0);
    let temperature = card
        .temperature(nvml_wrapper::enum_wrappers::device::TemperatureSensor::Gpu)
        .unwrap_or(0);
    let fan_speed = card.fan_speed().unwrap_or(0);

    let nvml_wrapper::struct_wrappers::device::MemoryInfo {
        used: mut memory_used,
        free: mut memory_free,
        total: mut memory_total,
    } = card
        .memory_info()
        .unwrap_or(nvml_wrapper::struct_wrappers::device::MemoryInfo {
            free: 0,
            total: 0,
            used: 0,
        });
    // memory_used/=10u64.pow(6);
    // memory_free/=10u64.pow(6);
    // memory_total/=10u64.pow(6);
    memory_used/=1024u64.pow(2);
    memory_free/=1024u64.pow(2);
    memory_total/=1024u64.pow(2);
    let nvml_wrapper::struct_wrappers::device::Utilization {
        memory: memory_utilization,
        gpu: gpu_utilization,
    } = card
        .utilization_rates()
        .unwrap_or(nvml_wrapper::struct_wrappers::device::Utilization { memory: 0, gpu: 0 });

    let power_usage = card.power_usage().unwrap_or(0)/1000;

    widget::Text::new(&str_line!(memory_used, " MB"))
        .font_size(FONT_SIZE)
        // .mid_top_of(ids.canvas)
        .bottom_left_of(ids.canvas)
        .set(ids.memory_used, ui);
    widget::Text::new(&str_line!(memory_free, " MB"))
        .font_size(FONT_SIZE)
        // .mid_top_of(ids.canvas)
        .mid_bottom_of(ids.canvas)
        .set(ids.memory_free, ui);
    widget::Text::new(&str_line!(memory_total, " MB"))
        .font_size(FONT_SIZE)
        // .mid_top_of(ids.canvas)
        .bottom_right_of(ids.canvas)
        .set(ids.memory_total, ui);

    widget::Text::new(&str_line!(memory_clock, " MHz"))
        .font_size(FONT_SIZE)
        .y_relative_to(ids.memory_used, MARGIN)
        .set(ids.card_info1, ui);
    widget::Text::new(&str_line!(graphics_clock, " MHz"))
        .font_size(FONT_SIZE)
        .y_relative_to(ids.memory_free, MARGIN)
        .set(ids.card_info2, ui);
    widget::Text::new(&str_line!(video_clock, " MHz"))
        .font_size(FONT_SIZE)
        .y_relative_to(ids.memory_total, MARGIN)
        .set(ids.card_info3, ui);

    widget::Text::new(&str_line!(temperature, " °C"))
        .font_size(FONT_SIZE)
        .y_relative_to(ids.card_info1, MARGIN)
        .set(ids.temperature, ui);
    widget::Text::new(&str_line!(fan_speed, "%"))
        .font_size(FONT_SIZE)
        .y_relative_to(ids.card_info2, MARGIN)
        .set(ids.fan_speed, ui);
    widget::Text::new(&str_line!(power_usage, " W"))
        .font_size(FONT_SIZE)
        .y_relative_to(ids.card_info3, MARGIN)
        .set(ids.power_usage, ui);

    widget::Text::new(&str_line!(memory_utilization, "%"))
        .font_size(FONT_SIZE)
        .y_relative_to(ids.temperature, MARGIN)
        .set(ids.memory_utilization, ui);
    widget::Text::new(&str_line!(gpu_utilization, "%"))
        .font_size(FONT_SIZE)
        .y_relative_to(ids.power_usage, MARGIN)
        .set(ids.gpu_utilization, ui);


    // for event in my_widgets::CircularButton::new()
    //     .middle_of(ids.canvas)
    //     .set(ids.circle, ui)
    // {
    //     println!("Click!");

    // }
}
