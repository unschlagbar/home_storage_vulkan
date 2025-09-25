use iron_oxide::{
    graphics::formats::Color,
    ui::{Container, FlexDirection, TypeConst, UiState, UiUnit::*},
};

pub fn build_main() -> UiState {
    let mut state = UiState::create(true);

    let root = Container {
        color: Color::ZERO,
        height: Relative(1.0),
        width: Relative(1.0),
        flex_direction: FlexDirection::Horizontal,
        childs: vec![
            Container {
                color: Color::rgb(20, 20, 20),
                height: Relative(1.0),
                width: Px(200.0),
                ..Default::default()
            }
            .wrap(&state),
        ],
        ..Default::default()
    };

    state.add_element(root);
    state
}
