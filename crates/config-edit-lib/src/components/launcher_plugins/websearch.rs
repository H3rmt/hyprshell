use crate::util::SetCursor;
use config_lib::SearchEngine;
use relm4::adw::prelude::*;
use relm4::gtk::{Align, SelectionMode};
use relm4::prelude::*;
use relm4::{ComponentParts, ComponentSender, SimpleComponent};
use relm4::{adw, gtk};
use relm4_components::alert::{Alert, AlertMsg, AlertResponse, AlertSettings};
use tracing::trace;

#[derive(Debug)]
pub struct WebSearch {
    config: crate::WebSearchConfig,
    prev_config: crate::WebSearchConfig,
    engines: FactoryVecDeque<Search>,

    create: adw::ButtonRow,
    create_dialog: Controller<Alert>,
    name: adw::EntryRow,
    url: adw::EntryRow,
    key: adw::EntryRow,
}

#[derive(Debug)]
pub enum WebSearchInput {
    Set(crate::WebSearchConfig),
    SetPrev(crate::WebSearchConfig),
    Reset,
    Engine(SearchOutput),
    OpenCreateNew,
    Create,
    Ignore,
}

#[derive(Debug)]
pub struct WebSearchInit {
    pub config: crate::WebSearchConfig,
}

#[derive(Debug)]
pub enum WebSearchOutput {
    Enabled(bool),
    Engines(Vec<SearchEngine>),
}

#[allow(unused_assignments)]
#[relm4::component(pub)]
impl SimpleComponent for WebSearch {
    type Init = WebSearchInit;
    type Input = WebSearchInput;
    type Output = WebSearchOutput;

    view! {
        #[root]
        adw::ExpanderRow {
            set_title_selectable: true,
            set_show_enable_switch: true,
            set_hexpand: true,
            set_css_classes: &["enable-frame"],
            add_prefix = &gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_halign: Align::Fill,
                set_valign: Align::Center,
                set_spacing: 15,
                gtk::Label {
                    set_label: "Web search",
                },
                gtk::Image::from_icon_name("dialog-information-symbolic") {
                    set_cursor_by_name: "help",
                    set_tooltip_text: Some("Allows searching for the typed query in a web browser.")
                },
            },
            #[watch]
            #[block_signal(h)]
            set_enable_expansion: model.config.enabled,
            connect_enable_expansion_notify[sender] => move |e| {sender.output_sender().emit(WebSearchOutput::Enabled(e.enables_expansion()));} @h,
            #[watch]
            set_expanded: model.config.enabled,

            #[local_ref]
            add_row = engines -> gtk::ListBox {
                set_halign: Align::Fill,
                set_valign: Align::Start,
                set_expand: true,
                set_selection_mode: SelectionMode::None,
                set_css_classes: &["items-list", "boxed-list"],
                #[local_ref]
                create -> adw::ButtonRow {
                    set_title: "Create new",
                    connect_activated[sender] => move |_b| {
                        sender.input(WebSearchInput::OpenCreateNew);
                    }
                },
            }
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let list = gtk::ListBox::builder()
            .selection_mode(SelectionMode::None)
            .vexpand(true)
            .hexpand(true)
            .css_classes(vec!["boxed-list"])
            .build();

        let name = adw::EntryRow::builder().title("Name").build();
        list.append(&name);
        let url = adw::EntryRow::builder().title("Url containing {}").build();
        list.append(&url);
        let key = adw::EntryRow::builder()
            .title("Key (single character)")
            .build();
        list.append(&key);

        let create_dialog = Alert::builder()
            .transient_for(&root)
            .launch(AlertSettings {
                text: Some("Create New Search Engine".to_string()),
                secondary_text: None,
                confirm_label: Some(String::from("Create")),
                cancel_label: Some(String::from("Cancel")),
                option_label: None,
                is_modal: true,
                destructive_accept: false,
                extra_child: Some(list.into()),
            })
            .forward(sender.input_sender(), |res| match res {
                AlertResponse::Confirm => WebSearchInput::Create,
                AlertResponse::Option | AlertResponse::Cancel => WebSearchInput::Ignore,
            });

        let mut engines = FactoryVecDeque::builder()
            .launch(gtk::ListBox::builder().build())
            .forward(sender.input_sender(), WebSearchInput::Engine);

        let mut l = engines.guard();
        for engine in &init.config.engines {
            l.push_back(engine.clone());
        }
        drop(l);

        let create = adw::ButtonRow::default();

        let model = Self {
            config: init.config.clone(),
            prev_config: init.config,
            engines,
            create,
            create_dialog,
            name,
            url,
            key,
        };

        let engines = model.engines.widget();
        let create = &model.create;
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        trace!("launcher_plugins::websearch::update: {message:?}");
        match message {
            WebSearchInput::Ignore => {}
            WebSearchInput::Set(config) => {
                self.config = config;
                let mut l = self.engines.guard();
                l.clear();
                for engine in &self.config.engines {
                    l.push_back(engine.clone());
                }
                drop(l);
            }
            WebSearchInput::SetPrev(config) => {
                self.prev_config = config;
            }
            WebSearchInput::Reset => {
                sender.input(WebSearchInput::Set(self.prev_config.clone()));
            }
            WebSearchInput::Engine(msg) => match msg {
                SearchOutput::Delete(ch) => {
                    let engines = self
                        .config
                        .engines
                        .iter()
                        .filter(|e| e.key != ch)
                        .cloned()
                        .collect();
                    sender
                        .output_sender()
                        .emit(WebSearchOutput::Engines(engines));
                }
            },
            WebSearchInput::OpenCreateNew => {
                self.create_dialog
                    .widget()
                    .set_transient_for(self.create.toplevel_window().as_ref());
                self.create_dialog.emit(AlertMsg::Show);
                self.create_dialog.widgets().gtk_window_12.set_modal(true); // TODO remove if https://github.com/Relm4/Relm4/issues/837 fixed
            }
            WebSearchInput::Create => {
                let name = self.name.text().to_string().into_boxed_str();
                let url = self.url.text().to_string().into_boxed_str();
                let key = self.key.text().to_string().chars().next().unwrap_or('x');
                self.create_dialog.emit(AlertMsg::Hide);
                self.name.set_text("");
                self.url.set_text("");
                self.key.set_text("");

                let mut engines = self.config.engines.clone();
                engines.push(SearchEngine { name, url, key });
                sender
                    .output_sender()
                    .emit(WebSearchOutput::Engines(engines));
            }
        }
    }
}

/// ------------------------------------------------------------------------------------------------

#[derive(Debug)]
struct Search {
    engine: SearchEngine,
}

#[derive(Debug)]
enum SearchInput {}

#[derive(Debug)]
pub enum SearchOutput {
    Delete(char),
}

#[relm4::factory]
impl FactoryComponent for Search {
    type Init = SearchEngine;
    type Input = SearchInput;
    type Output = SearchOutput;
    type CommandOutput = ();
    type ParentWidget = gtk::ListBox;

    view! {
        adw::ActionRow {
            set_title: &format!("{} ({})", self.engine.name, self.engine.key),
            set_subtitle: &self.engine.url,
            add_suffix = &gtk::Button::from_icon_name("delete-symbolic") {
                connect_clicked[sender, key = self.engine.key] => move |_| sender.output_sender().emit(SearchOutput::Delete(key)),
            }
        }
    }

    fn init_model(init: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        Self { engine: init }
    }

    fn update(&mut self, msg: Self::Input, _sender: FactorySender<Self>) {
        match msg {};
    }
}
