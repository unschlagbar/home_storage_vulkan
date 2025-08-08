use std::{cell::RefCell, fs, path::PathBuf, rc::Rc};

use iron_oxide::{
    graphics::formats::Color,
    ui::{
        Button, ButtonState, CallContext, Container, DirtyFlags, ElementBuild, ErasedFnPointer, FlexDirection, OutArea, Text, UiState, UiUnit
    },
};

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
                    width: UiUnit::Fill,
                    height: UiUnit::Relative(1.0),
                    flex_direction: FlexDirection::Vertical,
                    ..Default::default()
                },
                2,
            );
            ui.add_child_to(
                Container {
                    color: Color::rgb(20, 20, 20),
                    width: UiUnit::Fill,
                    height: UiUnit::Px(40.0),
                    ..Default::default()
                },
                3,
            );
            ui.add_child_to(
                Container {
                    color: Color::rgb(30, 30, 30),
                    width: UiUnit::Fill,
                    height: UiUnit::Fill,
                    padding: OutArea::new(4.0),
                    flex_direction: FlexDirection::Vertical,
                    border: [1.0; 4],
                    border_color: Color::rgb(150, 150, 150),
                    ..Default::default()
                },
                3,
            )
        };
        Self {
            content_window,
            path: PathBuf::new(),
            ui,
        }
    }

    pub fn display_path(&mut self, path: PathBuf) {
        let mut ui = self.ui.borrow_mut();
        let content = ui.get_element(self.content_window).unwrap();
        content.clear_childs();
        let struct_pointer = &(*self) as *const Self as *mut Self;

        let entries = fs::read_dir(path).unwrap();
        for entry in entries {
            let entry = entry.unwrap();
            let name = entry.file_name();
            let child = Button {
                color: Color::ZERO,
                height: UiUnit::Px(30.0),
                width: UiUnit::Relative(1.0),
                margin: OutArea::new(1.0),
                padding: OutArea::horizontal(UiUnit::Px(5.0)),
                corner: [UiUnit::Px(5.0); 4],
                callback: ErasedFnPointer::from_associated_vars::<Explorer>(struct_pointer, on_click),
                childs: vec![
                    Text {
                        text: name.to_str().unwrap().to_string(),
                        ..Default::default()
                    }
                    .wrap(&ui),
                ],
                ..Default::default()
            };
            ui.add_child_to(child, self.content_window);
        }
    }
}

fn on_click(explorer: &mut Explorer, context: CallContext) {
    let button: &mut Button = unsafe { context.element.downcast_mut() };
    let text: &mut Text = unsafe { button.childs[0].downcast_mut() };
    match button.state {
        ButtonState::Normal => {
            button.color = Color::ZERO;
        }
        ButtonState::Hovered => {
            button.color = Color::rgb(40, 40, 40);
        }
        ButtonState::Pressed => {
            button.color = Color::rgb(40, 40, 40);
            println!("path: , {}",&text.text);
            explorer.display_path(explorer.path.join(&text.text));
        }
        ButtonState::Disabled => unreachable!(),
    }
    context.ui.dirty = DirtyFlags::Color;
}
