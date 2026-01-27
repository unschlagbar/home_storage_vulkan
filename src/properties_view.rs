use iron_oxide::graphics::formats::RGBA;
use iron_oxide::ui::{Absolute, Button, ElementBuilder, Shadow, Text, Ui, UiRect};
use iron_oxide::ui::{Align, UiUnit::*};

use crate::explorer::on_click_tooltip;

#[derive(Default)]
pub struct PropertiesView {
    pub id: u32,
}

impl PropertiesView {
    pub fn build(&mut self, ui: &mut Ui) {
        self.id = ui.add_child_to_root(
            Absolute {
                align: Align::Center,
                width: Px(400.0),
                height: Auto,
                padding: UiRect::new(2.0),
                color: RGBA::grey(50),
                corner: [Px(7.0); 4],
                shadow: Shadow::new(15, RGBA::rgba(25, 25, 25, 200)),
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
                        message: 0,
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
                        width: Fill,
                        height: Px(30.0),
                        color: RGBA::ZERO,
                        border_color: RGBA::BLUE,
                        corner: [Px(5.0); 4],
                        callback: Some(on_click_tooltip),
                        message: 0,
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
                        callback: Some(on_click_tooltip),
                        message: 0,
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
                        callback: Some(on_click_tooltip),
                        message: 0,
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
