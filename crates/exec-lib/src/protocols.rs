//! Vendored Hyprland protocol bindings.
//!
//! The XML spec is vendored from `hyprwm/hyprland-protocols` at commit
//! `bd153e76f751f150a09328dbdeb5e4fab9d23622`, the exact revision embedded in
//! the former `wayland-protocols-hyprland` 1.2.0 dependency. It is licensed
//! under BSD-3-Clause.
//!
//! Bindings are generated at compile time by `wayland-scanner`.

pub mod toplevel_mapping {
    pub mod v1 {
        /// Client-side API of this protocol.
        pub mod client {
            #![allow(dead_code, non_camel_case_types, unused_unsafe, unused_variables)]
            #![allow(non_upper_case_globals, non_snake_case, unused_imports)]
            #![allow(missing_docs, clippy::all, clippy::pedantic, clippy::nursery)]

            use wayland_client;
            use wayland_client::protocol::*;
            use wayland_protocols::ext::foreign_toplevel_list::v1::client::*;
            use wayland_protocols_wlr::foreign_toplevel::v1::client::*;

            pub mod __interfaces {
                use wayland_client::protocol::__interfaces::*;
                use wayland_protocols::ext::foreign_toplevel_list::v1::client::__interfaces::*;
                use wayland_protocols_wlr::foreign_toplevel::v1::client::__interfaces::*;
                wayland_scanner::generate_interfaces!(
                    "./protocols/hyprland-toplevel-mapping-v1.xml"
                );
            }
            use self::__interfaces::*;

            wayland_scanner::generate_client_code!("./protocols/hyprland-toplevel-mapping-v1.xml");
        }
    }
}
