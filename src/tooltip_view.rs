use iron_oxide::{
    graphics::formats::RGBA,
    primitives::Vec2,
    ui::{
        Absolute, Button, ButtonContext, ButtonState, ElementBuilder, Shadow, Text, Ui, UiRect,
        UiUnit::{self, *},
    },
};

use crate::explorer::ENTRY_ACTION;

pub struct ToolTipView {
    pub id: u32,
}

impl ToolTipView {
    pub fn is_active(&self) -> bool {
        self.id != u32::MAX
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
                            Button {
                                width: UiUnit::FILL,
                                height: Px(30.0),
                                color: RGBA::ZERO,
                                border_color: RGBA::BLUE,
                                corner: [Px(5.0); 4],
                                callback: Some(Self::callback),
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
                                    .wrap_transparent(""),
                                ],
                            ),
                            Button {
                                width: UiUnit::FILL,
                                height: Px(30.0),
                                color: RGBA::ZERO,
                                border_color: RGBA::BLUE,
                                corner: [Px(5.0); 4],
                                callback: Some(Self::callback),
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
                                    .wrap_transparent(""),
                                ],
                            ),
                            Button {
                                width: Relative(1.0),
                                height: Px(30.0),
                                color: RGBA::ZERO,
                                border_color: RGBA::BLUE,
                                corner: [Px(5.0); 4],
                                callback: Some(Self::callback),
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
                                    .wrap_transparent(""),
                                ],
                            ),
                            Button {
                                width: Relative(1.0),
                                height: Px(30.0),
                                color: RGBA::ZERO,
                                border_color: RGBA::BLUE,
                                corner: [Px(5.0); 4],
                                callback: Some(Self::callback),
                                message: ENTRY_ACTION,
                                ..Default::default()
                            }
                            .wrap_childs(
                                "properties",
                                vec![
                                    Text {
                                        text: "Eigenschaften".to_string(),
                                        color: RGBA::grey(220),
                                        ..Default::default()
                                    }
                                    .wrap_transparent(""),
                                ],
                            ),
                        ],
                    ),
                )
                .id();
        }
    }

    pub fn remove(&mut self, ui: &mut Ui) {
        if self.is_active() {
            ui.remove_element(self.id);

            self.id = u32::MAX;
            ui.layout_changed();
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
}

impl Default for ToolTipView {
    fn default() -> Self {
        Self { id: u32::MAX }
    }
}
