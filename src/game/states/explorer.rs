use std::env;
use std::{cell::RefCell, fs, path::PathBuf, process::Command, rc::Rc};

use iron_oxide::ui::{AbsoluteLayout, Align, ScrollPanel, Ticking, TypeConst};
use iron_oxide::{
    graphics::formats::Color,
    ui::{
        Button, ButtonState, CallContext, Container, DirtyFlags, ErasedFnPointer, OutArea,
        QueuedEvent, Text, UiEvent, UiState, UiUnit,
    },
};

const FOLDER_CLICK: u16 = 1;
const FILE_CLICK: u16 = 2;
const GO_BACK: u16 = 3;

pub struct Explorer {
    pub content_window: u32,
    #[allow(unused)]
    pub path: PathBuf,
    pub ui: Rc<RefCell<UiState>>,
}

impl Explorer {
    pub fn new(ui: Rc<RefCell<UiState>>) -> Self {
        let content_window = {
            let mut ui = ui.borrow_mut();
            ui.add_child_to(
                Container {
                    color: Color::ZERO,
                    width: UiUnit::Relative(1.0),
                    height: UiUnit::Relative(1.0),
                    ..Default::default()
                },
                1,
            );
            ui.add_child_to(
                Container {
                    color: Color::rgb(20, 20, 20),
                    width: UiUnit::Relative(1.0),
                    height: UiUnit::Px(40.0),
                    ..Default::default()
                },
                2,
            );
            ui.add_child_to(
                Button {
                    color: Color::rgb(20, 20, 20),
                    width: UiUnit::Px(40.0),
                    height: UiUnit::Px(40.0),
                    margin: OutArea::new(3.0),
                    callback: ErasedFnPointer::from_free(on_click),
                    message: GO_BACK,
                    ..Default::default()
                },
                3,
            );
            ui.add_child_to(
                Container {
                    color: Color::rgb(30, 30, 30),
                    width: UiUnit::Fill,
                    height: UiUnit::Fill,
                    border: [1.0; 4],
                    border_color: Color::rgb(100, 100, 100),
                    padding: OutArea::new(2.0),
                    ..Default::default()
                },
                2,
            );
            ui.add_child_to(
                ScrollPanel {
                    padding: OutArea::new(2.0),
                    ..Default::default()
                },
                5,
            )
            .unwrap()
        };
        Self {
            content_window,
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
                        color: Color::ZERO,
                        height: UiUnit::Px(30.0),
                        width: UiUnit::Relative(1.0),
                        padding: OutArea::horizontal(UiUnit::Px(2.0)),
                        corner: [UiUnit::Px(5.0); 4],
                        callback: ErasedFnPointer::from_free(on_click),
                        message: if entry.path().is_dir() {
                            FOLDER_CLICK
                        } else {
                            FILE_CLICK
                        },
                        childs: vec![
                            Text {
                                text: name,
                                ..Default::default()
                            }
                            .wrap(&ui),
                        ],
                        ..Default::default()
                    };
                    ui.add_child_to(child, self.content_window);
                }

                if is_empty {
                    let child = Container {
                        color: Color::ZERO,
                        height: UiUnit::Px(50.0),
                        width: UiUnit::Relative(1.0),
                        padding: OutArea::horizontal(UiUnit::Px(2.0)),
                        childs: vec![
                            Text {
                                text: "This Folder is Empty".to_string(),
                                color: Color::RED,
                                align: Align::Center,
                                ..Default::default()
                            }
                            .wrap(&ui),
                        ],
                        ..Default::default()
                    };
                    ui.add_child_to(child, self.content_window);
                }
            }
            Err(error) => {
                if let Some(path) = self.path.parent() {
                    self.path = path.into();
                }

                let e_message = Ticking {
                    inner: AbsoluteLayout {
                        x: UiUnit::Px(ui.cursor_pos.x),
                        y: UiUnit::Px(ui.cursor_pos.y),
                        border: [1.0; 4],
                        width: UiUnit::Auto,
                        height: UiUnit::Auto,
                        padding: OutArea::new(3.0),
                        corner: [UiUnit::Px(4.0); 4],
                        childs: vec![
                            Text {
                                text: error.to_string(),
                                color: Color::RED,
                                ..Default::default()
                            }
                            .wrap(&ui),
                        ],
                        ..Default::default()
                    },
                    callback: ErasedFnPointer::from_free(tick_error),
                    ..Default::default()
                };

                ui.add_element(e_message);
            }
        }
    }

    pub fn proceed_event(&mut self, event: QueuedEvent) {
        if matches!(event.event, UiEvent::Press) {
            match event.message {
                FOLDER_CLICK => {
                    {
                        let mut ui = self.ui.borrow_mut();
                        let element = ui.get_element(event.element_id).unwrap();
                        let text = element.get_text().unwrap();
                        self.path.push(text);
                    };
                    self.display_path();
                }
                GO_BACK => {
                    if let Some(path) = self.path.parent() {
                        self.path = path.into();
                        self.display_path();
                    }
                }
                FILE_CLICK => {
                    let path = {
                        let mut ui = self.ui.borrow_mut();
                        let element = ui.get_element(event.element_id).unwrap();
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
                _ => unreachable!(),
            }
        }
    }
}

fn on_click(context: CallContext) {
    let button: &mut Button = context.element.downcast_mut();
    match button.state {
        ButtonState::Normal => {
            button.color = Color::ZERO;
        }
        ButtonState::Hovered => {
            button.color = Color::rgb(40, 40, 40);
        }
        ButtonState::Pressed => {
            button.color = Color::rgb(60, 60, 60);
        }
        ButtonState::Disabled => unreachable!(),
    }
    context.ui.dirty = DirtyFlags::Color;
}

fn tick_error(context: CallContext) {
    let this: &mut Ticking<AbsoluteLayout> = context.element.downcast_mut();
    if this.last_tick.elapsed().as_secs_f32() > 1.0 {
        context.element.remove_self(context.ui);
        context.ui.dirty = DirtyFlags::Resize;
    }
}
