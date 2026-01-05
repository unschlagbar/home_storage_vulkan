use std::env;
use std::{cell::RefCell, fs, path::PathBuf, rc::Rc};

use iron_oxide::ui::{Absolute, Align, ElementBuilder, FlexDirection, Image, ScrollPanel, Ticking};
use iron_oxide::{
    graphics::formats::RGBA,
    ui::{
        Button, ButtonState, CallContext, Container, QueuedEvent, Text, UiEvent, UiRect, Ui,
        UiUnit::*,
    },
};

use crate::UiIcons;

const OPEN: u16 = 1;
const ENTRY_ACTION: u16 = 2;
const GO_BACK: u16 = 3;

pub struct Explorer {
    pub content_window: u32,
    pub tool_tip: u32,
    pub hovered_element: u32,
    pub selected_file: u32,

    pub path: PathBuf,
    pub ui: Rc<RefCell<Ui>>,
}

impl Explorer {
    pub fn new(ui: Rc<RefCell<Ui>>) -> Self {
        let content_window = {
            let mut ui = ui.borrow_mut();

            let root = ui.add_child_to_root(
                Container {
                    color: RGBA::ZERO,
                    width: Fill,
                    height: Fill,
                    ..Default::default()
                }
                .wrap("root"),
            );

            let nav_bar = ui
                .add_child_to(
                    Container {
                        color: RGBA::grey(25),
                        width: Fill,
                        height: Px(40.0),
                        ..Default::default()
                    }
                    .wrap("nav_bar"),
                    root,
                )
                .unwrap();

            //Back Button
            let back_button = ui
                .add_child_to(
                    Button {
                        color: RGBA::ZERO,
                        width: Px(34.0),
                        height: Px(34.0),
                        margin: UiRect::new(3.0),
                        padding: UiRect::new(2.0),
                        callback: Some(on_click),
                        message: GO_BACK,
                        ..Default::default()
                    }
                    .wrap("back"),
                    nav_bar,
                )
                .unwrap();

            ui.add_child_to(
                Image {
                    atlas_index: UiIcons::Back as u32,
                    ..Default::default()
                }
                .wrap("back_image"),
                back_button,
            );

            let content = ui
                .add_child_to(
                    Container {
                        color: RGBA::grey(30),
                        width: Fill,
                        height: Fill,
                        border: [0, 1, 0, 0],
                        border_color: RGBA::grey(90),
                        padding: UiRect::new(2.0),
                        ..Default::default()
                    }
                    .wrap("content"),
                    root,
                )
                .unwrap();

            ui.add_child_to(
                ScrollPanel {
                    padding: UiRect::new(2.0),
                    ..Default::default()
                }
                .wrap("scroll_pannel"),
                content,
            )
            .unwrap()
        };

        let path = env::var(Self::HOME)
            .ok()
            .unwrap_or(Self::ROOT_PATH.to_string())
            .into();

        Self {
            content_window,
            tool_tip: u32::MAX,
            hovered_element: 0,
            selected_file: 0,
            path,
            ui,
        }
    }

    pub fn display_path(&mut self) {
        let mut ui = self.ui.borrow_mut();

        match fs::read_dir(&self.path) {
            Ok(entries) => {
                let content = ui.get_element(self.content_window).unwrap();
                // Todo fix unchecked ticks & selection
                ui.remove_all_element(&content);

                let mut is_empty = true;
                for entry in entries {
                    let entry = entry.unwrap();
                    let name = entry.file_name().into_string().unwrap();
                    let extention = name.split('.').next_back().unwrap_or_default();

                    if name.starts_with('.') || extention == "ini" {
                        continue;
                    }

                    is_empty = false;

                    let (element_name, icon) = if entry.path().is_dir() {
                        ("folder", UiIcons::Folder as u32)
                    } else {
                        let icon = match extention {
                            "txt" => UiIcons::TxtFile,
                            "rs" => UiIcons::RustFile,
                            _ => UiIcons::TxtFile,
                        } as u32;
                        ("file", icon)
                    };

                    ui.add_child_to(
                        Button {
                            color: RGBA::ZERO,
                            border_color: RGBA::GREEN,
                            height: Auto,
                            width: Fill,
                            flex_direction: FlexDirection::Horizontal,
                            padding: UiRect::horizontal(Px(2.0)),
                            corner: [Px(5.0); 4],
                            callback: Some(on_click),
                            message: OPEN,
                            ..Default::default()
                        }
                        .wrap_childs(
                            element_name,
                            vec![
                                Container {
                                    height: Px(30.0),
                                    width: Px(30.0),
                                    margin: UiRect::from(&[0.0, 0.0, 6.0, 0.0]),
                                    color: RGBA::TRANSPARENT,
                                    padding: UiRect::new(3.0),
                                    ..Default::default()
                                }
                                .wrap_childs(
                                    "",
                                    vec![
                                        Image {
                                            atlas_index: icon,
                                            ..Default::default()
                                        }
                                        .wrap(""),
                                    ],
                                ),
                                Text {
                                    color: RGBA::grey(220),
                                    text: name,
                                    align: Align::Left,
                                    ..Default::default()
                                }
                                .wrap(""),
                            ],
                        ),
                        self.content_window,
                    );
                }

                if is_empty {
                    ui.add_child_to(
                        Container {
                            color: RGBA::ZERO,
                            height: Px(50.0),
                            width: Relative(1.0),
                            padding: UiRect::horizontal(Px(2.0)),
                            ..Default::default()
                        }
                        .wrap_childs(
                            "",
                            vec![
                                Text {
                                    text: "This Folder\nis Empty".to_string(),
                                    color: RGBA::grey(130),
                                    align: Align::Center,
                                    ..Default::default()
                                }
                                .wrap("empty_msg"),
                            ],
                        ),
                        self.content_window,
                    );
                }
            }
            Err(error) => {
                if let Some(path) = self.path.parent() {
                    self.path = path.into();
                }

                let e_message = Ticking {
                    inner: Absolute {
                        color: RGBA::grey(50),
                        x: Px(ui.cursor_pos.x),
                        y: Px(ui.cursor_pos.y),
                        border: [1; 4],
                        width: Auto,
                        height: Auto,
                        padding: UiRect::new(3.0),
                        corner: [Px(4.0); 4],
                        ..Default::default()
                    },
                    tick: Some(tick_error),
                    ..Default::default()
                }
                .wrap_childs(
                    "e_msg",
                    vec![
                        Text {
                            text: error.to_string(),
                            color: RGBA::RED,
                            ..Default::default()
                        }
                        .wrap(""),
                    ],
                );

                ui.add_child_to_root(e_message);
            }
        }
    }

    pub fn proceed_event(&mut self, event: QueuedEvent) {
        if event.event == UiEvent::Press {
            match event.message {
                OPEN => {
                    if event.element_name == "folder" {
                        let element = {
                            let mut ui = self.ui.borrow_mut();
                            ui.get_element(event.element_id).unwrap()
                        };

                        let text = element.get_text_at_pos(1).unwrap();
                        self.path.push(text);

                        self.display_path();
                    } else {
                        self.open_file(event.element_id);
                    }
                }
                GO_BACK => {
                    if let Some(path) = self.path.parent() {
                        self.path = path.into();
                        self.display_path();
                    }
                }
                ENTRY_ACTION => match event.element_name {
                    "open" => {
                        let selected =  {
                            let mut ui = self.ui.borrow_mut();
                            ui.get_element(self.hovered_element).unwrap()
                        };

                        if selected.name == "folder" {
                            let text = selected.get_text_at_pos(1).unwrap();
                            self.path.push(text);
                            self.display_path();
                        } else {
                            let id = selected.id();
                            self.open_file(id);
                        }
                    }
                    "rename" => {
                        let mut ui = self.ui.borrow_mut();

                        let selected = ui.get_element_mut(self.selected_file).unwrap();
                        let container: &mut Button = selected.downcast_mut().unwrap();
                        container.border = [1; 4];
                        container.callback = None;

                        let child = selected.get_child(1).unwrap();
                        Text::focus(&mut ui, &child, 0..0);
                    }
                    name => println!("{name}"),
                },

                _ => unreachable!(),
            }
        }
    }

    pub fn right_click(&mut self, ui: &mut Ui) -> bool {
        if let Some(hovered) = ui.get_hovered() {
            if hovered.name == "file" || hovered.name == "folder" {
                self.hovered_element = hovered.id();
                self.selected_file = hovered.id();

                if self.tool_tip != u32::MAX {
                    ui.remove_element_by_id(self.tool_tip);
                }

                let x = Px(ui.cursor_pos.x);
                let y = Px(ui.cursor_pos.y);

                self.tool_tip = ui.add_child_to_root(
                    Absolute {
                        x,
                        y,
                        width: Px(200.0),
                        height: Auto,
                        padding: UiRect::new(2.0),
                        color: RGBA::grey(50),
                        corner: [Px(7.0); 4],
                        ..Default::default()
                    }
                    .wrap_childs(
                        "",
                        vec![
                            Button {
                                width: Fill,
                                height: Px(30.0),
                                color: RGBA::ZERO,
                                border_color: RGBA::BLUE,
                                corner: [Px(5.0); 4],
                                callback: Some(on_click_tooltip),
                                message: ENTRY_ACTION,
                                ..Default::default()
                            }
                            .wrap_childs(
                                "open",
                                vec![
                                    Text {
                                        text: "Öffnen".to_string(),
                                        color: RGBA::grey(220),
                                        ..Default::default()
                                    }
                                    .wrap(""),
                                ],
                            ),
                            Button {
                                width: Fill,
                                height: Px(30.0),
                                color: RGBA::ZERO,
                                border_color: RGBA::BLUE,
                                corner: [Px(5.0); 4],
                                callback: Some(on_click_tooltip),
                                message: ENTRY_ACTION,
                                ..Default::default()
                            }
                            .wrap_childs(
                                "rename",
                                vec![
                                    Text {
                                        text: "Umbennenen".to_string(),
                                        color: RGBA::grey(220),

                                        ..Default::default()
                                    }
                                    .wrap(""),
                                ],
                            ),
                            Button {
                                width: Relative(1.0),
                                height: Px(30.0),
                                color: RGBA::ZERO,
                                border_color: RGBA::BLUE,
                                corner: [Px(5.0); 4],
                                callback: Some(on_click_tooltip),
                                message: ENTRY_ACTION,
                                ..Default::default()
                            }
                            .wrap_childs(
                                "delete",
                                vec![
                                    Text {
                                        text: "Löschen".to_string(),
                                        color: RGBA::grey(220),
                                        ..Default::default()
                                    }
                                    .wrap(""),
                                ],
                            ),
                        ],
                    ),
                );
            }
            ui.layout_changed();
            true
        } else {
            false
        }
    }

    pub fn mouse_click(&mut self, ui: &mut Ui) -> bool {
        if self.tool_tip != u32::MAX {
            ui.remove_element_by_id(self.tool_tip);

            self.tool_tip = u32::MAX;
            ui.layout_changed();
            true
        } else {
            false
        }
    }
}

fn on_click(context: CallContext) {
    let button: &mut Button = context.element.get_mut(context.ui).downcast_mut().unwrap();

    match button.state {
        ButtonState::Normal => {
            button.color = RGBA::ZERO;
        }
        ButtonState::Hovered => {
            button.color = RGBA::grey(40);
        }
        ButtonState::Pressed => {
            button.color = RGBA::grey(60);
        }
        ButtonState::Disabled => unreachable!(),
    }
    context.ui.color_changed();
}

fn on_click_tooltip(context: CallContext) {
    let button: &mut Button = context.element.get_mut(context.ui).downcast_mut().unwrap();

    match button.state {
        ButtonState::Normal => {
            button.color = RGBA::ZERO;
            button.border = [0; 4];
        }
        ButtonState::Hovered => {
            button.color = RGBA::grey(40);
            button.border = [1; 4];
        }
        ButtonState::Pressed => {
            button.color = RGBA::grey(60);
            button.border = [1; 4];
        }
        ButtonState::Disabled => unreachable!(),
    }
    context.ui.color_changed();
}

fn tick_error(context: CallContext) {
    let this: &Ticking<Absolute> = context.element.downcast().unwrap();
    if this.last_tick.elapsed().as_secs_f32() > 1.0 {
        context.ui.remove_element(&context.element);
    }
}
