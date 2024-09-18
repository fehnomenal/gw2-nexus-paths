use std::mem::MaybeUninit;

use paths_renderer::{RenderConfig, Renderer};

use crate::input_manager::InputManager;

pub static mut RENDERER: MaybeUninit<Renderer<'static>> = MaybeUninit::uninit();
pub static mut RENDER_CONFIG: MaybeUninit<RenderConfig> = MaybeUninit::uninit();
pub static mut UI_INPUT_MANAGER: MaybeUninit<InputManager> = MaybeUninit::uninit();
pub static mut IS_UI_VISIBLE: bool = false;
