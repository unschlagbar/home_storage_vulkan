use std::env;
use std::ptr::null_mut;
use std::{cell::RefCell, fs, path::PathBuf, process::Command, rc::Rc};

use iron_oxide::ui::{Absolute, Align, ScrollPanel, Ticking, TypeConst};
use iron_oxide::{
    graphics::formats::RGBA,
    ui::{
        Button, ButtonState, CallContext, Container, DirtyFlags, FnPtr, OutArea, QueuedEvent, Text,
        UiEvent, UiState, UiUnit::*,
    },
};

const OPEN: u16 = 1;
const ENTRY_ACTION: u16 = 2;
const GO_BACK: u16 = 3;

pub struct Explorer {
    pub content_window: u32,
    pub tool_tip: u32,
    pub hovered_element: u32,

    pub path: PathBuf,
    pub ui: Rc<RefCell<UiState>>,
}

impl Explorer {
    pub fn new(ui: Rc<RefCell<UiState>>) -> Self {
        let content_window = {
            let mut ui = ui.borrow_mut();

            let root = ui.add_element(
                Container {
                    color: RGBA::ZERO,
                    width: Relative(1.0),
                    height: Relative(1.0),
                    ..Default::default()
                },
                "root",
            );

            let nav_bar = ui
                .add_child_to(
                    Container {
                        color: RGBA::rgb(20, 20, 20),
                        width: Relative(1.0),
                        height: Px(40.0),
                        ..Default::default()
                    },
                    "nav_bar",
                    root,
                )
                .unwrap();

            //Back Button
            ui.add_child_to(
                Button {
                    color: RGBA::rgb(20, 20, 20),
                    width: Px(40.0),
                    height: Px(40.0),
                    margin: OutArea::new(3.0),
                    callback: FnPtr::new(on_click),
                    message: GO_BACK,
                    childs: vec![],
                    ..Default::default()
                },
                "back",
                nav_bar,
            );

            let content = ui
                .add_child_to(
                    Container {
                        color: RGBA::rgb(30, 30, 30),
                        width: Fill,
                        height: Fill,
                        border: [1.0; 4],
                        border_color: RGBA::rgb(100, 100, 100),
                        padding: OutArea::new(2.0),
                        ..Default::default()
                    },
                    "content",
                    root,
                )
                .unwrap();

            ui.add_child_to(
                ScrollPanel {
                    padding: OutArea::new(2.0),
                    ..Default::default()
                },
                "scroll_pannel",
                content,
            )
            .unwrap()
        };
        Self {
            content_window,
            tool_tip: u32::MAX,
            hovered_element: 0,
            #[cfg(target_os = "windows")]
            path: env::var("USERPROFILE").ok().unwrap_or("C:/".into()).into(),
            #[cfg(not(target_os = "windows"))]
            path: env::var("HOME").ok().unwrap_or("".into()).into(),
            ui,
        }
    }

    pub fn display_path(&mut self) {
        let mut ui = self.ui.borrow_mut();

        match fs::read_dir(&self.path) {
            Ok(entries) => {
                let content = ui.get_element(self.content_window).unwrap();
                content.clear_childs();

                let mut is_empty = true;
                for entry in entries {
                    let entry = entry.unwrap();
                    let name = entry.file_name().into_string().unwrap();

                    if name.starts_with('.')
                        || entry
                            .path()
                            .extension()
                            .unwrap_or_default()
                            .to_str()
                            .unwrap_or_default()
                            == "ini"
                    {
                        continue;
                    }

                    is_empty = false;

                    let child = Button {
                        color: RGBA::ZERO,
                        height: Px(30.0),
                        width: Relative(1.0),
                        padding: OutArea::horizontal(Px(10.0)),
                        corner: [Px(5.0); 4],
                        callback: FnPtr::new(on_click),
                        message: OPEN,
                        childs: vec![
                            Text {
                                text: name,
                                ..Default::default()
                            }
                            .wrap("", &ui),
                        ],
                        ..Default::default()
                    };
                    ui.add_child_to(
                        child,
                        if entry.path().is_dir() {
                            "folder"
                        } else {
                            "file"
                        },
                        self.content_window,
                    );
                }

                if is_empty {
                    let child = Container {
                        color: RGBA::ZERO,
                        height: Px(50.0),
                        width: Relative(1.0),
                        padding: OutArea::horizontal(Px(2.0)),
                        childs: vec![
                            Text {
                                text: "This Folder is Empty".to_string(),
                                color: RGBA::RED,
                                align: Align::Center,
                                ..Default::default()
                            }
                            .wrap("empty_msg", &ui),
                        ],
                        ..Default::default()
                    };
                    ui.add_child_to(child, "", self.content_window);
                }
            }
            Err(error) => {
                if let Some(path) = self.path.parent() {
                    self.path = path.into();
                }

                let e_message = Ticking {
                    inner: Absolute {
                        color: RGBA::rgb(255, 255, 255),
                        x: Px(ui.cursor_pos.x),
                        y: Px(ui.cursor_pos.y),
                        border: [1.0; 4],
                        width: Auto,
                        height: Auto,
                        padding: OutArea::new(3.0),
                        corner: [Px(4.0); 4],
                        childs: vec![
                            Text {
                                text: error.to_string(),
                                color: RGBA::RED,
                                ..Default::default()
                            }
                            .wrap("", &ui),
                        ],
                        ..Default::default()
                    },
                    tick: FnPtr::new(tick_error),
                    ..Default::default()
                };

                ui.add_element(e_message, "e_msg");
            }
        }
    }

    pub fn proceed_event(&mut self, event: QueuedEvent) {
        if event.event == UiEvent::Press {
            match event.message {
                OPEN => {
                    if event.element_name == "folder" {
                        {
                            let mut ui = self.ui.borrow_mut();
                            let element = ui.get_element(event.element_id).unwrap();
                            let text = element.get_text().unwrap();
                            self.path.push(text);
                        };
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
                        let mut ui = self.ui.borrow_mut();
                        let selected = ui.get_element(self.hovered_element).unwrap();
                        if selected.name == "folder" {
                            let text = selected.get_text().unwrap();
                            self.path.push(text);
                            drop(ui);
                            self.display_path();
                        } else {
                            let id = selected.id;
                            drop(ui);
                            self.open_file(id);
                        }
                    }
                    name => println!("{name}"),
                },

                _ => unreachable!(),
            }
        }
    }

    fn open_file(&mut self, clicked_id: u32) {
        let path = {
            let mut ui = self.ui.borrow_mut();
            let element = ui.get_element(clicked_id).unwrap();
            let text = element.get_text().unwrap();
            self.path.join(text)
        };

        let _ = if cfg!(target_os = "windows") {
            Command::new("cmd")
                .args(["/C", "start", "", path.to_str().unwrap()])
                .spawn()
        } else if cfg!(target_os = "linux") {
            Command::new("xdg-open").arg(path).spawn()
        } else {
            Command::new("open").arg(path).spawn()
        }
        .expect("Datei konnte nicht geöffnet werden")
        .wait();
    }

    pub fn right_click(&mut self) -> bool {
        let mut ui = self.ui.borrow_mut();

        if let Some(hovered) = ui.get_hovered() {
            if hovered.name == "file" || hovered.name == "folder" {
                self.hovered_element = hovered.id;

                if self.tool_tip != u32::MAX {
                    ui.remove_element(null_mut(), self.tool_tip);
                }

                let x = Px(ui.cursor_pos.x);
                let y = Px(ui.cursor_pos.y);

                let tool_tip = Absolute {
                    x,
                    y,
                    width: Px(200.0),
                    height: Auto,
                    color: RGBA::rgb(50, 50, 50),
                    border_color: RGBA::GREEN,
                    border: [1.0; 4],
                    corner: [Px(7.0); 4],
                    childs: vec![
                        Button {
                            margin: OutArea::new(2.0),
                            // TODO Make fill work!
                            width: Relative(1.0),
                            height: Px(30.0),
                            color: RGBA::ZERO,
                            border_color: RGBA::BLUE,
                            border: [1.0; 4],
                            corner: [Px(5.0); 4],
                            callback: FnPtr::new(on_click),
                            message: ENTRY_ACTION,
                            childs: vec![
                                Text {
                                    text: "Öffnen".to_string(),
                                    color: RGBA::WHITE,
                                    ..Default::default()
                                }
                                .wrap("", &ui),
                            ],
                            ..Default::default()
                        }
                        .wrap("open", &ui),
                        Button {
                            margin: OutArea::new(2.0),
                            // TODO Make fill work!
                            width: Relative(1.0),
                            height: Px(30.0),
                            color: RGBA::ZERO,
                            border_color: RGBA::BLUE,
                            border: [1.0; 4],
                            corner: [Px(5.0); 4],
                            callback: FnPtr::new(on_click),
                            message: ENTRY_ACTION,
                            childs: vec![
                                Text {
                                    text: "Umbennenen".to_string(),
                                    color: RGBA::WHITE,
                                    ..Default::default()
                                }
                                .wrap("", &ui),
                            ],
                            ..Default::default()
                        }
                        .wrap("rename", &ui),
                        Button {
                            margin: OutArea::new(2.0),
                            // TODO Make fill work!
                            width: Relative(1.0),
                            height: Px(30.0),
                            color: RGBA::ZERO,
                            border_color: RGBA::BLUE,
                            border: [1.0; 4],
                            corner: [Px(5.0); 4],
                            callback: FnPtr::new(on_click),
                            message: ENTRY_ACTION,
                            childs: vec![
                                Text {
                                    text: "Löschen".to_string(),
                                    color: RGBA::WHITE,
                                    ..Default::default()
                                }
                                .wrap("", &ui),
                            ],
                            ..Default::default()
                        }
                        .wrap("delete", &ui),
                    ],
                    ..Default::default()
                };

                self.tool_tip = ui.add_element(tool_tip, "");
            }
            ui.dirty = DirtyFlags::Resize;
            true
        } else {
            false
        }
    }

    pub fn mouse_click(&mut self) -> bool {
        if self.tool_tip != u32::MAX {
            let mut ui = self.ui.borrow_mut();
            ui.remove_element(null_mut(), self.tool_tip);

            self.tool_tip = u32::MAX;
            ui.dirty = DirtyFlags::Resize;
            true
        } else {
            false
        }
    }
}

fn on_click(context: CallContext) {
    let button: &mut Button = context.element.downcast_mut();
    match button.state {
        ButtonState::Normal => {
            button.color = RGBA::ZERO;
        }
        ButtonState::Hovered => {
            button.color = RGBA::rgb(40, 40, 40);
        }
        ButtonState::Pressed => {
            button.color = RGBA::rgb(60, 60, 60);
        }
        ButtonState::Disabled => unreachable!(),
    }
    context.ui.dirty = DirtyFlags::Color;
}

fn tick_error(context: CallContext) {
    let this: &mut Ticking<Absolute> = context.element.downcast_mut();
    if this.last_tick.elapsed().as_secs_f32() > 1.0 {
        context.element.remove_self(context.ui);
        context.ui.dirty = DirtyFlags::Resize;
    }
}
