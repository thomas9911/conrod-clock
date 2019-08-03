use {WIN_H, WIN_W};
use gui;

// A function that runs the conrod loop.
pub fn run_conrod(
    event_rx: std::sync::mpsc::Receiver<conrod_core::event::Input>,
    render_tx: std::sync::mpsc::Sender<conrod_core::render::OwnedPrimitives>,
    events_loop_proxy: glium::glutin::EventsLoopProxy,
) {
    // Construct our `Ui`.
    let mut ui = conrod_core::UiBuilder::new([WIN_W as f64, WIN_H as f64])
        .theme(gui::theme())
        .build();

    // Add a `Font` to the `Ui`'s `font::Map` from file.
    // let assets = find_folder::Search::KidsThenParents(3, 5).for_folder("assets").unwrap();
    // let font_path = assets.join("NotoSans-Regular.ttf");
    // ui.fonts.insert_from_file(font_path).unwrap();
    let font_data: &[u8] = include_bytes!("../assets/NotoSans-Regular.ttf");
    let font = conrod_core::text::Font::from_bytes(font_data).unwrap();
    ui.fonts.insert(font);

    // A demonstration of some app state that we want to control with the conrod GUI.
    let mut app = gui::Clock::new();
    let nvidia = nvml_wrapper::NVML::init().unwrap();
    let graphics_card = nvidia.device_by_index(0).unwrap();

    // The `widget::Id` of each widget instantiated in `gui::gui`.
    let ids = gui::Ids::new(ui.widget_id_generator());

    // Many widgets require another frame to finish drawing after clicks or hovers, so we
    // insert an update into the conrod loop using this `bool` after each event.
    let mut needs_update = true;
    'conrod: loop {
        // Collect any pending events.
        let mut events = Vec::new();
        // app.update();
        while let Ok(event) = event_rx.try_recv() {
            events.push(event);
        }

        // If there are no events pending, wait for them.
        if events.is_empty() || !needs_update {
            match event_rx.recv() {
                Ok(event) => events.push(event),
                Err(_) => break 'conrod,
            };
        }

        needs_update = false;

        // Input each event into the `Ui`.
        for event in events {
            ui.handle_event(event);
            needs_update = true;
        }

        // Instantiate a GUI demonstrating every widget type provided by conrod.
        if needs_update{
            gui::gui(&mut ui.set_widgets(), &ids, &mut app, &graphics_card);
        }

        // Render the `Ui` to a list of primitives that we can send to the main thread for
        // display. Wakeup `winit` for rendering.
        if let Some(primitives) = ui.draw_if_changed() {
            if render_tx.send(primitives.owned()).is_err() || events_loop_proxy.wakeup().is_err() {
                break 'conrod;
            }
        }
    }}
