// TODO: Propagate window info to a bevy `Window` (so we can get window position, size, title, etc...).

use bevy_ecs::prelude::*;
use bevy_window::{prelude::*, RawHandleWrapper, WindowCreated};
use windows::Win32::Foundation::HWND;

use crate::window::{Win32WindowHandle, Win32Windows};

// TODO: We dont need this now...
pub fn create_windows(
    mut commands: Commands,
    mut windows: Query<(Entity, &Window)>,
    mut wc_writer: EventWriter<WindowCreated>,
    mut win32_windows: ResMut<Win32Windows>,
) {
    for (entity, window) in &windows {
        // TODO: Wait... we already have our handle...
        if win32_windows.get_window(entity).is_some() {
            continue;
        }

        log::debug!(
            "creating new window {:?} ({:?})",
            window.title.as_str(),
            entity
        );

        // Insert RawHandleWrapper
        // commands.entity(entity).insert().insert(CachedWindow {
        //     window: window.clone(),
        // });
        // win32_windows.associate_handle_with_entity(entity, window_handle)
    }
}
