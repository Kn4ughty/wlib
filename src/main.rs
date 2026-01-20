use wayland_client as wc;
use wlib;

const HEIGHT: u32 = 128;
const WIDTH: u32 = 128;

// fn xdg_wm_base_handle_ping() {
//     wayland_client::
// }

struct State;

impl wc::Dispatch<wc::protocol::wl_registry::WlRegistry, ()> for State {
    fn event(
        _state: &mut Self,
        _proxy: &wc::protocol::wl_registry::WlRegistry,
        event: <wc::protocol::wl_registry::WlRegistry as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &wayland_client::Connection,
        _qhandle: &wayland_client::QueueHandle<Self>,
    ) {
        if let wc::protocol::wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
        {
            println!("a thing!: [{}] {} (v{})", name, interface, version);
        }
    }
}

fn main() {
    println!("hloe world");
    let conn = wc::Connection::connect_to_env().unwrap();
    let display = conn.display();

    let mut event_queue = conn.new_event_queue();
    let queue_handle = event_queue.handle();

    let _registry = display.get_registry(&queue_handle, ());

    println!("done!. Advertised globals:");

    event_queue.roundtrip(&mut State).unwrap();
}
