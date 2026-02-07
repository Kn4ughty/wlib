use std::time::Duration;

use smithay_client_toolkit::activation::RequestData;
use smithay_client_toolkit::reexports::calloop::{EventLoop, LoopHandle};
use smithay_client_toolkit::reexports::calloop_wayland_source::WaylandSource;
use smithay_client_toolkit::{
    activation::{ActivationHandler, ActivationState},
    compositor::{CompositorHandler, CompositorState},
    delegate_activation, delegate_compositor, delegate_keyboard, delegate_output, delegate_pointer,
    delegate_registry, delegate_seat, delegate_shm, delegate_xdg_shell, delegate_xdg_window,
    output::{OutputHandler, OutputState},
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    seat::{
        Capability, SeatHandler, SeatState,
        keyboard::{KeyEvent, KeyboardHandler, Keysym, Modifiers, RawModifiers},
        pointer::PointerHandler,
    },
    shell::{
        WaylandSurface,
        xdg::{
            XdgShell,
            window::{Window, WindowConfigure, WindowDecorations, WindowHandler},
        },
    },
    shm::{
        Shm, ShmHandler,
        slot::{Buffer, SlotPool},
    },
};

use wayland_client::{
    Connection, QueueHandle,
    globals::registry_queue_init,
    protocol::{wl_keyboard, wl_output, wl_pointer, wl_seat, wl_shm, wl_surface},
};

pub use smithay_client_toolkit::seat::{
    keyboard,
    pointer::{PointerEvent, PointerEventKind},
};

// Todo.
// -  Wrap draw type arguments with Frame struct
// -  More information to update like keyboard state.
// -  Potientially merge event and update, so update just recieves a queue of events it can iterate
//    through like pygame.get_events or whatever its called

pub trait WindowAble {
    /// Ran before draw so you can set up your scene with information from `context`
    /// You can include any requests you want wlib to do in in the returned output
    fn update(&mut self, context: Context) -> Option<WLibRequest>;

    /// Write your pixels to this buffer
    /// Since the window size is controlled by compositor, the width and height is given here.
    /// Foramt is always ARGB little endian. (So real byte order is BGRA)
    /// # Example
    /// ```rust
    /// fn draw(&mut self, buffer: &mut [u8], frame: wlib::FrameInfo) {
    ///     let width = frame.width;
    ///     let height = frame.height;
    ///
    ///     buffer
    ///         .chunks_exact_mut(4)
    ///         .enumerate()
    ///         .for_each(|(index, chunk)| {
    ///             let x = index as u32 % width;
    ///             let y = index as u32 / width;
    ///
    ///             let a: u8 = 0xFF;
    ///             let r: u8 = ((x as f32 / (width as f32)) * 255.0) as u8;
    ///             let g: u8 = 0;
    ///             let b: u8 = ((y as f32 / (height as f32)) * 255.0) as u8;
    ///
    ///             let array: &mut [u8; 4] = chunk.try_into().unwrap();
    ///             *array = [b, g, r, a]
    ///         });
    /// }
    /// ```
    fn draw(&mut self, buf: &mut [u8], frame_info: WindowSize);
}

#[derive(Debug, Clone)]
pub enum Event {
    KeyPress(KeyEvent),
    KeyRelease(KeyEvent),
    PointerEvent(PointerEvent),
    CloseRequested,
}

/// Some information you want to tell WLib.
pub enum WLibRequest {
    /// This is so you can prompt for the user to save their work or confirm exit etc, before
    /// closing the window
    CloseAccepted,
}

#[derive(Debug, Clone)]
pub struct Context {
    pub delta_time: std::time::Duration,
    pub pressed_keys: Vec<Keysym>,
    pub close_requested: bool,
    /// List of events since the last frame.
    /// Use it if you specfically need keyup/keydown events
    pub event_queue: Vec<Event>,
}

struct WindowManager {
    registry_state: RegistryState,
    seat_state: SeatState,
    output_state: OutputState,
    shm: Shm,
    xdg_activation: Option<ActivationState>,

    close_accepted: bool,
    first_configure: bool,
    pool: SlotPool,
    width: u32,
    height: u32,
    buffer: Option<Buffer>,
    window: Window,
    keyboard: Option<wl_keyboard::WlKeyboard>,
    keyboard_focus: bool,
    pointer: Option<wl_pointer::WlPointer>,
    loop_handle: LoopHandle<'static, WindowManager>,
    last_frame_time: Option<std::time::Instant>,

    managed_window: Box<dyn WindowAble>,
    settings: WLibSettings,
    context: Context,
}

pub struct WindowSize {
    pub width: u32,
    pub height: u32,
}

/// The available settings to configure window stuff
#[derive(Default)]
pub struct WLibSettings {
    /// If the window size should be static instead of updating when needed.
    window_static_size: Option<WindowSize>,

    /// Title of the window to show in in window selectors, decorations etc.
    window_title: String,

    /// App Id. Should be [reverse domain
    /// notation](https://en.wikipedia.org/wiki/Reverse_domain_name_notation)
    app_id: String,
}

impl WLibSettings {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_static_size(mut self, size: WindowSize) -> Self {
        self.window_static_size = Some(size);
        self
    }

    pub fn with_title(mut self, title: String) -> Self {
        self.window_title = title;
        self
    }

    pub fn with_app_id(mut self, id: String) -> Self {
        self.app_id = id;
        self
    }
}

pub fn run(state: Box<dyn WindowAble>, settings: WLibSettings) {
    // All Wayland apps start by connecting the compositor (server).
    let conn = Connection::connect_to_env().unwrap();

    // Enumerate the list of globals to get the protocols the server implements.
    let (globals, event_queue) = registry_queue_init(&conn).unwrap();
    let qh = event_queue.handle();
    let mut event_loop: EventLoop<WindowManager> =
        EventLoop::try_new().expect("Failed to initialize the event loop!");
    let loop_handle = event_loop.handle();
    WaylandSource::new(conn.clone(), event_queue)
        .insert(loop_handle)
        .unwrap();

    // The compositor (not to be confused with the server which is commonly called the compositor) allows
    // configuring surfaces to be presented.
    let compositor = CompositorState::bind(&globals, &qh).expect("wl_compositor not available");
    // For desktop platforms, the XDG shell is the standard protocol for creating desktop windows.
    let xdg_shell = XdgShell::bind(&globals, &qh).expect("xdg shell is not available");

    // Since we are not using the GPU in this example, we use wl_shm to allow software rendering to a buffer
    // we share with the compositor process.
    let shm = Shm::bind(&globals, &qh).expect("wl shm is not available.");
    // If the compositor supports xdg-activation it probably wants us to use it to get focus
    let xdg_activation = ActivationState::bind(&globals, &qh).ok();

    // A window is created from a surface.
    let surface = compositor.create_surface(&qh);

    // And then we can create the window.
    let window = xdg_shell.create_window(surface, WindowDecorations::RequestServer, &qh);

    // Configure the window, this may include hints to the compositor about the desired minimum size of the
    // window, app id for WM identification, the window title, etc.
    window.set_title(&settings.window_title);

    // GitHub does not let projects use the `org.github` domain but the `io.github` domain is fine.
    window.set_app_id(&settings.app_id);

    // In order for the window to be mapped, we need to perform an initial commit with no attached buffer.
    // For more info, see WaylandSurface::commit
    //
    // The compositor will respond with an initial configure that we can then use to present to the window with
    // the correct options.
    window.commit();

    // To request focus, we first need to request a token
    if let Some(activation) = xdg_activation.as_ref() {
        activation.request_token(
            &qh,
            RequestData {
                seat_and_serial: None,
                surface: Some(window.wl_surface().clone()),
                app_id: Some(String::from(
                    "io.github.smithay.client-toolkit.SimpleWindow",
                )),
            },
        )
    }

    let (width, height) = if let Some(ref dimensions) = settings.window_static_size {
        // If both min and max size are set to the same value, it means the size is static.
        // from (niri docs)[https://github.com/YaLTeR/niri/wiki/Floating-Windows], if this is the
        // case the window is set to be floating in tiling window managers
        window.set_min_size(Some((dimensions.width, dimensions.height)));
        window.set_max_size(Some((dimensions.width, dimensions.height)));

        (dimensions.width, dimensions.height)
    } else {
        (200, 200)
    };

    // We don't know how large the window will be yet, so lets assume the minimum size we suggested for the
    // initial memory allocation.
    let pool = SlotPool::new((width * height * 4) as usize, &shm).expect("Failed to create pool");

    let mut window_manager = WindowManager {
        // Seats and outputs may be hotplugged at runtime, therefore we need to setup a registry state to
        // listen for seats and outputs.
        registry_state: RegistryState::new(&globals),
        seat_state: SeatState::new(&globals, &qh),
        output_state: OutputState::new(&globals, &qh),
        shm,
        xdg_activation,

        close_accepted: false,
        first_configure: true,
        pool,
        width,
        height,
        buffer: None,
        window,
        keyboard: None,
        keyboard_focus: false,
        pointer: None,
        loop_handle: event_loop.handle(),
        last_frame_time: None,

        managed_window: state,
        context: Context {
            delta_time: std::time::Duration::from_millis(0),
            pressed_keys: Vec::new(),
            close_requested: false,
            event_queue: Vec::new(),
        },
        settings,
    };

    // We don't draw immediately, the configure will notify us when to first draw.
    loop {
        event_loop
            .dispatch(Duration::ZERO, &mut window_manager)
            .unwrap();

        if window_manager.close_accepted {
            println!("exiting example");
            break;
        }
    }
}

impl CompositorHandler for WindowManager {
    fn scale_factor_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _new_factor: i32,
    ) {
        // Not needed for this example.
    }

    fn transform_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _new_transform: wl_output::Transform,
    ) {
        // Not needed for this example.
    }

    fn frame(
        &mut self,
        conn: &Connection,
        qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _time: u32,
    ) {
        let now = std::time::Instant::now();
        let delta = self
            .last_frame_time
            .map(|last| now - last)
            .unwrap_or(Duration::ZERO);

        self.last_frame_time = Some(now);
        self.context.delta_time = delta;

        let request = self.managed_window.update(self.context.clone());
        self.handle_update(request);

        self.draw(conn, qh);

        self.context.event_queue.clear();
    }

    fn surface_enter(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _output: &wl_output::WlOutput,
    ) {
        // Not needed for this example.
    }

    fn surface_leave(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _output: &wl_output::WlOutput,
    ) {
        // Not needed for this example.
    }
}

impl OutputHandler for WindowManager {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }

    fn new_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }

    fn update_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }

    fn output_destroyed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }
}

impl WindowHandler for WindowManager {
    fn request_close(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &Window) {
        self.context.close_requested = true;
    }

    fn configure(
        &mut self,
        conn: &Connection,
        qh: &QueueHandle<Self>,
        _window: &Window,
        configure: WindowConfigure,
        _serial: u32,
    ) {
        println!("Window configured to: {:?}", configure);

        self.buffer = None;

        if self.settings.window_static_size.is_none() {
            self.width = configure.new_size.0.map(|v| v.get()).unwrap_or(256);
            self.height = configure.new_size.1.map(|v| v.get()).unwrap_or(256);
        }

        // self.width = configure.new_size.0.map(|v| v.get()).unwrap();
        // self.height = configure.new_size.1.map(|v| v.get()).unwrap();

        // Initiate the first draw.
        if self.first_configure {
            self.first_configure = false;
            self.draw(conn, qh);
        }
    }
}

impl ActivationHandler for WindowManager {
    type RequestData = RequestData;

    fn new_token(&mut self, token: String, _data: &Self::RequestData) {
        self.xdg_activation
            .as_ref()
            .unwrap()
            .activate::<WindowManager>(self.window.wl_surface(), token);
    }
}

impl SeatHandler for WindowManager {
    fn seat_state(&mut self) -> &mut SeatState {
        &mut self.seat_state
    }

    fn new_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}

    fn new_capability(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        seat: wl_seat::WlSeat,
        capability: Capability,
    ) {
        if capability == Capability::Keyboard && self.keyboard.is_none() {
            println!("Set keyboard capability");
            let keyboard = self
                .seat_state
                .get_keyboard_with_repeat(
                    qh,
                    &seat,
                    None,
                    self.loop_handle.clone(),
                    Box::new(|_state, _wl_kbd, event| {
                        println!("Repeat: {:?} ", event);
                    }),
                )
                .expect("Failed to create keyboard");

            self.keyboard = Some(keyboard);
        }

        if capability == Capability::Pointer && self.pointer.is_none() {
            println!("Set pointer capability");
            let pointer = self
                .seat_state
                .get_pointer(qh, &seat)
                .expect("Failed to create pointer");
            self.pointer = Some(pointer);
        }
    }

    fn remove_capability(
        &mut self,
        _conn: &Connection,
        _: &QueueHandle<Self>,
        _: wl_seat::WlSeat,
        capability: Capability,
    ) {
        if capability == Capability::Keyboard && self.keyboard.is_some() {
            println!("Unset keyboard capability");
            self.keyboard.take().unwrap().release();
        }

        if capability == Capability::Pointer && self.pointer.is_some() {
            println!("Unset pointer capability");
            self.pointer.take().unwrap().release();
        }
    }

    fn remove_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}
}

impl KeyboardHandler for WindowManager {
    fn enter(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        surface: &wl_surface::WlSurface,
        _: u32,
        _: &[u32],
        keysyms: &[Keysym],
    ) {
        if self.window.wl_surface() == surface {
            println!("Keyboard focus on window with pressed syms: {keysyms:?}");
            self.keyboard_focus = true;
        }
    }

    fn leave(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        surface: &wl_surface::WlSurface,
        _: u32,
    ) {
        if self.window.wl_surface() == surface {
            println!("Release keyboard focus on window");
            self.keyboard_focus = false;
        }
    }

    fn press_key(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        _: u32,
        event: KeyEvent,
    ) {
        println!("Key press: {event:?}");
        self.context
            .event_queue
            .push(Event::KeyPress(event.clone()));

        self.context.pressed_keys.dedup(); // Shouldnt ever be needed but could happen maybe??
        self.context.pressed_keys.push(event.keysym);
    }

    fn repeat_key(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        _: u32,
        event: KeyEvent,
    ) {
        println!("Key repeat: {event:?}");
    }

    fn release_key(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        _: u32,
        event: KeyEvent,
    ) {
        println!("Key release: {event:?}");
        self.context
            .event_queue
            .push(Event::KeyRelease(event.clone()));
        self.context.pressed_keys.retain(|&key| key != event.keysym);
    }

    fn update_modifiers(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        _serial: u32,
        modifiers: Modifiers,
        _raw_modifiers: RawModifiers,
        _layout: u32,
    ) {
        println!("Update modifiers: {modifiers:?}");
    }
}

impl PointerHandler for WindowManager {
    fn pointer_frame(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _pointer: &wl_pointer::WlPointer,
        events: &[PointerEvent],
    ) {
        for event in events {
            // Ignore events for other surfaces
            if &event.surface != self.window.wl_surface() {
                continue;
            }

            self.context
                .event_queue
                .push(Event::PointerEvent(event.clone()));
        }
    }
}

impl ShmHandler for WindowManager {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.shm
    }
}

impl WindowManager {
    pub fn draw(&mut self, _conn: &Connection, qh: &QueueHandle<Self>) {
        let width = self.width;
        let height = self.height;
        let stride = self.width as i32 * 4;

        let buffer = self.buffer.get_or_insert_with(|| {
            self.pool
                .create_buffer(
                    width as i32,
                    height as i32,
                    stride,
                    wl_shm::Format::Argb8888,
                )
                .expect("create buffer")
                .0
        });
        let canvas = match self.pool.canvas(buffer) {
            Some(canvas) => canvas,
            None => {
                // This should be rare, but if the compositor has not released the previous
                // buffer, we need double-buffering.
                let (second_buffer, canvas) = self
                    .pool
                    .create_buffer(
                        self.width as i32,
                        self.height as i32,
                        stride,
                        wl_shm::Format::Argb8888,
                    )
                    .expect("create buffer");
                *buffer = second_buffer;
                canvas
            }
        };

        // Draw to the window:
        self.managed_window.draw(
            canvas,
            WindowSize {
                width: self.width,
                height: self.height,
            },
        );

        // Damage the entire window
        self.window
            .wl_surface()
            .damage_buffer(0, 0, self.width as i32, self.height as i32);

        // Request our next frame
        self.window
            .wl_surface()
            .frame(qh, self.window.wl_surface().clone());

        // Attach and commit to present.
        buffer
            .attach_to(self.window.wl_surface())
            .expect("buffer attach");
        self.window.commit();
    }

    fn handle_update(&mut self, request: Option<WLibRequest>) {
        match request {
            Some(WLibRequest::CloseAccepted) => self.close_accepted = true,
            None => {}
        }
    }
}

delegate_compositor!(WindowManager);
delegate_output!(WindowManager);
delegate_shm!(WindowManager);

delegate_seat!(WindowManager);
delegate_keyboard!(WindowManager);
delegate_pointer!(WindowManager);

delegate_xdg_shell!(WindowManager);
delegate_xdg_window!(WindowManager);
delegate_activation!(WindowManager);

delegate_registry!(WindowManager);

impl ProvidesRegistryState for WindowManager {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }
    registry_handlers![OutputState, SeatState,];
}
