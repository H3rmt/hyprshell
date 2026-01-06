use crate::util::SetCursor;
use config_lib::actions::ToAction;
use config_lib::{ActionsPluginAction, ActionsPluginActionCustom};
use relm4::adw::prelude::*;
use relm4::gtk::{Align, SelectionMode};
use relm4::prelude::*;
use relm4::{ComponentParts, ComponentSender, SimpleComponent};
use relm4_components::alert::{Alert, AlertMsg, AlertResponse, AlertSettings};
use std::path::Path;

#[allow(clippy::struct_field_names)]
#[derive(Debug)]
pub struct Actions {
    config: crate::ActionsPluginConfig,
    prev_config: crate::ActionsPluginConfig,
    actions: FactoryVecDeque<Action>,

    create: adw::ButtonRow,
    create_dialog: Controller<Alert>,
    pub names: adw::EntryRow,
    pub details: adw::EntryRow,
    pub command: adw::EntryRow,
    pub icon: adw::EntryRow,
}

#[derive(Debug)]
pub enum ActionsInput {
    Set(crate::ActionsPluginConfig),
    SetPrev(crate::ActionsPluginConfig),
    Reset,
    Action(ActionOutput),
    OpenCreateNew,
    Create,
    Ignore,
}

#[derive(Debug)]
pub struct ActionsInit {
    pub config: crate::ActionsPluginConfig,
}

#[derive(Debug)]
pub enum ActionsOutput {
    Enabled(bool),
    Actions(Vec<ActionsPluginAction>),
}

#[allow(unused_assignments)]
#[relm4::component(pub)]
impl SimpleComponent for Actions {
    type Init = ActionsInit;
    type Input = ActionsInput;
    type Output = ActionsOutput;

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
                    set_label: "Actions",
                },
                gtk::Image::from_icon_name("dialog-information-symbolic") {
                    set_cursor_by_name: "help",
                    set_tooltip_text: Some("Runs the specified action like reboot, hibernate, etc. Custom actions can also be specified.")
                },
            },
            #[watch]
            #[block_signal(h)]
            set_enable_expansion: model.config.enabled,
            connect_enable_expansion_notify[sender] => move |e| {sender.output_sender().emit(ActionsOutput::Enabled(e.enables_expansion()));} @h,
            #[watch]
            set_expanded: model.config.enabled,

            #[local_ref]
            add_row = actions -> gtk::ListBox {
                set_halign: Align::Fill,
                set_valign: Align::Start,
                set_expand: true,
                set_selection_mode: SelectionMode::None,
                set_css_classes: &["items-list", "boxed-list"],

                #[local_ref]
                create -> adw::ButtonRow {
                    set_title: "Create new",
                    connect_activated[sender] => move |_b| {
                        sender.input(ActionsInput::OpenCreateNew);
                    }
                },
            }
        }
    }

    #[allow(unused_assignments)]
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

        let names = adw::EntryRow::builder()
            .title("Names (separate with ,)")
            .build();
        list.append(&names);
        let details = adw::EntryRow::builder().title("Description").build();
        list.append(&details);
        let command = adw::EntryRow::builder().title("Shell command").build();
        list.append(&command);
        let icon = adw::EntryRow::builder().title("Icon name").build();
        list.append(&icon);

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
                AlertResponse::Confirm => ActionsInput::Create,
                AlertResponse::Option | AlertResponse::Cancel => ActionsInput::Ignore,
            });

        let mut actions = FactoryVecDeque::builder()
            .launch(gtk::ListBox::builder().build())
            .forward(sender.input_sender(), ActionsInput::Action);

        let mut l = actions.guard();
        for engine in &init.config.actions {
            l.push_back(engine.clone());
        }
        drop(l);

        let create = adw::ButtonRow::default();

        let model = Self {
            config: init.config.clone(),
            prev_config: init.config,
            actions,
            create,
            create_dialog,
            names,
            details,
            command,
            icon,
        };

        let actions = model.actions.widget();
        let create = &model.create;
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match message {
            ActionsInput::Ignore => {}
            ActionsInput::Set(config) => {
                self.config = config;
                let mut l = self.actions.guard();
                l.clear();
                for engine in &self.config.actions {
                    l.push_back(engine.clone());
                }
                drop(l);
            }
            ActionsInput::SetPrev(config) => {
                self.prev_config = config;
            }
            ActionsInput::Reset => {
                sender.input(ActionsInput::Set(self.prev_config.clone()));
            }
            ActionsInput::Action(msg) => match msg {
                ActionOutput::Delete(command) => {
                    #[allow(clippy::iter_overeager_cloned)]
                    let actions = self
                        .config
                        .actions
                        .iter()
                        .cloned()
                        .filter(|e| e.clone().to_action().command != command)
                        .collect::<Vec<_>>();
                    sender.output_sender().emit(ActionsOutput::Actions(actions));
                }
            },
            ActionsInput::OpenCreateNew => {
                self.create_dialog
                    .widget()
                    .set_transient_for(self.create.toplevel_window().as_ref());
                self.create_dialog.emit(AlertMsg::Show);
                self.create_dialog.widgets().gtk_window_12.set_modal(true); // TODO remove if https://github.com/Relm4/Relm4/issues/837 fixed
            }
            ActionsInput::Create => {
                let names = self
                    .names
                    .text()
                    .split(',')
                    .map(|s| s.trim().to_string().into_boxed_str())
                    .collect::<Vec<_>>();
                let details = self.details.text().to_string().into_boxed_str();
                let command = self.details.text().to_string().into_boxed_str();
                let icon = Box::from(Path::new(&self.icon.text()));
                self.create_dialog.emit(AlertMsg::Hide);
                self.names.set_text("");
                self.details.set_text("");
                self.command.set_text("");
                self.icon.set_text("");

                let mut actions = self.config.actions.clone();
                actions.push(ActionsPluginAction::Custom(ActionsPluginActionCustom {
                    names,
                    details,
                    command,
                    icon,
                }));
                sender.output_sender().emit(ActionsOutput::Actions(actions));
            }
        }
    }
}

/// ------------------------------------------------------------------------------------------------

#[derive(Debug)]
struct Action {
    action: config_lib::ActionsPluginActionCustom,
}

#[derive(Debug)]
enum ActionInput {
    Update,
}

#[derive(Debug)]
pub enum ActionOutput {
    Delete(Box<str>),
}

#[allow(unused_assignments)]
#[relm4::factory]
impl FactoryComponent for Action {
    type Init = ActionsPluginAction;
    type Input = ActionInput;
    type Output = ActionOutput;
    type CommandOutput = ();
    type ParentWidget = gtk::ListBox;

    view! {
        adw::ActionRow {
            set_title: &self.action.names.join(", "),
            set_subtitle: &self.action.details,
            #[name = "image"]
            add_prefix = &gtk::Image {},
            add_suffix = &gtk::Button::from_icon_name("delete-symbolic") {
                connect_clicked[sender, command = self.action.command.clone()] => move |_| sender.output_sender().emit(ActionOutput::Delete(command.clone())),
            }
        }
    }

    fn init_model(init: Self::Init, _index: &DynamicIndex, sender: FactorySender<Self>) -> Self {
        sender.input(ActionInput::Update);
        Self {
            action: init.to_action(),
        }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        _message: Self::Input,
        _sender: FactorySender<Self>,
    ) {
        let icon_path = self.action.icon.clone();
        if icon_path.is_absolute() {
            widgets.image.set_from_file(Some(Path::new(&*icon_path)));
        } else {
            widgets
                .image
                .set_icon_name(icon_path.file_name().and_then(|name| name.to_str()));
        }
    }

    fn update(&mut self, msg: Self::Input, _sender: FactorySender<Self>) {
        match msg {
            ActionInput::Update => {}
        }
    }
}
