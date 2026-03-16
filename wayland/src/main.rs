#![allow(dead_code, unused_imports, unused_variables)]

mod error;

use error::Result;
use wayland_client::{
    Connection, Dispatch, EventQueue, QueueHandle,
    protocol::{
        wl_buffer, wl_registry, wl_registry::WlRegistry, wl_surface, wl_surface::WlSurface,
    },
};
use wayland_protocols::xdg::shell::client::{xdg_surface, xdg_toplevel, xdg_wm_base};

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

impl Dispatch<wl_registry::WlRegistry, ()> for State {
    fn event(
        state: &mut Self,
        proxy: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        data: &(),
        conn: &Connection,
        qhandle: &QueueHandle<Self>,
    ) {
        log::debug!("<- {:?}", event);
    }
}
