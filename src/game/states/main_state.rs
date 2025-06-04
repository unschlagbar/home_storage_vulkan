use iron_oxide::{
    graphics::formats::Color,
    ui::{
        AbsoluteLayout,
        Align,
        Button,
        ButtonState,
        CallContext,
        Container,
        DirtyFlags,
        ElementBuild,
        ErasedFnPointer,
        OutArea,
        Text,
        UiState,
        UiUnit::*
    }
};

pub fn build_main() -> UiState {
    let mut state = UiState::create(true);

    state.add_element(
        AbsoluteLayout {
            color: Color::new(0.03, 0.03, 0.03, 1.0),
            align: Align::Left,
            height: Relative(1.0),
            width: Px(200.0),
            x: Zero,
            y: Zero,
            padding: OutArea::horizontal(Px(20.0)),
            childs: vec![
                Container {
                    margin: OutArea::vertical(Px(20.0)),
                    width: Relative(1.0),
                    height: Px(32.0),
                    color: Color::new(0.06, 0.06, 0.06, 1.0),
                    childs: vec![
                        Text { 
                            text: "Vulkan is the best!".to_string(),
                            color: Color::RED,
                            align: Align::Center,
                            ..Default::default()
                        }.wrap(&state)
                    ],
                    ..Default::default()
                }.wrap(&state),
                Button {
                    margin: OutArea::vertical(Px(1.0)),
                    width: Relative(1.0),
                    height: Px(41.0),
                    color: Color::new(0.06, 0.06, 0.06, 1.0),
                    childs: vec![
                        Text { 
                            text: "Normal".to_string(),
                            color: Color::RED,
                            align: Align::Center,
                            ..Default::default()
                        }.wrap(&state)
                    ],
                    callback: ErasedFnPointer::from_free(on_click),
                    ..Default::default()
                }.wrap(&state),
                Button {
                    margin: OutArea::vertical(Px(1.0)),
                    width: Relative(1.0),
                    height: Px(41.0),
                    color: Color::new(0.06, 0.06, 0.06, 1.0),
                    childs: vec![
                        Text { 
                            text: "Normal".to_string(),
                            color: Color::RED,
                            align: Align::Center,
                            ..Default::default()
                        }.wrap(&state)
                    ],
                    callback: ErasedFnPointer::from_free(on_click),
                    ..Default::default()
                }.wrap(&state),
                Button {
                    margin: OutArea::vertical(Px(1.0)),
                    width: Relative(1.0),
                    height: Px(41.0),
                    color: Color::new(0.06, 0.06, 0.06, 1.0),
                    childs: vec![
                        Text { 
                            text: "Normal".to_string(),
                            color: Color::RED,
                            align: Align::Center,
                            ..Default::default()
                        }.wrap(&state)
                    ],
                    callback: ErasedFnPointer::from_free(on_click),
                    ..Default::default()
                }.wrap(&state),
                Container {
                    margin: OutArea::vertical(Px(1.0)),
                    width: Relative(1.0),
                    height: Px(41.0),
                    color: Color::new(0.06, 0.06, 0.06, 1.0),
                    border: [0.0; 4],
                    ..Default::default()
                }.wrap(&state),
                Container {
                    margin: OutArea::vertical(Px(1.0)),
                    width: Relative(1.0),
                    height: Px(41.0),
                    color: Color::new(0.06, 0.06, 0.06, 1.0),
                    border: [0.0; 4],
                    ..Default::default()
                }.wrap(&state),
                Container {
                    margin: OutArea::vertical(Px(1.0)),
                    width: Relative(1.0),
                    height: Px(41.0),
                    color: Color::new(0.06, 0.06, 0.06, 1.0),
                    border: [0.0; 4],
                    ..Default::default()
                }.wrap(&state),
            ],
            ..Default::default()
        }
    );

    state
}

fn on_click(context: CallContext) {
    let button: &mut Button = unsafe { context.element.downcast_mut() };
    let text: &mut Text = unsafe { button.childs[0].downcast_mut() };
    match button.state {
        ButtonState::Normal => {
            button.color = Color::new(0.06, 0.06, 0.06, 1.0);
            text.set_new("Button");
        },
        ButtonState::Hovered => {
            button.color = Color::new(0.1, 0.1, 0.1, 1.0);
            text.set_new("Hover");
        },
        ButtonState::Pressed => {
            button.color = Color::new(0.2, 0.2, 0.2, 1.0);
            text.set_new("Press");
        },
        ButtonState::Disabled => unreachable!(),
    }
    context.ui.dirty = DirtyFlags::Color;
}