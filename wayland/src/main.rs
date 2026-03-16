#![allow(dead_code, unused_imports, unused_variables)]

mod error;

use error::Result;
use wayland_client::{
    Connection, Dispatch, EventQueue, QueueHandle, WEnum, delegate_noop,
    protocol::{
        wl_buffer, wl_compositor, wl_keyboard, wl_keyboard::WlKeyboard, wl_registry,
        wl_registry::WlRegistry, wl_seat, wl_shm, wl_shm_pool, wl_surface, wl_surface::WlSurface,
    },
};
use wayland_protocols::xdg::shell::client::{
    xdg_surface, xdg_surface::XdgSurface, xdg_toplevel, xdg_toplevel::XdgToplevel, xdg_wm_base,
    xdg_wm_base::XdgWmBase,
};

struct State {
    running: bool,
    surface: Option<wl_surface::WlSurface>,
    buffer: Option<wl_buffer::WlBuffer>,
    wm_base: Option<xdg_wm_base::XdgWmBase>,
    xdg_surface: Option<(xdg_surface::XdgSurface, xdg_toplevel::XdgToplevel)>,
    configured: bool,
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
    };

    println!("Starting example window app, press <ESC> to quit.");

    while state.running {
        event_queue.blocking_dispatch(&mut state)?;
    }

    Ok(())
}

// ignore events for the object types below
delegate_noop!(State: ignore wl_compositor::WlCompositor);
delegate_noop!(State: ignore wl_surface::WlSurface);
delegate_noop!(State: ignore wl_shm::WlShm);
delegate_noop!(State: ignore wl_shm_pool::WlShmPool);
delegate_noop!(State: ignore wl_buffer::WlBuffer);

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

impl Dispatch<wl_registry::WlRegistry, ()> for State {
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        data: &(),
        conn: &Connection,
        queue_handle: &QueueHandle<Self>,
    ) {
        if let wl_registry::Event::Global {
            name, interface, ..
        } = event
        {
            match &interface[..] {
                "wl_compositor" => {
                    let compositor = registry.bind::<wl_compositor::WlCompositor, _, _>(
                        name,
                        1,
                        queue_handle,
                        (),
                    );
                    let surface = compositor.create_surface(queue_handle, ());
                    state.surface = Some(surface);
                    // check if we still need to init the XDG surface
                    if state.wm_base.is_some() && state.xdg_surface.is_none() {
                        state.init_xdg_surface(queue_handle);
                    }
                }
                "wl_seat" => {
                    registry.bind::<wl_seat::WlSeat, _, _>(name, 1, queue_handle, ());
                }
                "xdg_wm_base" => {
                    let wm_base =
                        registry.bind::<xdg_wm_base::XdgWmBase, _, _>(name, 1, queue_handle, ());
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

impl Dispatch<xdg_surface::XdgSurface, ()> for State {
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

impl Dispatch<xdg_wm_base::XdgWmBase, ()> for State {
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

impl Dispatch<xdg_toplevel::XdgToplevel, ()> for State {
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

impl Dispatch<wl_seat::WlSeat, ()> for State {
    fn event(
        _: &mut Self,
        seat: &wl_seat::WlSeat,
        event: wl_seat::Event,
        _: &(),
        _: &Connection,
        queue_handle: &QueueHandle<Self>,
    ) {
        if let wl_seat::Event::Capabilities {
            capabilities: WEnum::Value(capabilities),
            ..
        } = event
            && capabilities.contains(wl_seat::Capability::Keyboard)
        {
            seat.get_keyboard(queue_handle, ());
        }
    }
}

impl Dispatch<wl_keyboard::WlKeyboard, ()> for State {
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
