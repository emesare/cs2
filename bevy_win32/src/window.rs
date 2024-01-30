use std::{
    ffi::{CStr, CString},
    hash::Hash,
};

use bevy_ecs::prelude::*;
use bevy_math::Vec2;
use bevy_utils::HashMap;
use bevy_window::{
    prelude::*, CompositeAlphaMode, Cursor, InternalWindowState, PresentMode, WindowLevel,
    WindowMode, WindowResolution, WindowTheme,
};
use windows::Win32::{
    Foundation::*,
    UI::WindowsAndMessaging::{GetWindowRect, GetWindowTextLengthW, GetWindowTextW},
};

// TODO: Propagate window info to a bevy `Window` (so we can get window position, size, title, etc...).

// TODO: Move Win32WindowHandle to RawWindowHandle?
#[derive(Component, Eq, Clone, Copy, PartialEq, Debug)]
pub struct Win32WindowHandle(HWND);

impl Win32WindowHandle {
    pub fn handle(&self) -> HWND {
        self.0
    }

    // TODO
    pub fn cursor(&self) -> Cursor {
        Cursor::default()
    }

    // TODO
    pub fn present_mode(&self) -> PresentMode {
        PresentMode::Immediate
    }

    // TODO
    pub fn mode(&self) -> WindowMode {
        WindowMode::Windowed
    }

    // TODO: https://learn.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-windowinfo
    pub fn position(&self) -> WindowPosition {
        let mut rect = RECT::default();
        unsafe {
            // TODO: Error handling.
            GetWindowRect(self.0, &mut rect).unwrap();
        }
        WindowPosition::At((rect.left, rect.top).into())
    }

    // TODO: https://learn.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-windowinfo
    // TODO: Add scaling: `with_scale_factor_override`
    pub fn resolution(&self) -> WindowResolution {
        let mut rect = RECT::default();
        unsafe {
            // TODO: Error handling.
            GetWindowRect(self.0, &mut rect).unwrap();
        }
        WindowResolution::new(
            (rect.right - rect.left) as f32,
            (rect.bottom - rect.top) as f32,
        )
    }

    pub fn title(&self) -> String {
        // TODO: Error handling.
        let title_len = unsafe { GetWindowTextLengthW(self.0) };
        if title_len == 0 {
            panic!("Window title length no good!")
        }
        let mut title: Vec<u16> = vec![0; title_len as usize];
        let written = unsafe { GetWindowTextW(self.0, &mut title) };
        String::from_utf16(title[0..(written as usize)].as_ref()).unwrap()
    }

    // TODO
    pub fn composite_alpha_mode(&self) -> CompositeAlphaMode {
        CompositeAlphaMode::PreMultiplied
    }

    // TODO
    pub fn resize_constraints(&self) -> WindowResizeConstraints {
        WindowResizeConstraints::default()
    }

    // TODO
    pub fn resizable(&self) -> bool {
        false
    }

    // TODO
    pub fn decorations(&self) -> bool {
        false
    }

    // TODO
    pub fn transparent(&self) -> bool {
        false
    }

    // TODO: https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getguithreadinfo?redirectedfrom=MSDN
    pub fn focused(&self) -> bool {
        true
    }

    // TODO
    pub fn window_level(&self) -> WindowLevel {
        WindowLevel::Normal
    }

    pub fn canvas(&self) -> Option<String> {
        None
    }

    pub fn fit_canvas_to_parent(&self) -> bool {
        false
    }

    pub fn prevent_default_event_handling(&self) -> bool {
        false
    }

    // TODO
    pub fn internal(&self) -> InternalWindowState {
        InternalWindowState::default()
    }

    pub fn ime_enabled(&self) -> bool {
        false
    }

    pub fn ime_position(&self) -> Vec2 {
        Vec2::default()
    }

    pub fn window_theme(&self) -> Option<WindowTheme> {
        None
    }
}

impl Hash for Win32WindowHandle {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0 .0.hash(state)
    }
}

impl From<HWND> for Win32WindowHandle {
    fn from(value: HWND) -> Self {
        Self(value)
    }
}

impl From<Win32WindowHandle> for bevy_window::Window {
    fn from(handle: Win32WindowHandle) -> Self {
        Self {
            cursor: handle.cursor(),
            present_mode: handle.present_mode(),
            mode: handle.mode(),
            position: handle.position(),
            resolution: handle.resolution(),
            title: handle.title(),
            composite_alpha_mode: handle.composite_alpha_mode(),
            resize_constraints: handle.resize_constraints(),
            resizable: handle.resizable(),
            decorations: handle.decorations(),
            transparent: handle.transparent(),
            focused: handle.focused(),
            window_level: handle.window_level(),
            canvas: handle.canvas(),
            fit_canvas_to_parent: handle.fit_canvas_to_parent(),
            prevent_default_event_handling: handle.prevent_default_event_handling(),
            internal: handle.internal(),
            ime_enabled: handle.ime_enabled(),
            ime_position: handle.ime_position(),
            window_theme: handle.window_theme(),
        }
    }
}

/// A resource mapping window entities to their `win32`-backend states.
#[derive(Resource, Debug, Default)]
pub struct Win32Windows {
    /// Stores [`winit`] windows by window identifier.
    pub windows: Vec<Win32WindowHandle>,
    /// Maps entities to `win32` window identifiers.
    pub entity_to_win32: HashMap<Entity, Win32WindowHandle>,
    /// Maps `win32` window identifiers to entities.
    pub win32_to_entity: HashMap<Win32WindowHandle, Entity>,
    // Many `win32` window functions (e.g. `set_window_icon`) can only be called on the main thread.
    // If they're called on other threads, the program might hang. This marker indicates that this
    // type is not thread-safe and will be `!Send` and `!Sync`.
    //_not_send_sync: core::marker::PhantomData<*const ()>, // TODO: Check this?
}

impl Win32Windows {
    pub fn associate_handle_with_entity(
        &mut self,
        entity: Entity,
        window_handle: Win32WindowHandle,
    ) {
        self.entity_to_win32.insert(entity, window_handle);
        self.win32_to_entity.insert(window_handle, entity);
        self.windows.push(window_handle);
    }

    /// Get the win32 window handle that is associated with our entity.
    pub fn get_window(&self, entity: Entity) -> Option<&Win32WindowHandle> {
        self.entity_to_win32.get(&entity).and_then(|win32_handle| {
            self.windows.get(
                self.windows
                    .iter()
                    .position(|hwnd| *hwnd == *win32_handle)?,
            )
        })
    }

    /// Get the entity associated with the win32 window handle.
    ///
    /// This is mostly just an intermediary step between us and win32.
    pub fn get_window_entity(&self, window_handle: Win32WindowHandle) -> Option<Entity> {
        self.win32_to_entity.get(&window_handle).cloned()
    }

    pub fn remove_window(&mut self, entity: Entity) -> Option<Win32WindowHandle> {
        let handle = self.entity_to_win32.remove(&entity)?;
        // Don't remove from `win32_to_window_id` so we know the window used to exist.
        Some(
            self.windows
                .swap_remove(self.windows.iter().position(|hwnd| *hwnd == handle)?),
        )
    }
}
