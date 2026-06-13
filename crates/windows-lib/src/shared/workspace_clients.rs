use crate::icon::set_icon;
use core_lib::{ClientData, ClientId};
use relm4::adw::gtk;
use relm4::adw::prelude::*;
use relm4::factory::Position;
use relm4::gtk::gdk;
use relm4::gtk::pango;
use relm4::prelude::*;

/// Workspace clients component for Fixed parent (workspace mode)
/// Shows clients positioned according to their window coordinates
#[derive(Debug)]
pub struct WorkspaceClients {
    active: bool,
    pub data: ClientData,
    pub id: ClientId,
    pub scale: f64,
    pub paintable: Option<gdk::Paintable>,
    live_thumbnails: bool,
}

#[derive(Debug)]
pub enum WorkspaceClientsInput {
    SetActive(bool),
    UpdateThumbnail(gdk::Texture),
}

#[derive(Debug)]
pub struct WorkspaceClientsInit {
    pub data: ClientData,
    pub id: ClientId,
    pub scale: f64,
    pub live_thumbnails: bool,
}

#[derive(Debug)]
pub enum WorkspaceClientsOutput {
    Clicked(ClientId),
}

#[relm4::factory(pub)]
impl FactoryComponent for WorkspaceClients {
    type Init = WorkspaceClientsInit;
    type Input = WorkspaceClientsInput;
    type Output = WorkspaceClientsOutput;
    type CommandOutput = ();
    type ParentWidget = gtk::Fixed;

    view! {
        #[root]
        gtk::Button {
            #[watch]
            set_css_classes: if self.active { &["active", "client"] } else { &["client"] },
            set_cursor_from_name: Some("pointer"),
            set_width_request: scale(self.data.width, self.scale),
            set_height_request: scale(self.data.height, self.scale),
            connect_clicked[sender, id = self.id] => move |_| sender.output_sender().emit(WorkspaceClientsOutput::Clicked(id)),
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
                if self.live_thumbnails {
                    #[name(picture)]
                    gtk::Picture {
                        set_content_fit: gtk::ContentFit::Contain,
                        #[watch]
                        set_paintable: self.paintable.as_ref(),
                        set_css_classes: if self.data.enabled { &["client-picture"] } else { &["client-picture", "monochrome"] },
                    }
                } else {
                    #[name(image)]
                    gtk::Image {
                        set_css_classes: if self.data.enabled { &["client-image"] } else { &["client-image", "monochrome"] },
                        set_pixel_size: calc_image_size(self.data.height, self.data.width, self.scale),
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
            id: init.id,
            scale: init.scale,
            paintable: None,
            live_thumbnails: init.live_thumbnails,
        }
    }

    fn init_widgets(
        &mut self,
        _index: &DynamicIndex,
        root: Self::Root,
        _returned_widget: &<Self::ParentWidget as relm4::factory::FactoryView>::ReturnedWidget,
        sender: FactorySender<Self>,
    ) -> Self::Widgets {
        let widgets = view_output!();

        if !self.live_thumbnails {
            // Set the icon for this client (only if large enough to show)
            let client_h_w =
                scale(self.data.height, self.scale).min(scale(self.data.width, self.scale));
            if client_h_w > 70 {
                set_icon(&self.data.class, self.data.pid, &widgets.image);
            }
        }

        widgets
    }

    fn update(&mut self, message: Self::Input, _sender: FactorySender<Self>) {
        match message {
            WorkspaceClientsInput::SetActive(active) => {
                self.active = active;
            }
            WorkspaceClientsInput::UpdateThumbnail(texture) => {
                self.paintable = Some(texture.upcast());
            }
        }
    }
}

impl Position<(f64, f64), DynamicIndex> for WorkspaceClients {
    fn position(&self, _index: &DynamicIndex) -> (f64, f64) {
        (
            f64::from(scale(self.data.x, self.scale)),
            f64::from(scale(self.data.y, self.scale)),
        )
    }
}

fn scale<T: Into<f64>>(value: T, scale: f64) -> i32 {
    (value.into() / (15f64 - scale)) as i32
}

fn calc_image_size(height: i16, width: i16, scale_val: f64) -> i32 {
    let client_h_w = scale(height, scale_val).min(scale(width, scale_val));
    if client_h_w > 70 {
        (f64::from(client_h_w.clamp(50, 600)) / 1.6) as i32 - 20
    } else {
        0 // Hide image for very small clients
    }
}

/*
impl FactoryView for gtk::Fixed {
    type Children = gtk::Widget;
    type ReturnedWidget = gtk::Widget;
    type Position = (f64, f64);

    fn factory_remove(&self, widget: &Self::ReturnedWidget) {
        use gtk::prelude::FixedExt;
        self.remove(widget);
    }

    fn factory_append(
        &self,
        widget: impl AsRef<Self::Children>,
        position: &Self::Position,
    ) -> Self::ReturnedWidget {
        use gtk::prelude::FixedExt;
        self.put(
            widget.as_ref(),
            position.0,
            position.1
        );
        widget.as_ref().clone()
    }

    fn factory_prepend(
        &self,
        widget: impl AsRef<Self::Children>,
        position: &Self::Position,
    ) -> Self::ReturnedWidget {
        self.factory_append(widget, position)
    }

    fn factory_insert_after(
        &self,
        widget: impl AsRef<Self::Children>,
        position: &Self::Position,
        _other: &Self::ReturnedWidget,
    ) -> Self::ReturnedWidget {
        self.factory_append(widget, position)
    }

    fn factory_move_after(&self, _widget: &Self::ReturnedWidget, _other: &Self::ReturnedWidget) {}

    fn factory_move_start(&self, _widget: &Self::ReturnedWidget) {}

    fn returned_widget_to_child(returned_widget: &Self::ReturnedWidget) -> Self::Children {
        returned_widget.clone()
    }

    fn factory_update_position(&self, widget: &Self::ReturnedWidget, position: &Self::Position) {
        self.factory_remove(widget);
        self.factory_append(widget, position);
    }
}
 */
