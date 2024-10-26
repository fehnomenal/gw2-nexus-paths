use std::{mem::MaybeUninit, path::PathBuf};

use debounce::EventDebouncer;
use paths_core::{
    loadable::BackgroundLoadable,
    markers::{ActiveMarkerCategories, MarkerCategoryTree},
    settings::Settings,
    ui::UiState,
};

use crate::{input_manager::InputManager, renderer::Renderer};

use super::AddonUiActions;

pub static mut ACTIVE_MARKER_CATEGORIES: MaybeUninit<ActiveMarkerCategories> =
    MaybeUninit::uninit();

pub static mut API: MaybeUninit<api::AddonApiWrapper> = MaybeUninit::uninit();

pub static mut MARKER_CATEGORY_TREE: MaybeUninit<BackgroundLoadable<MarkerCategoryTree>> =
    MaybeUninit::new(BackgroundLoadable::Loading);

pub static mut MUMBLE_DATA: MaybeUninit<&api::Mumble_Data> = MaybeUninit::uninit();

pub static mut MUMBLE_IDENTITY: Option<&api::Mumble_Identity> = None;

pub static mut NEXUS_LINK_DATA: MaybeUninit<&api::NexusLinkData> = MaybeUninit::uninit();

pub static mut RENDERER: MaybeUninit<Renderer> = MaybeUninit::uninit();

pub static mut SETTINGS_FILE_PATH: MaybeUninit<PathBuf> = MaybeUninit::uninit();

pub static mut SETTINGS_SAVER: MaybeUninit<EventDebouncer<()>> = MaybeUninit::uninit();

pub static mut SETTINGS: MaybeUninit<Settings> = MaybeUninit::uninit();

pub static mut UI_INPUT_MANAGER: MaybeUninit<InputManager> = MaybeUninit::uninit();

pub static mut UI_STATE: MaybeUninit<UiState<AddonUiActions>> = MaybeUninit::uninit();
