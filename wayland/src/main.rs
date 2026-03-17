mod error;

use std::{
    fs::File,
    io::{BufWriter, Write},
    os::fd::AsFd,
};

use cairo::{Context, Format, ImageSurface};
use error::Result;
use wayland_client::{
    Connection, Dispatch, EventQueue, QueueHandle, WEnum, delegate_noop,
    protocol::{
        wl_buffer::WlBuffer, wl_compositor::WlCompositor, wl_keyboard, wl_keyboard::WlKeyboard,
        wl_pointer, wl_pointer::WlPointer, wl_registry, wl_registry::WlRegistry, wl_seat,
        wl_seat::WlSeat, wl_shm, wl_shm::WlShm, wl_shm_pool::WlShmPool, wl_surface::WlSurface,
    },
};
use wayland_protocols::xdg::shell::client::{
    xdg_surface, xdg_surface::XdgSurface, xdg_toplevel, xdg_toplevel::XdgToplevel, xdg_wm_base,
    xdg_wm_base::XdgWmBase,
};

struct State {
    surface: Option<WlSurface>,
    buffer: Option<WlBuffer>,
    wm_base: Option<XdgWmBase>,
    xdg_surface: Option<(XdgSurface, XdgToplevel)>,
    seat: Option<WlSeat>,
    running: bool,
    configured: bool,
    cursor_x: f64,
    cursor_y: f64,
}

fn main() -> Result<()> {
    env_logger::init();

    let conn = Connection::connect_to_env()?;
    let mut event_queue: EventQueue<State> = conn.new_event_queue();
    let queue_handle = event_queue.handle();
    let display = conn.display();
    display.get_registry(&queue_handle, ());

    let mut state = State {
        running: true,
        surface: None,
        buffer: None,
        wm_base: None,
        xdg_surface: None,
        configured: false,
        seat: None,
        cursor_x: 0.0,
        cursor_y: 0.0,
    };

    println!("Starting example window app, press <ESC> to quit.");

    while state.running {
        event_queue.blocking_dispatch(&mut state)?;
    }

    Ok(())
}

// ignore events for the object types below
delegate_noop!(State: ignore WlCompositor);
delegate_noop!(State: ignore WlSurface);
delegate_noop!(State: ignore WlShm);
delegate_noop!(State: ignore WlShmPool);
delegate_noop!(State: ignore WlBuffer);

impl State {
    /// Initializes the XDG surface and toplevel for the window.
    ///
    /// This method retrieves the XDG surface from the window manager base and creates a toplevel
    /// window. It sets the title of the window and commits the surface to apply changes.
    ///
    /// # Arguments
    ///
    /// * `queue_handle` - The event queue handle used to create the new XDG objects.
    ///
    /// # Panics
    ///
    /// Panics if `wm_base` or `surface` have not been initialized in the `State`.
    fn init_xdg_surface(&mut self, queue_handle: &QueueHandle<State>) {
        let wm_base = self.wm_base.as_ref().unwrap();
        let surface = self.surface.as_ref().unwrap();

        let xdg_surface = wm_base.get_xdg_surface(surface, queue_handle, ());
        let toplevel = xdg_surface.get_toplevel(queue_handle, ());
        toplevel.set_title("A fantastic window!".into());
        surface.commit();

        self.xdg_surface = Some((xdg_surface, toplevel));
    }
}

impl Dispatch<WlRegistry, ()> for State {
    fn event(
        state: &mut Self,
        registry: &WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &Connection,
        queue_handle: &QueueHandle<Self>,
    ) {
        if let wl_registry::Event::Global {
            name, interface, ..
        } = event
        {
            match &interface[..] {
                "wl_compositor" => {
                    let compositor = registry.bind::<WlCompositor, _, _>(name, 1, queue_handle, ());
                    let surface = compositor.create_surface(queue_handle, ());
                    state.surface = Some(surface);
                    // check if we still need to init the XDG surface
                    if state.wm_base.is_some() && state.xdg_surface.is_none() {
                        state.init_xdg_surface(queue_handle);
                    }
                }
                "wl_shm" => {
                    let shm = registry.bind::<WlShm, _, _>(name, 1, queue_handle, ());
                    let (init_w, init_h) = (320, 240);
                    let mut file = tempfile::tempfile().unwrap();
                    draw(&mut file, (init_w, init_h));
                    let pool = shm.create_pool(
                        file.as_fd(),
                        (init_w * init_h * 4) as i32,
                        queue_handle,
                        (),
                    );
                    let buffer = pool.create_buffer(
                        0,
                        init_w as i32,
                        init_h as i32,
                        (init_w * 4) as i32,
                        wl_shm::Format::Argb8888,
                        queue_handle,
                        (),
                    );
                    state.buffer = Some(buffer.clone());
                    if state.configured {
                        let surface = state.surface.as_ref().unwrap();
                        surface.attach(Some(&buffer), 0, 0);
                        surface.commit();
                    }
                }
                "wl_seat" => {
                    let seat = registry.bind::<WlSeat, _, _>(name, 1, queue_handle, ());
                    state.seat = Some(seat);
                }
                "xdg_wm_base" => {
                    let wm_base = registry.bind::<XdgWmBase, _, _>(name, 1, queue_handle, ());
                    state.wm_base = Some(wm_base);
                    // check if we still need to init the XDG surface
                    if state.wm_base.is_some() && state.xdg_surface.is_none() {
                        state.init_xdg_surface(queue_handle);
                    }
                }
                _ => {
                    log::debug!("ignore global object: {interface}");
                }
            }
        }
    }
}

fn draw(tmp: &mut File, (buf_x, buf_y): (u32, u32)) {
    let width = buf_x as f64;
    let height = buf_y as f64;

    let mut surface = ImageSurface::create(Format::ARgb32, buf_x as i32, buf_y as i32).unwrap();
    // This block causes the Cairo context to drop it's mutable surface reference at the end of it.
    // This is needed because in order to write the surface data into the `tmp` file, exclusive
    // access to the surface data is needed.
    {
        let cr = Context::new(&surface).unwrap();
        // 1. Fill background (Open Look Gray)
        cr.set_source_rgb(0.8, 0.8, 0.8);
        cr.paint().unwrap();
        // 2. Draw 3D Bevel Border (Window Frame)
        let border_w = 4.0;
        draw_bevel(&cr, 0.0, 0.0, width, height, border_w, false);
        // 3. Header/Title Bar
        let header_h = 26.0;
        // Window Button (Menu) - Top Left (Recessed)
        let btn_pad = 4.0;
        let btn_size = header_h - 2.0 * btn_pad;
        let btn_x = border_w + btn_pad;
        let btn_y = border_w + btn_pad;
        draw_bevel(&cr, btn_x, btn_y, btn_size, btn_size, 2.0, true);
        // Draw the "Menu Mark" (simple recessed line/box look)
        cr.set_source_rgb(0.0, 0.0, 0.0);
        cr.rectangle(
            btn_x + 3.0,
            btn_y + btn_size / 2.0 - 1.0,
            btn_size - 6.0,
            2.0,
        );
        cr.fill().unwrap();
        // Title Text Area (Centered)
        cr.select_font_face("Sans", cairo::FontSlant::Normal, cairo::FontWeight::Bold);
        cr.set_font_size(13.0);
        let title = "Wayland Window";
        let extents = cr.text_extents(title).unwrap();
        let text_x = (width - extents.width()) / 2.0;
        let text_y = border_w + header_h / 2.0 + extents.height() / 2.0 - 2.0;
        cr.set_source_rgb(0.0, 0.0, 0.0);
        cr.move_to(text_x, text_y);
        cr.show_text(title).unwrap();
        // 4. Content Area
        // Separator line
        let line_y = border_w + header_h;
        cr.set_source_rgb(0.0, 0.0, 0.0);
        cr.set_line_width(1.0);
        cr.move_to(border_w, line_y);
        cr.line_to(width - border_w, line_y);
        cr.stroke().unwrap();
        // Draw content background (White)
        cr.set_source_rgb(1.0, 1.0, 1.0);
        cr.rectangle(
            border_w,
            line_y + 1.0,
            width - 2.0 * border_w,
            height - line_y - 1.0 - border_w,
        );
        cr.fill().unwrap();
    }

    surface.flush();
    let data = surface.data().unwrap();

    let mut buf = BufWriter::new(tmp);
    buf.write_all(&data).unwrap();
    buf.flush().unwrap();
}

/// Helper to draw a 3D bevel box
fn draw_bevel(cr: &Context, x: f64, y: f64, w: f64, h: f64, bw: f64, sunk: bool) {
    let light = (1.0, 1.0, 1.0);
    let shadow = (0.4, 0.4, 0.4);

    let (c_top_left, c_bot_right) = if sunk {
        (shadow, light)
    } else {
        (light, shadow)
    };

    // Top & Left
    cr.set_source_rgb(c_top_left.0, c_top_left.1, c_top_left.2);
    // Top trapezoid
    cr.move_to(x, y);
    cr.line_to(x + w, y);
    cr.line_to(x + w - bw, y + bw);
    cr.line_to(x + bw, y + bw);
    cr.close_path();
    cr.fill().unwrap();
    // Left trapezoid
    cr.move_to(x, y);
    cr.line_to(x + bw, y + bw);
    cr.line_to(x + bw, y + h - bw);
    cr.line_to(x, y + h);
    cr.close_path();
    cr.fill().unwrap();
    // Bottom & Right
    cr.set_source_rgb(c_bot_right.0, c_bot_right.1, c_bot_right.2);
    // Bottom trapezoid
    cr.move_to(x, y + h);
    cr.line_to(x + w, y + h);
    cr.line_to(x + w - bw, y + h - bw);
    cr.line_to(x + bw, y + h - bw);
    cr.close_path();
    cr.fill().unwrap();
    // Right trapezoid
    cr.move_to(x + w, y);
    cr.line_to(x + w, y + h);
    cr.line_to(x + w - bw, y + h - bw);
    cr.line_to(x + w - bw, y + bw);
    cr.close_path();
    cr.fill().unwrap();
}

impl Dispatch<XdgSurface, ()> for State {
    fn event(
        state: &mut Self,
        xdg_surface: &XdgSurface,
        event: xdg_surface::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let xdg_surface::Event::Configure { serial, .. } = event {
            xdg_surface.ack_configure(serial);
            state.configured = true;
            let surface = state.surface.as_ref().unwrap();
            if let Some(ref buffer) = state.buffer {
                surface.attach(Some(buffer), 0, 0);
                surface.commit();
            }
        }
    }
}

impl Dispatch<XdgWmBase, ()> for State {
    fn event(
        _: &mut Self,
        wm_base: &XdgWmBase,
        event: xdg_wm_base::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let xdg_wm_base::Event::Ping { serial, .. } = event {
            wm_base.pong(serial);
        }
    }
}

impl Dispatch<XdgToplevel, ()> for State {
    fn event(
        state: &mut Self,
        _: &XdgToplevel,
        event: xdg_toplevel::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let xdg_toplevel::Event::Close = event {
            state.running = false; // quit the app
        }
    }
}

impl Dispatch<WlSeat, ()> for State {
    fn event(
        _: &mut Self,
        seat: &WlSeat,
        event: wl_seat::Event,
        _: &(),
        _: &Connection,
        queue_handle: &QueueHandle<Self>,
    ) {
        if let wl_seat::Event::Capabilities {
            capabilities: WEnum::Value(capabilities),
            ..
        } = event
        {
            if capabilities.contains(wl_seat::Capability::Keyboard) {
                seat.get_keyboard(queue_handle, ());
            }
            if capabilities.contains(wl_seat::Capability::Pointer) {
                seat.get_pointer(queue_handle, ());
            }
        }
    }
}

impl Dispatch<WlKeyboard, ()> for State {
    fn event(
        state: &mut Self,
        _: &WlKeyboard,
        event: wl_keyboard::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let wl_keyboard::Event::Key { key, .. } = event
            && key == 1
        {
            state.running = false; // quit the app
        }
    }
}

impl Dispatch<WlPointer, ()> for State {
    fn event(
        state: &mut Self,
        _: &WlPointer,
        event: wl_pointer::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            wl_pointer::Event::Enter {
                surface_x,
                surface_y,
                ..
            } => {
                state.cursor_x = surface_x;
                state.cursor_y = surface_y;
            }
            wl_pointer::Event::Motion {
                surface_x,
                surface_y,
                ..
            } => {
                state.cursor_x = surface_x;
                state.cursor_y = surface_y;
            }
            wl_pointer::Event::Button {
                state: WEnum::Value(wl_pointer::ButtonState::Pressed),
                serial,
                ..
            } if state.cursor_y < 30.0 => {
                // Title bar clicked
                if state.cursor_x > 290.0 {
                    state.running = false;
                } else if let (Some((_, toplevel)), Some(seat)) = (&state.xdg_surface, &state.seat)
                {
                    toplevel._move(seat, serial);
                }
            }
            _ => {}
        }
    }
}
