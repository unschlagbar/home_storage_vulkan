#[allow(unused)]
    #[inline]
    pub fn update_cursor(&mut self, ui: &mut UiState, parent_size: Vec2, parent_pos: Vec2, cursor_pos: Vec2, ui_event: UiEvent) -> u8 {
        //0 = no event
        //1 = no event break
        //2 = old event
        //3 = new event

        if !self.visible {
            return 0;
        }

        let (self_size, self_pos) = (self.size, self.pos);

        if self_pos < cursor_pos {

            if self_pos.x + self_size.x > cursor_pos.x && self_pos.y + self_size.y > cursor_pos.y {

                for child in self.element.childs() {
                    let result = child.update_cursor(ui, self_size, self_pos, cursor_pos, ui_event);
                    if result > 0 { return result };
                }

                match ui_event {
                    UiEvent::Press => {
                        let mut dirty = false;
                        match &mut self_ptr.inherit {
                            UiType::Button(button) => {
                                button.interaction = Interaction::Pressed;
                                button.before_press.call(ui, self_ptr2);
                                self_ptr.dirty = true;
                                ui.pressed = Some(self.clone());
                            },
                            UiType::DragBox(dragbox) => {
                                dragbox.interaction = Interaction::Pressed;
                                ui.pressed = Some(self.clone());
                                self_ptr.dirty = true;
                                return 1;
                            },
                            UiType::CheckBox(checkbox) => {
                                checkbox.pressed = true;
                                ui.pressed = Some(self.clone());
                                self_ptr.dirty = true;
                            },
                            _ => return 1
                        };
                    },
                    UiEvent::Release => {
                        match &self.inherit {
                            UiType::Button(_) => (),
                            UiType::CheckBox(_) => (),
                            _ => return 1
                        };
                    },
                    UiEvent::Move => {
                        let mut dirty = false;
                        match &mut self_ptr.inherit {
                            UiType::Button(button) => {
                                if button.interaction != Interaction::Hover {

                                    button.interaction = Interaction::Hover;
                                    dirty = true;

                                    if let Some(raw_ref) = &mut ui.selected {
                                        let ptr = Rc::get_mut(raw_ref).unwrap();
                                        let ptr2 = unsafe { &mut *(ptr as *mut UiElement) };
                                        match &mut ptr.inherit {
                                            UiType::Button(b) => {
                                                b.interaction = Interaction::None;
                                                ptr2.dirty = true;
                                            },
                                            UiType::CheckBox(b) => {
                                                b.selected = false;
                                                ptr2.dirty = true;
                                            },
                                            _ => ()
                                        }
                                    }

                                    ui.selected = Some(self.clone());
                                } else {
                                    return 2;
                                }
                            },
                            UiType::CheckBox(checkbox) => {
                                if !checkbox.selected {

                                    checkbox.selected = true;
                                    dirty = true;

                                    if let Some(raw_ref) = &mut ui.selected {
                                        let ptr = Rc::make_mut(raw_ref);
                                        let ptr2 = unsafe { &mut *(ptr as *mut UiElement) };
                                        match &mut ptr.inherit {
                                            UiType::Button(b) => {
                                                b.interaction = Interaction::None;
                                                ptr2.dirty = true;
                                            },
                                            UiType::CheckBox(b) => {
                                                b.selected = false;
                                                ptr2.dirty = true;
                                            },
                                            _ => ()
                                        }
                                    }

                                    ui.selected = Some(self.clone());
                                } else {
                                    return 2;
                                }
                            },
                            _ => return 1
                        }

                        if dirty {
                            self_ptr2.dirty = true;
                        }
                    },
                };
                return 3;
            }
        }
        0
    }

        #[inline(always)]
    pub fn rebuild(&mut self, parent_size: Vec2, parent_pos: Vec2, font: &Font) {

        let style: &Style = match &self.inherit {
            UiType::Button(button) => {
                if button.interaction == Interaction::Pressed {
                    &button.press_style
                } else if button.interaction == Interaction::Hover {
                    &button.hover_style
                } else {
                    &self.style
                }
            },
            UiType::CheckBox(checkbox) => {
                if checkbox.enabled {
                    &checkbox.press_style
                } else if checkbox.selected {
                    &checkbox.hover_style
                } else {
                    &self.style
                }
            }
            _ => &self.style,
        };

        let mut context = BuildContext::default(font, parent_size);

        match style.position {
            Position::Absolute(absolute) => {
                self.computed.size = Vec2::new(
                    style.width.width(parent_size),
                    style.height.height(parent_size)
                );

                self.computed.pos = absolute.align.get_pos(parent_size, self.computed.size, Vec2::new(absolute.x.pixelx(parent_size), absolute.y.pixely(parent_size)));
            },
            Position::Inline(inline) => {

                let space = Vec2::new(
                    parent_size.x - inline.margin[0].pixelx(parent_size) - inline.margin[2].pixelx(parent_size),
                    parent_size.y -  inline.margin[1].pixely(parent_size) - inline.margin[3].pixely(parent_size)
                );
                
                let old_pos = self.computed.pos;
                let old_size = self.computed.size;

                self.computed.size = Vec2::new(
                    style.width.width(space),
                    style.height.height(space)
                );

                self.computed.pos = parent_pos + Vec2::new(
                    inline.margin[0].pixelx(parent_size),
                    inline.margin[1].pixely(parent_size),
                );

                let original_start_pos = self.get_offset();

                context.parent_pos = parent_pos;
                context.line_offset = self.computed.size.y;
                context.start_pos = original_start_pos;

                context.fits_in_line(&inline, &mut self.computed.pos, &mut self.computed.size);

                self.computed.pos = old_pos;

                if self.computed.pos != old_pos && self.computed.size != old_size && false {

                    if let Some(parent) = &mut self.parent {

                        for i in (self.computed.order as usize) + 1..parent.childs.len() {
                            let mut neightbour = parent.childs[i].clone();
  
                            let neightbour = Rc::get_mut(&mut neightbour).unwrap();
    
                            if context.parent_size.x - context.start_pos.x >= neightbour.computed.size.x {
                                neightbour.move_computed_absolute(original_start_pos);
    
                                context.line_offset = context.line_offset.max(neightbour.computed.size.y + inline.margin[1].pixely(context.parent_size) + inline.margin[3].pixely(context.parent_size));
                                context.start_pos.x += neightbour.computed.size.x + inline.margin[0].pixelx(context.parent_size) + inline.margin[2].pixelx(context.parent_size);
    
                            } else {
                                neightbour.move_computed(Vec2::new(0.0, context.line_offset));
                                neightbour.computed.pos.y += context.start_pos.y;
                                context.line_offset = neightbour.computed.size.y + inline.margin[1].pixely(context.parent_size) + inline.margin[3].pixely(context.parent_size);
                                context.start_pos.x = neightbour.computed.size.x + inline.margin[0].pixelx(context.parent_size) + inline.margin[2].pixelx(context.parent_size);
                            }
                        }
                    }
                }
            }
        };

        self.computed.color = style.color.as_color();
        self.computed.border_color = style.border_color.as_color();
        self.computed.border = style.border[0];
        self.computed.corner = style.corner[0].pixelx(self.computed.size);

        if let UiType::Text(text) = &mut self.inherit {
            text.build_text(&self.style, Vec2::zero(), self.computed.pos, &mut context);
        }

        self.dirty = false
    }

    pub fn update_cursor(&mut self, ui_size: Vec2, cursor_pos: Vec2, event: UiEvent) -> u8 {

        //0 = no event
        //1 = no event break
        //2 = old event
        //3 = new event
        let mut bol = 0;

        let ui = unsafe { &mut *(self as *mut UiState) };

        for i in self.elements.iter_mut().rev() {
            let result = i.update_cursor(ui, ui_size, Vec2::default(), cursor_pos, event);
            if result > 0 {
                bol = result;
                break;
            }
        }

        //Not old event
        if bol != 2 {
            if self.selected.is_some() && bol < 2 {
                let selected = &mut self.selected.take().unwrap();
                let selected = Rc::make_mut(selected);
                match &mut selected.inherit {
                    UiType::Button(button) => {
                        if button.interaction == Interaction::Hover {
                            button.interaction = Interaction::None;
                            selected.dirty = true;
                            self.dirty = true;
                            return 3;
                        }
                    },
                    UiType::CheckBox(checkbox) => {
                        if checkbox.selected {
                            checkbox.selected = false;
                            selected.dirty = true;
                            self.dirty = true;
                            return 3;
                        }
                    }
                    _ => todo!()
                }
            }
            if let Some(element) = &mut self.pressed {
                let element = Rc::make_mut(element);
                let is_in = element.is_in(cursor_pos);
                match event {
                    UiEvent::Move => {
                        match &element.inherit {
                            UiType::DragBox(drag) => {
                                match element.style.position {
                                    Position::Inline(_) if element.parent.is_some() => {
                                        let mut move_vec = match drag.axis {
                                            1 => Vec2::new(1.0, 0.0),
                                            2 => Vec2::new(0.0, 1.0),
                                            3 => Vec2::one(),
                                            0 => Vec2::zero(),
                                            _ => unreachable!()

                                        } * (cursor_pos - self.cursor_pos);

                                        if !drag.on_drag.is_null() {
                                            let fn_ptr = drag.on_drag;
                                            let mut event = DragEvent { move_vec, element };
                                            
                                            fn_ptr.call_vars(&mut event);
                                            move_vec = event.move_vec;
                                        }

                                        let parent = element.parent.as_mut().unwrap();

                                        Rc::make_mut(parent).move_computed(move_vec);
                                    },
                                    _ => {
                                        let mut move_vec = match drag.axis {
                                            1 => Vec2::new(1.0, 0.0),
                                            2 => Vec2::new(0.0, 1.0),
                                            3 => Vec2::one(),
                                            0 => Vec2::zero(),
                                            _ => unreachable!()

                                        } * (cursor_pos - self.cursor_pos);

                                        if !drag.on_drag.is_null() {
                                            let fn_ptr = drag.on_drag;
                                            let mut event = DragEvent { move_vec, element };
                                            
                                            fn_ptr.call_vars(&mut event);
                                            move_vec = event.move_vec;
                                        }

                                        element.computed.pos += move_vec;
                                    }
                                }
                                bol = 3;
                            }
                            _ => ()
                        }
                    },
                    UiEvent::Release => {
                        let mut dirty = false;
                        let mut remove_pressed = false;
                        match &mut element.inherit {
                            UiType::Button(button) => {
                                button.interaction = Interaction::None;
                                dirty = true;
                                bol = 3;
                                let button2 = &button as *const _ as *mut _;
                                if is_in {
                                    #[allow(invalid_reference_casting)]
                                    button.on_press.call(ui, unsafe { &mut *button2 });
                                }
                                remove_pressed = true;
                            },
                            UiType::CheckBox(checkbox) => {
                                checkbox.pressed = false;
                                dirty = true;
                                bol = 3;
                                let checkbox2 = &checkbox as *const _ as *mut _;
                                if is_in {
                                    if checkbox.enabled {
                                        #[allow(invalid_reference_casting)]
                                        checkbox.on_disable.call(ui, unsafe { &mut *checkbox2 });
                                    } else {
                                        #[allow(invalid_reference_casting)]
                                        checkbox.on_enable.call(ui, unsafe { &mut *checkbox2 });
                                    }
                                    checkbox.enabled = !checkbox.enabled;
                                }
                            },
                            UiType::DragBox(dragbox) => {
                                dragbox.interaction = Interaction::None;
                            },
                            _ => ()
                        }
                        if dirty {
                            element.dirty = true;
                        }

                        if remove_pressed {
                            self.pressed = None;
                        }
                    },
                    _ => (),
                }
            }
        }
        //New event
        if bol == 3 { self.dirty = true }
        self.cursor_pos = cursor_pos;
        bol
    }

    use std::rc::Rc;

use iron_oxide::{graphics::formats::RGBA, ui::{Align, CheckBox, DragBox, Inline, Overflow, Padding, Position, Slider, Style, Text, UIUnit::{self, Pixel}, UiElement, UiSize::{self, Size}, UiState, UiType}};

use crate::graphics::VulkanRender;

#[inline(always)]
pub fn build_main(renderer: &mut VulkanRender) -> UiState {
    let style = Style::new(Align::Center, UIUnit::Zero, UIUnit::Zero, Size(UIUnit::RelativeWidth(0.2)), Size(UIUnit::RelativeWidth(0.4)), RGBA::new(0, 0, 0, 150), RGBA::GREEN, 2.0, Pixel(10.0));
    let style2 = Style::inline(UIUnit::Pixel(5.0), RGBA::RED, RGBA::GREEN, 1.0, Pixel(10.0), UiSize::Fill, UiSize::Auto);
    let style25 = Style::inline(UIUnit::Pixel(4.0), RGBA::WHITE, RGBA::GREEN, 1.0, Pixel(3.0), UiSize::Size(UIUnit::Pixel(16.0)), UiSize::Size(UIUnit::Pixel(16.0)));
    let style3 = Style::inline(UIUnit::Pixel(4.0), RGBA::LIGHTGREY, RGBA::GREEN, 1.0, Pixel(3.0), UiSize::Size(UIUnit::Pixel(16.0)), UiSize::Size(UIUnit::Pixel(16.0)));
    let style4 = Style::inline(UIUnit::Pixel(4.0), RGBA::RED, RGBA::GREEN, 1.0, Pixel(3.0), UiSize::Size(UIUnit::Pixel(16.0)), UiSize::Size(UIUnit::Pixel(16.0)));
    let style5 = Style::new(Align::Center, UIUnit::Zero, UIUnit::Zero, Size(UIUnit::Pixel(60.0)), Size(UIUnit::Pixel(60.0)), RGBA::new(0, 0, 0, 150), RGBA::WHITE, 1.0, UIUnit::Relative(0.5));



    let slider_style = Style {
        position: Position::Inline(Inline { margin: [UIUnit::Pixel(4.0); 4], overflow: Overflow::clip() }),
        width: UiSize::Fill,
        height: UiSize::Size(UIUnit::Pixel(16.0)),
        color: RGBA::DARKGREY,
        border_color: RGBA::WHITE,
        border: [1.0; 4],
        corner: [UIUnit::RelativeHeight(0.5); 4],
        padding: Padding::new(1.0),
    };

    let statistic = DragBox::new(style5);

    let mut checkbox = CheckBox::checkbox(style3, style4);
    checkbox.on_enable(renderer, |renderer: &mut VulkanRender, _ui_state| {
        renderer.renderer = !renderer.renderer;
    });
    checkbox.on_disable(renderer, |renderer: &mut VulkanRender, _ui_state| {
        renderer.renderer = !renderer.renderer;
    });

    let slider = Slider::new(slider_style, 0.0, 100.0, 0.0, RGBA::BLUE, RGBA::GREEN);
    let slider_text = Text::new(slider_style, 0, "Value: 0", 3);

    let dragbox = UiElement::new(style, vec![
        Rc::new(DragBox::new(style2.clone())),
        UiElement::new(Style::label(RGBA::new(0, 0, 0, 100), UIUnit::Pixel(10.0), UiSize::Fill, UIUnit::Pixel(5.0)), vec![
                Rc::new(UiElement::extend(style25, Vec::with_capacity(0), UiType::CheckBox(checkbox))), 
                Rc::new(Text::new(Style::text(RGBA::new(255, 255, 255, 100), UIUnit::Zero, UIUnit::Pixel(16.0)), 0, "home_storage_vulkan", 2))
            ],
        ),
        UiElement::new(Style::label(RGBA::new(0, 0, 0, 100), UIUnit::Pixel(10.0), UiSize::Fill, UIUnit::Pixel(5.0)), vec![
            slider.clone(),
            Rc::new(slider_text),
            slider
        ])
    ]);

    let state = UiState::create(true);
    state
}
