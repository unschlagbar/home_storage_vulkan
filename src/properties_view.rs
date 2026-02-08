use std::fs::File;
use std::path::PathBuf;
use std::time::UNIX_EPOCH;

use iron_oxide::graphics::formats::RGBA;
use iron_oxide::primitives::Date;
use iron_oxide::ui::{
    Absolute, Button, ButtonContext, ButtonState, Container, ElementBuilder, FlexAlign,
    FlexDirection, Image, QueuedEvent, Shadow, Text, TextInput, Ui, UiElement, UiRect,
};
use iron_oxide::ui::{Align, UiUnit::*};

use crate::UiIcons;
use crate::explorer::{ExplorerData, PROPERTIES_ACTION};
use crate::file_size::FileSize;

#[derive(Default)]
pub struct PropertiesView {
    pub id: u32,
}

impl PropertiesView {
    pub const MESSAGE: u16 = PROPERTIES_ACTION;

    #[allow(unused)]
    pub fn is_active(&self) -> bool {
        self.id != 0
    }

    pub fn create(&mut self, ui: &mut Ui, data: &ExplorerData) {
        let element = ui.get_element(data.selected_file).unwrap();
        let text_box = element.child(1).unwrap();
        let text_element = text_box.child(0).unwrap();
        let text: &Text = text_element.downcast_ref().unwrap();

        let name = &text.text;
        let is_dir = element.name == "folder";
        let extention = name.split(".").last().unwrap_or_default();

        let path = data.path.clone().join(name);
        let file = File::open(&path).unwrap();

        let icon = if is_dir {
            UiIcons::Folder
        } else {
            match extention {
                "rs" => UiIcons::RustFile,
                "blend" | "blend1" => UiIcons::Blender,
                "code-workspace" => UiIcons::VSCode,
                _ => UiIcons::TxtFile,
            }
        } as u32;

        self.id = ui
            .add_child_to_root(
                Absolute {
                    align: Align::Center,
                    width: Px(400.0),
                    height: Fit,
                    color: RGBA::grey(35),
                    corner: [Px(7.0); 4],
                    border: [1; 4],
                    border_color: RGBA::GREEN,
                    shadow: Shadow::new(15, RGBA::rgba(25, 25, 25, 200)),
                    ..Default::default()
                }
                .wrap_childs(
                    "",
                    vec![
                        Container {
                            color: RGBA::ZERO,
                            height: Px(35.0),
                            width: Relative(1.0),
                            border: [0, 0, 0, 1],
                            border_color: RGBA::GREEN,
                            padding: UiRect::px(5.0),
                            flex_direction: FlexDirection::Horizontal,
                            ..Default::default()
                        }
                        .wrap_childs(
                            "",
                            vec![
                                Container {
                                    color: RGBA::ZERO,
                                    width: Fill(2.0),
                                    height: Relative(1.0),
                                    ..Default::default()
                                }
                                .wrap_childs(
                                    "",
                                    vec![
                                        Text {
                                            text: "Eigenschaften".to_string(),
                                            align: Align::Left,
                                            ..Default::default()
                                        }
                                        .wrap_transparent(""),
                                    ],
                                ),
                                Button {
                                    color: RGBA::ZERO,
                                    width: RelativeHeight(1.0),
                                    height: Relative(1.0),
                                    padding: UiRect::px(6.0),
                                    callback: Some(on_click),
                                    message: Self::MESSAGE,
                                    ..Default::default()
                                }
                                .wrap_childs(
                                    "close",
                                    vec![
                                        Image {
                                            atlas_index: UiIcons::Close as u32,
                                            color: RGBA::grey(200),
                                            ..Default::default()
                                        }
                                        .wrap_transparent("close_image"),
                                    ],
                                ),
                            ],
                        ),
                        Container {
                            color: RGBA::ZERO,
                            height: Fit,
                            width: Relative(1.0),
                            border: [0, 0, 0, 1],
                            border_color: RGBA::GREEN,
                            flex_direction: FlexDirection::Horizontal,
                            ..Default::default()
                        }
                        .wrap_childs_transparent(
                            "",
                            vec![
                                Container {
                                    color: RGBA::ZERO,
                                    margin: UiRect::px(16.0),
                                    height: Px(48.0),
                                    width: Px(48.0),
                                    ..Default::default()
                                }
                                .wrap_childs_transparent(
                                    "",
                                    vec![
                                        Image {
                                            max_width: Px(64.0),
                                            atlas_index: icon,
                                            ..Default::default()
                                        }
                                        .wrap_childs_transparent("", Vec::new()),
                                    ],
                                ),
                                Container {
                                    color: RGBA::grey(25),
                                    margin: UiRect::right(16.0),
                                    height: Fit,
                                    width: Fill(1.0),
                                    flex_align: FlexAlign::Center,
                                    border: [1; 4],
                                    corner: [Px(5.0); 4],
                                    padding: UiRect::px(5.0),
                                    ..Default::default()
                                }
                                .wrap_childs_transparent(
                                    "",
                                    vec![
                                        TextInput {
                                            text: name.clone(),
                                            ..Default::default()
                                        }
                                        .wrap(""),
                                    ],
                                ),
                            ],
                        ),
                        Container {
                            color: RGBA::ZERO,
                            height: Fit,
                            width: Relative(1.0),
                            padding: UiRect::px(5.0),
                            ..Default::default()
                        }
                        .wrap_childs_transparent("", attributes(file, path)),
                    ],
                ),
            )
            .id();
    }

    #[allow(unused)]
    pub fn proceed_event(&mut self, event: QueuedEvent, data: &mut ExplorerData) {
        match event.element_name {
            "close" => {
                let mut ui = data.ui.borrow_mut();
                ui.remove_element(self.id).unwrap();
                self.id = 0;
            }
            _ => unreachable!(),
        }
    }
}

fn on_click(mut context: ButtonContext) {
    let button: &mut Button = context.element.get_mut(context.ui).downcast_mut().unwrap();

    match button.state {
        ButtonState::Normal => {
            button.color = RGBA::ZERO;
        }
        ButtonState::Hovered => {
            button.color = RGBA::grey(50);
        }
        ButtonState::Pressed => {
            button.color = RGBA::grey(70);
        }
        ButtonState::Disabled => unreachable!(),
    }
    context.ui.color_changed();
}

fn attributes(file: File, path: PathBuf) -> Vec<UiElement> {
    let meta = file.metadata().unwrap();
    let file_type = meta.file_type();

    let mut file_size = FileSize(meta.len()).to_string();
    if meta.len() >= 1024 {
        file_size += &format!("\n{}B", meta.len())
    }

    let create_time = meta.created().unwrap().duration_since(UNIX_EPOCH).unwrap();
    let create_time = Date::from_unix_secs(create_time.as_secs());

    let modify_time = meta.modified().unwrap().duration_since(UNIX_EPOCH).unwrap();
    let modify_time = Date::from_unix_secs(modify_time.as_secs());

    let access_time = meta.accessed().unwrap().duration_since(UNIX_EPOCH).unwrap();
    let access_time = Date::from_unix_secs(access_time.as_secs());

    let typ = if file_type.is_symlink() {
        "System Link"
    } else if file_type.is_dir() {
        "Folder"
    } else if file_type.is_file() {
        "File"
    } else {
        "Unknown"
    }
    .to_string();

    let location = path.display().to_string();

    vec![
        attri("Type".to_string(), typ),
        attri("Location".to_string(), location),
        attri("Size".to_string(), file_size),
        attri("Created".to_string(), create_time.to_string()),
        attri("Modified".to_string(), modify_time.to_string()),
        attri("Accessed".to_string(), access_time.to_string()),
    ]
}

fn attri(name: String, value: String) -> UiElement {
    Container {
        color: RGBA::ZERO,
        width: Relative(1.0),
        height: Fit,
        padding: UiRect::from(&[16.0, 8.0, 16.0, 8.0]),
        flex_direction: FlexDirection::Horizontal,
        ..Default::default()
    }
    .wrap_childs(
        "",
        vec![
            Container {
                color: RGBA::ZERO,
                width: Relative(0.3),
                height: Fit,
                ..Default::default()
            }
            .wrap_childs(
                "",
                vec![
                    Text {
                        text: name,
                        ..Default::default()
                    }
                    .wrap(""),
                ],
            ),
            Container {
                color: RGBA::ZERO,
                width: Relative(0.7),
                height: Fit,
                ..Default::default()
            }
            .wrap_childs(
                "",
                vec![
                    Text {
                        text: value,
                        ..Default::default()
                    }
                    .wrap(""),
                ],
            ),
        ],
    )
}
