use core_lib::{ClientData, ClientId};
use relm4::SimpleComponent;
use relm4::adw::gtk;
use relm4::adw::prelude::*;
use relm4::factory::Position;
use relm4::gtk::pango;
use relm4::prelude::*;

#[derive(Debug)]
pub struct WorkspaceClients {
    active: bool,
    data: ClientData,
    id: ClientId,
    scale: f64,
}

#[derive(Debug)]
pub enum WorkspaceClientsInput {
    SetActive(bool),
}

#[derive(Debug)]
pub struct WorkspaceClientsInit {
    data: ClientData,
    id: ClientId,
    scale: f64,
}

#[derive(Debug)]
pub enum WorkspaceClientsOutput {}

#[relm4::factory(pub)]
impl FactoryComponent for WorkspaceClients {
    type Init = WorkspaceClientsInit;
    type Input = WorkspaceClientsInput;
    type Output = WorkspaceClientsOutput;
    type CommandOutput = ();
    type ParentWidget = gtk::Fixed;

    view! {
        gtk::Button {
            #[watch]
            set_css_classes: if self.active { &["active", "client", "no-hover"] } else { &["client", "no-hover"] },
            set_cursor_from_name: Some("pointer"),
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
                gtk::Image {
                    set_css_classes: &["client-image"],
                    set_pixel_size: (f64::from(scale(self.data.height, self.scale).min(scale(self.data.width, self.scale)).clamp(50, 600)) / 1.6) as i32 - 20,
                }
            }
        }
    }

    fn init_model(init: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        Self {
            active: false,
            data: init.data,
            id: init.id,
            scale: init.scale,
        }
    }

    fn update(&mut self, message: Self::Input, sender: FactorySender<Self>) {
        match message {
            WorkspaceClientsInput::SetActive(active) => {
                self.active = active;
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
