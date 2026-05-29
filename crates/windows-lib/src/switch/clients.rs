use crate::icon::set_icon;
use core_lib::{ClientData, ClientId, MonitorData};
use relm4::adw::gtk;
use relm4::adw::prelude::*;
use relm4::gtk::pango;
use relm4::prelude::*;

/// Clients component for FlowBox parent (non-workspace mode)
/// Shows clients in a flow layout without positioning
#[derive(Debug)]
pub struct Clients {
    active: bool,
    data: ClientData,
    pub id: ClientId,
    scale: f64,
    monitor_data: MonitorData,
}

#[derive(Debug)]
pub enum ClientsInput {
    SetActive(bool),
}

#[derive(Debug)]
pub struct ClientsInit {
    pub monitor_data: MonitorData,
    pub data: ClientData,
    pub id: ClientId,
    pub scale: f64,
}

#[derive(Debug)]
pub enum ClientsOutput {}

#[relm4::factory(pub)]
impl FactoryComponent for Clients {
    type Init = ClientsInit;
    type Input = ClientsInput;
    type Output = ClientsOutput;
    type CommandOutput = ();
    type ParentWidget = gtk::FlowBox;

    view! {
        gtk::FlowBoxChild {
            gtk::Button {
                #[watch]
                set_css_classes: if self.active { &["active", "client"] } else { &["client"] },
                set_cursor_from_name: Some("pointer"),
                set_width_request: scale(self.monitor_data.width, self.scale),
                set_height_request: scale(self.monitor_data.height, self.scale),
                gtk::Frame {
                    #[wrap(Some)]
                    set_label_widget = &gtk::Label {
                        set_overflow: gtk::Overflow::Visible,
                        set_margin_start: 6,
                        set_ellipsize: pango::EllipsizeMode::End,
                        set_label: if self.data.title.trim().is_empty() {
                            &self.data.class
                        } else {
                            &self.data.title
                        },
                    },
                    set_label_align: 0.5,
                    #[name(image)]
                    gtk::Image {
                        set_css_classes: if self.data.enabled { &["client-image"] } else { &["client-image", "monochrome"] },
                        set_pixel_size: calc_image_size(self.monitor_data.height, self.monitor_data.width, self.scale),
                    }
                }
            }
        }
    }

    fn init_model(init: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        // trace!("self {}x{}", init.data.width, init.data.height);
        Self {
            active: false,
            data: init.data,
            monitor_data: init.monitor_data,
            id: init.id,
            scale: init.scale,
        }
    }

    fn init_widgets(
        &mut self,
        _index: &DynamicIndex,
        root: Self::Root,
        _returned_widget: &<Self::ParentWidget as relm4::factory::FactoryView>::ReturnedWidget,
        _sender: FactorySender<Self>,
    ) -> Self::Widgets {
        let widgets = view_output!();

        // Set the icon for this client
        let client_h_w =
            scale(self.data.height, self.scale).min(scale(self.data.width, self.scale));
        if client_h_w > 70 {
            set_icon(&self.data.class, self.data.pid, &widgets.image);
        }

        widgets
    }

    fn update(&mut self, message: Self::Input, _sender: FactorySender<Self>) {
        match message {
            ClientsInput::SetActive(active) => {
                self.active = active;
            }
        }
    }
}

fn scale<T: Into<f64>>(value: T, scale: f64) -> i32 {
    (value.into() / (15f64 - scale)) as i32
}

fn calc_image_size(height: u16, width: u16, scale_val: f64) -> i32 {
    let client_h_w = scale(height, scale_val).min(scale(width, scale_val));
    if client_h_w > 70 {
        (f64::from(client_h_w.clamp(50, 600)) / 1.6) as i32 - 20
    } else {
        0 // Hide image for very small clients
    }
}
