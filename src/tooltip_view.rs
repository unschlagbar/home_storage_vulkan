use iron_oxide::{
    graphics::formats::RGBA,
    primitives::Vec2,
    ui::{
        Absolute, Button, ButtonContext, ButtonState, ElementBuilder, QueuedEvent, Shadow, Text,
        TextExitContext, TextInput, Ui, UiElement, UiRect, UiUnit::*,
    },
};

use crate::explorer::{ENTRY_ACTION, Explorer};

#[derive(Clone, Copy, Default)]
pub struct ToolTipView {
    pub id: u32,
}

impl ToolTipView {
    pub const MESSAGE: u16 = ENTRY_ACTION;

    pub fn is_active(&self) -> bool {
        self.id != 0
    }

    pub fn create(&mut self, ui: &mut Ui, pos: Vec2<f32>) {
        if self.is_active() {
            let tools = ui.get_element_mut(self.id).unwrap();
            let abs = tools.downcast_mut::<Absolute>().unwrap();
            abs.offset = pos;
        } else {
            self.id = ui
                .add_child_to_root(
                    Absolute {
                        offset: pos,
                        width: Px(200.0),
                        height: Fit,
                        padding: UiRect::px(2.0),
                        color: RGBA::grey(50),
                        corner: [Px(7.0); 4],
                        shadow: Shadow::new(15, RGBA::rgba(25, 25, 25, 200)),
                        ..Default::default()
                    }
                    .wrap_childs(
                        "",
                        vec![
                            Self::button("open", "Öffnen"),
                            Self::button("cut", "Auschneiden"),
                            Self::button("copy", "Kopieren"),
                            Self::button("copy_path", "Pfad Kopieren"),
                            Self::button("rename", "Umbenennen"),
                            Self::button("delete", "Löschen"),
                            Self::button("properties", "Eigenschaften"),
                        ],
                    ),
                )
                .id();
        }
    }

    fn button(name: &'static str, value: &str) -> UiElement {
        Button {
            width: Relative(1.0),
            height: Px(30.0),
            color: RGBA::ZERO,
            border_color: RGBA::BLUE,
            corner: [Px(5.0); 4],
            callback: Some(Self::callback),
            message: Self::MESSAGE,
            ..Default::default()
        }
        .wrap_childs(
            name,
            vec![
                Text {
                    text: value.to_string(),
                    color: RGBA::grey(220),
                    ..Default::default()
                }
                .wrap_transparent(""),
            ],
        )
    }

    pub fn remove(&mut self, ui: &mut Ui) {
        if self.is_active() {
            ui.remove_element(self.id);

            self.id = 0;
            ui.layout_changed();
        }
    }

    pub fn proceed_event(&mut self, event: QueuedEvent, exp: &mut Explorer) {
        let data = &mut exp.data;
        match event.element_name {
            "open" => {
                let selected = {
                    let mut ui = data.ui.borrow_mut();
                    ui.get_element(data.selected_file).unwrap()
                };

                if selected.name == "folder" {
                    let text_stuff = selected.child(1).unwrap();
                    let text = text_stuff.get_text().unwrap();
                    data.path.push(text);
                    data.display_path();
                } else {
                    let id = selected.id();
                    data.open_file(id);
                }
            }
            "rename" => {
                let mut ui = data.ui.borrow_mut();

                let mut selected = ui.get_element(data.selected_file).unwrap();
                let button: &mut Button = selected.downcast_mut(&mut ui).unwrap();
                button.border_color = RGBA::GREEN;
                button.callback = None;

                let text_element = selected.child(1).unwrap().child(0).unwrap();
                let text_element = ui.remove_element(text_element).unwrap();
                let text = text_element.downcast().unwrap();

                let mut text_input = TextInput::from(text);
                text_input.on_blur = Some(Self::on_submit);

                let mut text_input = ui
                    .add_child(text_input.wrap("inline rename"), selected.child(1).unwrap())
                    .unwrap();
                TextInput::focus(&mut ui, text_input);
                text_input
                    .downcast_mut::<TextInput>(&mut ui)
                    .unwrap()
                    .set_cursor();
            }
            "properties" => {
                let mut ui = data.ui.borrow_mut();
                exp.properties_view.create(&mut ui, &data);
                ui.layout_changed();
            }
            name => println!("{name}"),
        }
    }

    pub fn callback(mut context: ButtonContext) {
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

    fn on_submit(context: TextExitContext) {
        let element = context.element;
        let parent = element.parent.unwrap().parent.unwrap().get_mut(context.ui);

        let container: &mut Button = parent.downcast_mut().unwrap();
        container.border_color = RGBA::ZERO;
        container.callback = Some(Self::callback);

        context.ui.color_changed();
    }
}
