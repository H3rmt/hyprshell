use crate::flags_csv;
use crate::util::{handle_key_combo, key_combo_label, key_to_mod, SetCursor, SetTextIfDifferent};
use config_lib::KeyCombo;
use relm4::adw::gtk::Align;
use relm4::adw::prelude::*;
use relm4::factory::FactoryView;
use relm4::gtk::{EventControllerKey, SelectionMode};
use relm4::prelude::*;
use relm4::{adw, gtk, ComponentController, Controller};
use relm4_components::alert::{Alert, AlertMsg, AlertResponse, AlertSettings};

#[derive(Debug, Clone, Copy)]
pub(crate) enum BindKind {
    Forward,
    Reverse,
}

#[derive(Debug)]
struct SwitchBind {
    combo: KeyCombo,
    index: DynamicIndex,
}

#[derive(Debug)]
enum SwitchBindInput {}

#[derive(Debug)]
pub(crate) enum SwitchBindOutput {
    Delete(DynamicIndex),
}

#[relm4::factory]
impl FactoryComponent for SwitchBind {
    type Init = KeyCombo;
    type Input = SwitchBindInput;
    type Output = SwitchBindOutput;
    type CommandOutput = ();
    type ParentWidget = gtk::ListBox;

    view! {
        adw::ActionRow {
            set_title: &key_combo_label(&self.combo),
            add_suffix = &gtk::Button::from_icon_name("delete-symbolic") {
                connect_clicked[sender, idx = self.index.clone()] => move |_| {
                    sender.output_sender().emit(SwitchBindOutput::Delete(idx.clone()));
                },
            }
        }
    }

    fn init_model(init: Self::Init, index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        Self {
            combo: init,
            index: index.clone(),
        }
    }

    fn update(&mut self, msg: Self::Input, _sender: FactorySender<Self>) {
        match msg {}
    }
}

impl SwitchBind {
    fn set_combo(&mut self, combo: KeyCombo) {
        self.combo = combo;
    }
}

#[derive(Debug)]
pub(crate) struct Switch {
    config: crate::Switch,
    index: DynamicIndex,
    expanded: bool,
    forward_binds: FactoryVecDeque<SwitchBind>,
    reverse_binds: FactoryVecDeque<SwitchBind>,
    add_forward: adw::ButtonRow,
    add_reverse: adw::ButtonRow,
    bind_dialog: Controller<Alert>,
    bind_entry: gtk::Label,
    pending_combo: Option<KeyCombo>,
    pending_kind: Option<BindKind>,
    active_mods: std::rc::Rc<std::cell::RefCell<Vec<config_lib::KeyMod>>>,
}

#[derive(Debug)]
pub(crate) enum SwitchInput {
    SetEnabled(bool),
    SetExpanded(bool),
    ForwardBind(SwitchBindOutput),
    ReverseBind(SwitchBindOutput),
    AddBind(BindKind),
    BindDialogConfirm,
    BindDialogCancel,
    SetPendingCombo(KeyCombo),
    SetSameClass(bool),
    SetCurrentWorkspace(bool),
    SetCurrentMonitor(bool),
    SetSwitchWorkspaces(bool),
    SetExcludeWorkspaces(String),
    SetKillKey(char),
    Delete,
}

#[derive(Debug)]
pub(crate) enum SwitchOutput {
    Update(DynamicIndex, crate::Switch),
    Delete(DynamicIndex),
}

#[allow(unused_assignments)]
#[relm4::factory(pub(crate))]
impl FactoryComponent for Switch {
    type Init = crate::Switch;
    type Input = SwitchInput;
    type Output = SwitchOutput;
    type CommandOutput = ();
    type ParentWidget = gtk::ListBox;

    view! {
        #[root]
        adw::ExpanderRow {
            set_title_selectable: true,
            set_show_enable_switch: true,
            set_hexpand: true,
            set_css_classes: &["enable-frame"],
            set_title: "Switch",
            #[watch]
            #[block_signal(h)]
            set_enable_expansion: self.config.enabled,
            connect_enable_expansion_notify[sender] => move |e| {
                sender.input(SwitchInput::SetEnabled(e.enables_expansion()));
            } @h,
            #[watch]
            #[block_signal(h_exp)]
            set_expanded: self.expanded && self.config.enabled,
            connect_expanded_notify[sender] => move |e| {
                sender.input(SwitchInput::SetExpanded(e.is_expanded()));
            } @h_exp,
            add_row = &gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 10,
                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 10,
                    gtk::Label {
                        set_label: "Forward keybindings",
                    },
                    gtk::Image::from_icon_name("dialog-information-symbolic") {
                        set_cursor_by_name: "help",
                        set_tooltip_text: Some("Keybindings that open the switch window")
                    },
                },
                #[local_ref]
                forward_binds -> gtk::ListBox {
                    set_halign: gtk::Align::Fill,
                    set_valign: gtk::Align::Start,
                    set_expand: true,
                    set_selection_mode: SelectionMode::None,
                    set_css_classes: &["items-list", "boxed-list"],
                    #[local_ref]
                    add_forward -> adw::ButtonRow {
                        set_title: "Add forward keybinding",
                        connect_activated[sender] => move |_b| {
                            sender.input(SwitchInput::AddBind(BindKind::Forward));
                        }
                    },
                },
            },
            add_row = &gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 10,
                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 10,
                    gtk::Label {
                        set_label: "Reverse keybindings",
                    },
                    gtk::Image::from_icon_name("dialog-information-symbolic") {
                        set_cursor_by_name: "help",
                        set_tooltip_text: Some("Keybindings that move the switch selection in reverse")
                    },
                },
                #[local_ref]
                reverse_binds -> gtk::ListBox {
                    set_halign: gtk::Align::Fill,
                    set_valign: gtk::Align::Start,
                    set_expand: true,
                    set_selection_mode: SelectionMode::None,
                    set_css_classes: &["items-list", "boxed-list"],
                    #[local_ref]
                    add_reverse -> adw::ButtonRow {
                        set_title: "Add reverse keybinding",
                        connect_activated[sender] => move |_b| {
                            sender.input(SwitchInput::AddBind(BindKind::Reverse));
                        }
                    },
                },
            },
            add_row = &gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_css_classes: &["frame-row"],
                set_spacing: 30,
                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 10,
                    gtk::Label {
                        set_label: "Filter",
                    },
                    gtk::Image::from_icon_name("dialog-information-symbolic") {
                        set_cursor_by_name: "help",
                        set_tooltip_text: Some("Filter the shown windows by the provided filters")
                    },
                    adw::ExpanderRow {
                        #[watch]
                        set_title: &flags_csv!(self.config,same_class,current_monitor,current_workspace),
                        set_hexpand: true,
                        set_title_lines: 2,
                        set_css_classes: &["item-expander", "switch-item-expander"],
                        add_row = &adw::SwitchRow {
                            #[watch]
                            #[block_signal(h_2)]
                            set_active: self.config.same_class,
                            connect_active_notify[sender] => move |c| {
                                sender.input(SwitchInput::SetSameClass(c.is_active()));
                            } @h_2,
                            set_title: "Same class",
                        },
                        add_row = &adw::SwitchRow {
                            #[watch]
                            #[block_signal(h_3)]
                            set_active: self.config.current_workspace,
                            connect_active_notify[sender] => move |c| {
                                sender.input(SwitchInput::SetCurrentWorkspace(c.is_active()));
                            } @h_3,
                            set_title: "Current workspace",
                        },
                        add_row = &adw::SwitchRow {
                            #[watch]
                            #[block_signal(h_4)]
                            set_active: self.config.current_monitor,
                            connect_active_notify[sender] => move |c| {
                                sender.input(SwitchInput::SetCurrentMonitor(c.is_active()));
                            } @h_4,
                            set_title: "Current monitor",
                        }
                    }
                },
                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 10,
                    gtk::Label {
                        set_label: "Switch Workspaces",
                    },
                    gtk::Image::from_icon_name("dialog-information-symbolic") {
                        set_cursor_by_name: "help",
                        set_tooltip_text: Some("Switch between workspaces in the Switch mode instead of windows")
                    },
                    gtk::Switch {
                        #[watch]
                        #[block_signal(h_5)]
                        set_active: self.config.switch_workspaces,
                        connect_active_notify[sender] => move |e| {
                            sender.input(SwitchInput::SetSwitchWorkspaces(e.is_active()));
                        } @h_5,
                        set_valign: Align::Center,
                    },
                },
                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 10,
                    gtk::Label {
                        set_label: "Exclude workspaces",
                    },
                    gtk::Image::from_icon_name("dialog-information-symbolic") {
                        set_cursor_by_name: "help",
                        set_tooltip_text: Some("Exclude workspaces by regex \n(hyprctl workspaces -j | jq \".[].name\")")
                    },
                    gtk::Entry {
                        set_input_purpose: gtk::InputPurpose::FreeForm,
                        set_placeholder_text: Some("special:(monitor|second)"),
                        set_hexpand: true,
                        set_valign: Align::Center,
                        #[watch]
                        #[block_signal(h_6)]
                        set_text_if_different: &self.config.exclude_workspaces,
                        connect_changed[sender] => move |e| {
                            sender.input(SwitchInput::SetExcludeWorkspaces(e.text().into()));
                        } @h_6,
                    }
                },
                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 10,
                    gtk::Label {
                        set_label: "Key to kill window",
                    },
                    gtk::Image::from_icon_name("dialog-information-symbolic") {
                        set_cursor_by_name: "help",
                        set_tooltip_text: Some("Press this key to kill the focused window")
                    },
                    gtk::Entry {
                        set_input_purpose: gtk::InputPurpose::FreeForm,
                        set_placeholder_text: Some("q"),
                        set_hexpand: true,
                        set_valign: Align::Center,
                        #[watch]
                        #[block_signal(h_7)]
                        set_text_if_different: &self.config.kill_key.to_string(),
                        connect_changed[sender] => move |e| {
                            let key = e.text().to_string().chars().next().unwrap_or('q');
                            sender.input(SwitchInput::SetKillKey(key));
                        } @h_7,
                    }
                },
            },
            add_row = &adw::ButtonRow {
                set_title: "Remove switch",
                connect_activated[sender] => move |_| {
                    sender.input(SwitchInput::Delete);
                }
            },
        }
    }

    fn init_model(init: Self::Init, index: &DynamicIndex, sender: FactorySender<Self>) -> Self {
        let mut forward_binds = FactoryVecDeque::builder()
            .launch(gtk::ListBox::builder().selection_mode(SelectionMode::None).build())
            .forward(sender.input_sender(), SwitchInput::ForwardBind);
        {
            let mut list = forward_binds.guard();
            for combo in &init.forward_binds {
                list.push_back(combo.clone());
            }
        }
        let mut reverse_binds = FactoryVecDeque::builder()
            .launch(gtk::ListBox::builder().selection_mode(SelectionMode::None).build())
            .forward(sender.input_sender(), SwitchInput::ReverseBind);
        {
            let mut list = reverse_binds.guard();
            for combo in &init.reverse_binds {
                list.push_back(combo.clone());
            }
        }

        let bind_entry = gtk::Label::new(None);
        let active_mods = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
        let bind_dialog = Alert::builder()
            .launch(AlertSettings {
                text: Some("Press keybinding".to_string()),
                secondary_text: None,
                confirm_label: Some("Add".to_string()),
                cancel_label: Some("Cancel".to_string()),
                option_label: None,
                is_modal: true,
                destructive_accept: false,
                extra_child: Some(bind_entry.clone().into()),
            })
            .forward(sender.input_sender(), |res| match res {
                AlertResponse::Confirm => SwitchInput::BindDialogConfirm,
                AlertResponse::Cancel | AlertResponse::Option => SwitchInput::BindDialogCancel,
            });

        let key_controller = EventControllerKey::new();
        let entry = bind_entry.clone();
        let send = sender.clone();
        let active_mods_press = active_mods.clone();
        key_controller.connect_key_pressed(move |_, val, _, state| {
            let mods_snapshot = {
                let mut mods = active_mods_press.borrow_mut();
                if let Some(mod_key) = key_to_mod(val) {
                    if !mods.contains(&mod_key) {
                        mods.push(mod_key);
                    }
                }
                mods.clone()
            };
            match handle_key_combo(val, state, &mods_snapshot) {
                Some(combo) => {
                    entry.set_text(&key_combo_label(&combo));
                    send.input(SwitchInput::SetPendingCombo(combo));
                }
                None => {
                    entry.set_text("---");
                }
            }
            gtk::glib::Propagation::Stop
        });
        let active_mods_release = active_mods.clone();
        key_controller.connect_key_released(move |_, val, _, _| {
            if let Some(mod_key) = key_to_mod(val) {
                let mut mods = active_mods_release.borrow_mut();
                mods.retain(|m| *m != mod_key);
            }
        });
        bind_dialog.widgets().gtk_window_12.add_controller(key_controller);

        let expanded = init.enabled;
        Self {
            config: init,
            index: index.clone(),
            expanded,
            forward_binds,
            reverse_binds,
            add_forward: adw::ButtonRow::default(),
            add_reverse: adw::ButtonRow::default(),
            bind_dialog,
            bind_entry,
            pending_combo: None,
            pending_kind: None,
            active_mods,
        }
    }

    fn init_widgets(
        &mut self,
        _index: &DynamicIndex,
        root: Self::Root,
        _returned_widget: &<Self::ParentWidget as FactoryView>::ReturnedWidget,
        sender: FactorySender<Self>,
    ) -> Self::Widgets {
        let forward_binds = self.forward_binds.widget();
        let reverse_binds = self.reverse_binds.widget();
        let add_forward = &self.add_forward;
        let add_reverse = &self.add_reverse;
        let widgets = view_output!();
        widgets
    }

    fn update(&mut self, msg: Self::Input, sender: FactorySender<Self>) {
        match msg {
            SwitchInput::SetEnabled(enabled) => {
                self.config.enabled = enabled;
                self.expanded = enabled;
            }
            SwitchInput::SetExpanded(expanded) => {
                self.expanded = expanded;
                return;
            }
            SwitchInput::ForwardBind(msg) => match msg {
                SwitchBindOutput::Delete(index) => {
                    let idx = index.current_index();
                    if idx < self.config.forward_binds.len() {
                        self.config.forward_binds.remove(idx);
                    }
                    if idx < self.forward_binds.len() {
                        self.forward_binds.guard().remove(idx);
                    }
                    self.emit_update(sender);
                    return;
                }
            },
            SwitchInput::ReverseBind(msg) => match msg {
                SwitchBindOutput::Delete(index) => {
                    let idx = index.current_index();
                    if idx < self.config.reverse_binds.len() {
                        self.config.reverse_binds.remove(idx);
                    }
                    if idx < self.reverse_binds.len() {
                        self.reverse_binds.guard().remove(idx);
                    }
                    self.emit_update(sender);
                    return;
                }
            },
            SwitchInput::AddBind(kind) => {
                self.active_mods.borrow_mut().clear();
                self.pending_kind = Some(kind);
                self.pending_combo = None;
                self.bind_entry.set_text("");
                let target = match kind {
                    BindKind::Forward => &self.add_forward,
                    BindKind::Reverse => &self.add_reverse,
                };
                self.bind_dialog
                    .widget()
                    .set_transient_for(target.toplevel_window().as_ref());
                self.bind_dialog.emit(AlertMsg::Show);
                self.bind_dialog.widgets().gtk_window_12.set_modal(true); // TODO remove if https://github.com/Relm4/Relm4/issues/837 fixed
                return;
            }
            SwitchInput::BindDialogConfirm => {
                if let (Some(kind), Some(combo)) =
                    (self.pending_kind.take(), self.pending_combo.take())
                {
                    match kind {
                        BindKind::Forward => {
                            self.config.forward_binds.push(combo.clone());
                            self.forward_binds.guard().push_back(combo);
                        }
                        BindKind::Reverse => {
                            self.config.reverse_binds.push(combo.clone());
                            self.reverse_binds.guard().push_back(combo);
                        }
                    }
                    self.emit_update(sender);
                }
                self.active_mods.borrow_mut().clear();
                return;
            }
            SwitchInput::BindDialogCancel => {
                self.pending_kind = None;
                self.pending_combo = None;
                self.bind_entry.set_text("");
                self.active_mods.borrow_mut().clear();
                return;
            }
            SwitchInput::SetPendingCombo(combo) => {
                self.pending_combo = Some(combo);
                return;
            }
            SwitchInput::SetSameClass(enabled) => {
                self.config.same_class = enabled;
            }
            SwitchInput::SetCurrentWorkspace(enabled) => {
                self.config.current_workspace = enabled;
                if enabled {
                    self.config.current_monitor = false;
                }
            }
            SwitchInput::SetCurrentMonitor(enabled) => {
                self.config.current_monitor = enabled;
                if enabled {
                    self.config.current_workspace = false;
                }
            }
            SwitchInput::SetSwitchWorkspaces(enabled) => {
                self.config.switch_workspaces = enabled;
            }
            SwitchInput::SetExcludeWorkspaces(value) => {
                self.config.exclude_workspaces = value;
            }
            SwitchInput::SetKillKey(key) => {
                self.config.kill_key = key;
            }
            SwitchInput::Delete => {
                sender
                    .output_sender()
                    .emit(SwitchOutput::Delete(self.index.clone()));
                return;
            }
        }
        self.emit_update(sender);
    }
}

impl Switch {
    pub(crate) fn update_config(&mut self, config: crate::Switch) {
        self.config = config;
        if !self.config.enabled {
            self.expanded = false;
        }
        Self::sync_bind_list(&mut self.forward_binds, &self.config.forward_binds);
        Self::sync_bind_list(&mut self.reverse_binds, &self.config.reverse_binds);
    }

    fn sync_bind_list(list: &mut FactoryVecDeque<SwitchBind>, combos: &[KeyCombo]) {
        let mut guard = list.guard();
        let current_len = guard.len();
        let new_len = combos.len();
        let shared = current_len.min(new_len);
        for idx in 0..shared {
            if let Some(item) = guard.get_mut(idx) {
                item.set_combo(combos[idx].clone());
            }
        }
        if new_len > current_len {
            for combo in combos.iter().skip(current_len) {
                guard.push_back(combo.clone());
            }
        } else if new_len < current_len {
            for _ in new_len..current_len {
                guard.pop_back();
            }
        }
    }

    fn emit_update(&self, sender: FactorySender<Self>) {
        sender
            .output_sender()
            .emit(SwitchOutput::Update(self.index.clone(), self.config.clone()));
    }
}
