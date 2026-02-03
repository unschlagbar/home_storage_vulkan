use std::env;
use std::fs::File;
use std::io::ErrorKind;
use std::{cell::RefCell, fs, path::PathBuf, rc::Rc};

use iron_oxide::ui::text_layout::{TextLayout, TextOverflow};
use iron_oxide::ui::{
    Absolute, Align, ElementBuilder, FlexDirection, Image, ScrollPanel, Shadow, TextExitContext,
    TextInput, Ticking, UiRef, UiUnit,
};
use iron_oxide::{
    graphics::formats::RGBA,
    ui::{
        Button, ButtonContext, ButtonState, Container, QueuedEvent, Text, Ui, UiEvent, UiRect,
        UiUnit::*,
    },
};
use winit::window::CursorIcon;

use crate::UiIcons;
use crate::file_size::FileSize;
use crate::properties_view::PropertiesView;
use crate::tooltip_view::ToolTipView;

pub const OPEN: u16 = 1;
pub const ENTRY_ACTION: u16 = 2;
pub const GO_BACK: u16 = 3;

pub struct Explorer {
    pub content_window: u32,

    pub selected_file: u32,
    pub path_bar: u32,

    pub properties_view: PropertiesView,
    pub tooltip_view: ToolTipView,

    pub path: PathBuf,
    pub ui: Rc<RefCell<Ui>>,
}

impl Explorer {
    pub fn new(ui: Rc<RefCell<Ui>>) -> Self {
        let path_bar;
        let path: PathBuf = env::var(Self::HOME)
            .ok()
            .unwrap_or(Self::ROOT_PATH.to_string())
            .into();

        let content_window = {
            let mut ui = ui.borrow_mut();

            let root = ui
                .add_child_to_root(
                    Container {
                        color: RGBA::ZERO,
                        width: UiUnit::FILL,
                        height: UiUnit::FILL,
                        ..Default::default()
                    }
                    .wrap("root"),
                )
                .id();

            let nav_bar = ui
                .add_child(
                    Container {
                        color: RGBA::grey(20),
                        width: UiUnit::FILL,
                        height: Px(40.0),
                        flex_direction: FlexDirection::Horizontal,
                        padding: UiRect::px(5.0),
                        ..Default::default()
                    }
                    .wrap("nav_bar"),
                    root,
                )
                .unwrap()
                .id();

            //Back Button
            ui.add_child(
                Button {
                    color: RGBA::ZERO,
                    width: RelativeHeight(1.0),
                    height: RelativeHeight(1.0),
                    padding: UiRect::px(6.0),
                    callback: Some(on_click),
                    message: GO_BACK,
                    ..Default::default()
                }
                .wrap_childs(
                    "back",
                    vec![
                        Image {
                            atlas_index: UiIcons::Back as u32,
                            color: RGBA::grey(200),
                            ..Default::default()
                        }
                        .wrap_transparent("back_image"),
                    ],
                ),
                nav_bar,
            )
            .unwrap()
            .id();

            let path_bar_parent = ui
                .add_child(
                    Container {
                        color: RGBA::grey(35),
                        width: Fill(1.0),
                        height: UiUnit::Fill(1.0),
                        margin: UiRect::left(5.0),
                        padding: UiRect::horizontal(Px(15.0)),
                        corner: [RelativeHeight(0.5); 4],
                        border: [0; 4],
                        border_color: RGBA::grey(100),
                        ..Default::default()
                    }
                    .wrap_childs("pathbar", Vec::with_capacity(1)),
                    nav_bar,
                )
                .unwrap()
                .id();

            path_bar = ui
                .add_child(
                    TextInput {
                        text: path.to_str().unwrap().to_string(),
                        align: Align::Left,
                        layout: TextLayout {
                            overflow: TextOverflow::Ellipsis,
                            ..Default::default()
                        },
                        ..Default::default()
                    }
                    .wrap("lio"),
                    path_bar_parent,
                )
                .unwrap()
                .id();

            let content = ui
                .add_child(
                    Container {
                        color: RGBA::grey(25),
                        width: UiUnit::FILL,
                        height: UiUnit::Fill(1.0),
                        border: [0, 1, 0, 0],
                        border_color: RGBA::grey(70),
                        padding: UiRect::px(2.0),
                        ..Default::default()
                    }
                    .wrap("content"),
                    root,
                )
                .unwrap()
                .id();

            ui.add_child(
                ScrollPanel {
                    padding: UiRect::px(2.0),
                    ..Default::default()
                }
                .wrap("scroll_pannel"),
                content,
            )
            .unwrap()
            .id()
        };

        Self {
            content_window,
            selected_file: 0,
            path_bar,
            path,
            ui,
            properties_view: PropertiesView::default(),
            tooltip_view: ToolTipView::default(),
        }
    }

    pub fn display_path(&mut self) {
        let mut ui = self.ui.borrow_mut();

        let path_string = self.path.to_str().unwrap().to_string();
        let mut path_bar = ui.get_element(self.path_bar).unwrap();
        let path_bar_widget: &mut TextInput = path_bar.downcast_mut(&mut ui).unwrap();

        path_bar_widget.set_new(path_string);
        path_bar_widget.color = RGBA::grey(200);

        match fs::read_dir(&self.path) {
            Ok(entries) => {
                let content = ui.get_element(self.content_window).unwrap();

                ui.remove_all_elements(content);

                let mut is_empty = true;
                for entry in entries {
                    let entry = entry.unwrap();
                    let name = entry.file_name().into_string().unwrap();
                    let extention = name.split('.').next_back().unwrap_or_default();

                    if name.starts_with('.') || extention == "ini" {
                        continue;
                    }

                    is_empty = false;

                    let is_dir = entry.path().is_dir();

                    let (element_name, icon) = if is_dir {
                        ("folder", UiIcons::Folder as u32)
                    } else {
                        let icon = match extention {
                            "rs" => UiIcons::RustFile,
                            "blend" | "blend1" => UiIcons::Blender,
                            "code-workspace" => UiIcons::VSCode,
                            _ => UiIcons::TxtFile,
                        } as u32;
                        ("file", icon)
                    };

                    let button = Button {
                        color: RGBA::ZERO,
                        border_color: RGBA::GREEN,
                        height: Fit,
                        width: UiUnit::Fill(1.0),
                        flex_direction: FlexDirection::Horizontal,
                        padding: UiRect::horizontal(Px(2.0)),
                        corner: [Px(5.0); 4],
                        callback: Some(on_click),
                        cursor: CursorIcon::Default,
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
                                padding: UiRect::px(3.0),
                                ..Default::default()
                            }
                            .wrap_childs_transparent(
                                "image container",
                                vec![
                                    Image {
                                        atlas_index: icon,
                                        ..Default::default()
                                    }
                                    .wrap_childs_transparent("", Vec::new()),
                                ],
                            ),
                            Container {
                                height: Px(30.0),
                                width: Fill(1.0),
                                color: RGBA::TRANSPARENT,
                                padding: UiRect::left(5.0),
                                ..Default::default()
                            }
                            .wrap_childs_transparent(
                                "",
                                vec![
                                    Text {
                                        text: name,
                                        align: Align::Left,
                                        layout: TextLayout {
                                            overflow: TextOverflow::Ellipsis,
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    }
                                    .wrap_childs_transparent("text", Vec::new()),
                                ],
                            ),
                        ],
                    );

                    if !is_dir {
                        let file = File::open(entry.path()).unwrap();
                        let size = file.metadata().unwrap().len();
                        ui.add_child(
                            Text {
                                text: FileSize(size).to_string(),
                                color: RGBA::grey(120),
                                align: Align::Left,
                                ..Default::default()
                            }
                            .wrap_childs_transparent("text", Vec::new()),
                            UiRef::new_ref(&button),
                        );
                    }

                    ui.add_child(button, self.content_window);
                }

                if is_empty {
                    ui.add_child(
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
                if error.kind() == ErrorKind::NotFound {
                    path_bar.downcast_mut::<TextInput>(&mut ui).unwrap().color = RGBA::RED;
                } else if let Some(path) = self.path.parent() {
                    self.path = path.into();
                }

                let e_message = Ticking {
                    inner: Absolute {
                        color: RGBA::grey(45),
                        offset: ui.cursor_pos.into_f32(),
                        border: [1; 4],
                        width: Fit,
                        height: Fit,
                        padding: UiRect::px(5.0),
                        corner: [Px(5.0); 4],
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
        if event.event.is_release() {
            match event.message {
                OPEN => {
                    if event.element_name == "folder" {
                        let element = {
                            let mut ui = self.ui.borrow_mut();
                            ui.get_element(event.element_id).unwrap()
                        };

                        let text_stuff = element.child(1).unwrap();

                        let text = text_stuff.get_text().unwrap();
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
                        let selected = {
                            let mut ui = self.ui.borrow_mut();
                            ui.get_element(self.selected_file).unwrap()
                        };

                        if selected.name == "folder" {
                            let text_stuff = selected.child(1).unwrap();
                            let text = text_stuff.get_text().unwrap();
                            self.path.push(text);
                            self.display_path();
                        } else {
                            let id = selected.id();
                            self.open_file(id);
                        }
                    }
                    "rename" => {
                        let mut ui = self.ui.borrow_mut();

                        let mut selected = ui.get_element(self.selected_file).unwrap();
                        let button: &mut Button = selected.downcast_mut(&mut ui).unwrap();
                        button.border = [1; 4];
                        button.callback = None;

                        let text_element = selected.child(1).unwrap().child(0).unwrap();
                        let text_element = ui.remove_element(text_element).unwrap();
                        let text = text_element.downcast().unwrap();

                        let mut text_input = TextInput::from(text);
                        text_input.on_blur = Some(on_submit);

                        let mut text_input = ui
                            .add_child(text_input.wrap("input"), selected.child(1).unwrap())
                            .unwrap();
                        TextInput::focus(&mut ui, text_input);
                        text_input
                            .downcast_mut::<TextInput>(&mut ui)
                            .unwrap()
                            .set_cursor();
                    }
                    "properties" => {
                        let mut ui = self.ui.borrow_mut();
                        self.properties_view.create(&mut ui, self.selected_file);
                        ui.layout_changed();

                        let selected = self.selected_text(&mut ui);
                        let text_element = selected.child(0).unwrap();

                        let text: &Text = text_element.downcast_ref().unwrap();

                        self.path.push(&text.text);
                        let file = fs::File::open(&self.path).unwrap();
                        dbg!(file.metadata().unwrap());
                    }
                    name => println!("{name}"),
                },
                _ => unreachable!(),
            }
        } else if event.event == UiEvent::Submit {
            if event.element_id == self.path_bar {
                {
                    let mut ui = self.ui.borrow_mut();
                    let path_bar = ui.get_element_mut(event.element_id).unwrap();
                    let path_bar: &mut TextInput = path_bar.downcast_mut().unwrap();
                    if path_bar.text == self.path.to_str().unwrap() {
                        return;
                    } else {
                        self.path = PathBuf::from(
                            &path_bar.text.strip_suffix('/').unwrap_or(&path_bar.text),
                        );
                    }
                }
                self.display_path();
            } else {
                let mut ui = self.ui.borrow_mut();
                let text_input = ui.get_element(event.element_id).unwrap();

                let parent = text_input.as_ref().parent.unwrap();
                let text_input = ui.remove_element(text_input).unwrap();
                let text_input: TextInput = text_input.downcast().unwrap();
                let text = Text::from(text_input);

                ui.add_child(text.wrap_transparent("text"), parent).unwrap();
            }
        }
    }

    pub fn right_click(&mut self, ui: &mut Ui) -> bool {
        if let Some(hovered) = ui.get_hovered() {
            if hovered.name == "file" || hovered.name == "folder" {
                self.selected_file = hovered.id();

                let pos = ui.cursor_pos.into_f32();
                self.tooltip_view.create(ui, pos);
            }
            ui.layout_changed();
            true
        } else {
            false
        }
    }

    pub fn mouse_click(&mut self, ui: &mut Ui) {
        self.tooltip_view.remove(ui);
    }

    pub fn selected_text(&self, ui: &mut Ui) -> UiRef {
        ui.get_element(self.selected_file)
            .unwrap()
            .child(1)
            .unwrap()
    }
}

fn on_click(mut context: ButtonContext) {
    let button: &mut Button = context.element.get_mut(context.ui).downcast_mut().unwrap();

    match button.state {
        ButtonState::Normal => {
            button.color = RGBA::ZERO;
            button.shadow = Shadow::default();
        }
        ButtonState::Hovered => {
            button.color = RGBA::grey(40);
            button.shadow = Shadow::new(15, RGBA::RED);
            println!("efew");
        }
        ButtonState::Pressed => {
            button.color = RGBA::grey(60);
            button.shadow = Shadow::new(15, RGBA::rgba(25, 25, 25, 200));
        }
        ButtonState::Disabled => unreachable!(),
    }
    context.ui.color_changed();
}

pub fn on_click_tooltip(mut context: ButtonContext) {
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

fn tick_error(context: ButtonContext) {
    let this: &Ticking<Absolute> = context.element.downcast_ref().unwrap();
    if this.time.elapsed().as_millis() > 750 {
        context.ui.remove_element(context.element);
        context.ui.color_changed();
    }
}

fn on_submit(context: TextExitContext) {
    let element = context.element;
    let parent = element.parent.unwrap().parent.unwrap().get_mut(context.ui);

    let container: &mut Button = parent.downcast_mut().unwrap();
    container.border = [0; 4];
    container.callback = Some(on_click);

    context.ui.color_changed();
}
