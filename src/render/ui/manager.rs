use egui::{Context, Event, Modifiers, PointerButton, Pos2, Vec2};
use windows::Win32::UI::WindowsAndMessaging;

pub struct InputManager {
    pub egui_context: Context,
    events: Vec<Event>,
}

impl InputManager {
    pub fn new(egui_context: &Context) -> Self {
        Self {
            egui_context: egui_context.clone(),
            events: vec![],
        }
    }

    pub fn handle_wnd_proc(&mut self, msg: u32, w_param: u32, l_param: i32) -> bool {
        match msg {
            WindowsAndMessaging::WM_MOUSEMOVE => {
                let (x, y) = get_position(l_param);

                self.events.push(Event::PointerMoved(Pos2::new(x, y)));

                self.egui_context.wants_pointer_input()
            }

            WindowsAndMessaging::WM_LBUTTONDOWN
            | WindowsAndMessaging::WM_LBUTTONUP
            | WindowsAndMessaging::WM_RBUTTONDOWN
            | WindowsAndMessaging::WM_RBUTTONUP => {
                let (x, y) = get_position(l_param);

                let button = match msg {
                    WindowsAndMessaging::WM_LBUTTONDOWN | WindowsAndMessaging::WM_LBUTTONUP => {
                        PointerButton::Primary
                    }
                    WindowsAndMessaging::WM_RBUTTONDOWN | WindowsAndMessaging::WM_RBUTTONUP => {
                        PointerButton::Secondary
                    }
                    _ => unreachable!(),
                };

                let pressed = match msg {
                    WindowsAndMessaging::WM_LBUTTONDOWN | WindowsAndMessaging::WM_RBUTTONDOWN => {
                        true
                    }
                    WindowsAndMessaging::WM_LBUTTONUP | WindowsAndMessaging::WM_RBUTTONUP => false,
                    _ => unreachable!(),
                };

                self.events.push(Event::PointerButton {
                    pos: Pos2::new(x, y),
                    button,
                    pressed,
                    modifiers: Modifiers::NONE,
                });

                self.egui_context.wants_pointer_input()
            }

            WindowsAndMessaging::WM_MOUSEWHEEL | WindowsAndMessaging::WM_MOUSEHWHEEL => {
                let delta = (w_param >> 16) as i16 as f32 / WindowsAndMessaging::WHEEL_DELTA as f32;

                let (x, y) = match msg {
                    WindowsAndMessaging::WM_MOUSEWHEEL => (0.0, delta),
                    WindowsAndMessaging::WM_MOUSEHWHEEL => (delta, 0.0),
                    _ => unreachable!(),
                };

                self.events.push(Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Line,
                    delta: Vec2::new(x, y),
                    modifiers: Modifiers::NONE,
                });

                self.egui_context.wants_pointer_input()
            }
            _ => false,
        }
    }

    pub fn get_events(&mut self) -> Vec<Event> {
        std::mem::take(&mut self.events)
    }
}

fn get_position(l_param: i32) -> (f32, f32) {
    let x = (l_param & 0xFFFF) as i16 as f32;
    let y = (l_param >> 16 & 0xFFFF) as i16 as f32;

    (x, y)
}
