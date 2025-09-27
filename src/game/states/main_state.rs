use iron_oxide::{
    graphics::formats::Color,
    ui::{Container, UiState, UiUnit::*},
};

pub fn build_main() -> UiState {
    let mut state = UiState::create(true);

    let root = Container {
        color: Color::rgb(20, 20, 20),
        height: Relative(1.0),
        width: Relative(1.0),
        ..Default::default()
    };

    state.add_element(root);
    state
}
