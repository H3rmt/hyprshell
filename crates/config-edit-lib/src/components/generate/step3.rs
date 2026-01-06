use relm4::adw::prelude::*;
use relm4::gtk::{Align, Justification, SelectionMode};
use relm4::{ComponentParts, ComponentSender, RelmWidgetExt, SimpleComponent};
use relm4::{adw, gtk};
use tracing::trace;

#[derive(Debug, Copy, Clone)]
pub struct SearchEngines {
    pub google: bool,
    pub startpage: bool,
    pub duckduckgo: bool,
    pub bing: bool,
    pub wikipedia: bool,
    pub chatgpt: bool,
    pub youtube: bool,
    pub reddit: bool,
}

impl Default for SearchEngines {
    fn default() -> Self {
        Self {
            google: false,
            startpage: true,
            duckduckgo: false,
            bing: false,
            wikipedia: true,
            chatgpt: false,
            youtube: false,
            reddit: false,
        }
    }
}

#[derive(Debug)]
pub struct Step3 {
    pub google: adw::SwitchRow,
    pub startpage: adw::SwitchRow,
    pub duckduckgo: adw::SwitchRow,
    pub bing: adw::SwitchRow,
    pub wikipedia: adw::SwitchRow,
    pub chatgpt: adw::SwitchRow,
    pub youtube: adw::SwitchRow,
    pub reddit: adw::SwitchRow,
}

#[derive(Debug)]
pub enum Step3Input {
    // external set method
    SetData(SearchEngines),
    // internal set method
    _Update,
}

#[derive(Debug)]
pub struct Step3Init {}

#[allow(unused_assignments)]
#[relm4::component(pub)]
impl SimpleComponent for Step3 {
    type Init = Step3Init;
    type Input = Step3Input;
    type Output = SearchEngines;

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_hexpand: true,
            set_spacing: 40,
            gtk::Label::new(Some("Launcher Websearch Plugin")) {
                set_css_classes: &["title-1"],
                set_align: Align::Center,
                set_justify: Justification::Center,
            },
            gtk::ListBox {
                set_halign: Align::Center,
                set_valign: Align::Start,
                set_hexpand: true,
                set_selection_mode: SelectionMode::None,
                set_css_classes: &["items-list", "boxed-list", "generate-min-width"],
                #[local_ref]
                google -> adw::SwitchRow {
                    set_title: "Google",
                    set_subtitle: "https://www.google.com/search?q={}",
                    connect_active_notify => Step3Input::_Update,
                },
                #[local_ref]
                startpage -> adw::SwitchRow {
                    set_title: "Startpage",
                    set_subtitle: "https://www.startpage.com/sp/search?query={}",
                    connect_active_notify => Step3Input::_Update,
                },
                #[local_ref]
                duckduckgo -> adw::SwitchRow {
                    set_title: "DuckDuckGo",
                    set_subtitle: "https://duckduckgo.com/?q={}",
                    connect_active_notify => Step3Input::_Update,
                },
                #[local_ref]
                bing -> adw::SwitchRow {
                    set_title: "Bing",
                    set_subtitle: "https://www.bing.com/search?q={}",
                    connect_active_notify => Step3Input::_Update,
                },
                #[local_ref]
                wikipedia -> adw::SwitchRow {
                    set_title: "Wikipedia",
                    set_subtitle: "https://en.wikipedia.org/wiki/Special:Search?search={}",
                    connect_active_notify => Step3Input::_Update,
                },
                #[local_ref]
                chatgpt -> adw::SwitchRow {
                    set_title: "ChatGpt",
                    set_subtitle: "https://chatgpt.com/?q={}",
                    connect_active_notify => Step3Input::_Update,
                },
                #[local_ref]
                youtube -> adw::SwitchRow {
                    set_title: "YouTube",
                    set_subtitle: "https://www.youtube.com/results?search_query={}",
                    connect_active_notify => Step3Input::_Update,
                },
                #[local_ref]
                reddit -> adw::SwitchRow {
                    set_title: "Reddit",
                    set_subtitle: "https://www.reddit.com/search?q={}",
                    connect_active_notify => Step3Input::_Update,
                },
            },
            gtk::Label::new(Some("Search engines used for the websearch plugin in the launcher\n{} are replaced with the search string")) {
                set_css_classes: &["title-4"],
                set_justify: Justification::Center,
                set_vexpand: true,
                set_valign: Align::End,
            },
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let google = adw::SwitchRow::default();
        let startpage = adw::SwitchRow::default();
        let duckduckgo = adw::SwitchRow::default();
        let bing = adw::SwitchRow::default();
        let wikipedia = adw::SwitchRow::default();
        let chatgpt = adw::SwitchRow::default();
        let youtube = adw::SwitchRow::default();
        let reddit = adw::SwitchRow::default();

        let model = Self {
            google: google.clone(),
            startpage: startpage.clone(),
            duckduckgo: duckduckgo.clone(),
            bing: bing.clone(),
            wikipedia: wikipedia.clone(),
            chatgpt: chatgpt.clone(),
            youtube: youtube.clone(),
            reddit: reddit.clone(),
        };

        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        trace!("launcher::step3:update: {message:?}");
        match message {
            Step3Input::_Update => {
                let data = self.get_data();
                sender.output_sender().emit(data);
            }
            Step3Input::SetData(data) => {
                self.google.set_active(data.google);
                self.startpage.set_active(data.startpage);
                self.duckduckgo.set_active(data.duckduckgo);
                self.bing.set_active(data.bing);
                self.wikipedia.set_active(data.wikipedia);
                self.chatgpt.set_active(data.chatgpt);
                self.youtube.set_active(data.youtube);
                self.reddit.set_active(data.reddit);
            }
        }
    }
}

impl Step3 {
    fn get_data(&self) -> SearchEngines {
        SearchEngines {
            google: self.google.is_active(),
            startpage: self.startpage.is_active(),
            duckduckgo: self.duckduckgo.is_active(),
            bing: self.bing.is_active(),
            wikipedia: self.wikipedia.is_active(),
            chatgpt: self.chatgpt.is_active(),
            youtube: self.youtube.is_active(),
            reddit: self.reddit.is_active(),
        }
    }
}
