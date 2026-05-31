
use wayland_client::{Connection, Dispatch, EventQueue};
use wayland_client::protocol::wl_registry;
use wayland_protocols::ext::foreign_toplevel_list::v1::client::ext_foreign_toplevel_list_v1;
use wayland_protocols::ext::foreign_toplevel_list::v1::client::ext_foreign_toplevel_handle_v1;
use wayland_protocols::ext::foreign_toplevel_list::v1::client::ext_foreign_toplevel_list_v1::ExtForeignToplevelListV1;
use wayland_protocols::ext::foreign_toplevel_list::v1::client::ext_foreign_toplevel_handle_v1::ExtForeignToplevelHandleV1;

struct AppState { }

impl Dispatch<ExtForeignToplevelHandleV1, ()> for AppState {
    fn event(
        state: &mut AppState,
        _proxy: &ExtForeignToplevelHandleV1,
        _event: ext_foreign_toplevel_handle_v1::Event,
        _udata: &(),
        _conn: &wayland_client::Connection,
        _qhandle: &wayland_client::QueueHandle<AppState>,
    ) {
        match _event {
            ext_foreign_toplevel_handle_v1::Event::Title { title }  => {
                println!("ext_foreign_toplevel_handle_v1::Event::Title: {}", title);
            }
            ext_foreign_toplevel_handle_v1::Event::AppId { app_id }  => {
                println!("ext_foreign_toplevel_handle_v1::Event::AppId: {}", app_id);
            }
            ext_foreign_toplevel_handle_v1::Event::Done  => {
                println!("ext_foreign_toplevel_handle_v1::Event::Done");
            }
            _ => { }
        }
    }
}

impl Dispatch<ExtForeignToplevelListV1, ()> for AppState {
    fn event(
        state: &mut AppState,
        _proxy: &ExtForeignToplevelListV1,
        _event: ext_foreign_toplevel_list_v1::Event,
        _udata: &(),
        _conn: &wayland_client::Connection,
        _qhandle: &wayland_client::QueueHandle<AppState>,
    ) {
        match _event {
            ext_foreign_toplevel_list_v1::Event::Toplevel { toplevel } => {
                println!("ext_foreign_toplevel_list_v1::Event::Toplevel: {:?}", toplevel);
            }
            ext_foreign_toplevel_list_v1::Event::Finished  => {
                println!("ext_foreign_toplevel_list_v1::Event::Finished");
            }
            _ => { }
        }
    }

    fn event_created_child(
        opcode: u16,
        _qhandle: &wayland_client::QueueHandle<Self>
    ) -> std::sync::Arc<dyn wayland_client::backend::ObjectData> {
        _qhandle.make_data::<ExtForeignToplevelHandleV1, _>(())
    }
}

impl Dispatch<wl_registry::WlRegistry, ()> for AppState {
    fn event(
        state: &mut AppState,
        _proxy: &wl_registry::WlRegistry,
        _event: wl_registry::Event,
        _udata: &(),
        _conn: &wayland_client::Connection,
        _qhandle: &wayland_client::QueueHandle<AppState>,
    ) {
        match _event {
            wl_registry::Event::Global { name, interface, version } if interface == "ext_foreign_toplevel_list_v1" => {
                _proxy.bind::<ExtForeignToplevelListV1, _, _>(name, version.min(1), _qhandle, ());
            }
            // wl_registry::Event::GlobalRemove { name } => {
            // }
            _ => { }

        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let connection                       = Connection::connect_to_env()?;
    let mut equeue: EventQueue<AppState> = connection.new_event_queue();
    let reg                              = connection.display().get_registry(&equeue.handle(), ());
    let mut state                        = AppState { };
    equeue.roundtrip(&mut state)?;
    equeue.roundtrip(&mut state)?;
    equeue.roundtrip(&mut state)?;
    Ok(())
}
