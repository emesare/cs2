use bevy_app::prelude::*;
use bevy_ecs::{prelude::*, system::SystemParam};
use bevy_input::{
    keyboard::KeyboardInput,
    mouse::{MouseButtonInput, MouseMotion, MouseScrollUnit, MouseWheel},
    prelude::*,
    ButtonState,
};

use bevy_window::{prelude::*, PrimaryWindow};
use window::{Win32WindowHandle, Win32Windows};
use windows::Win32::{Foundation::*, UI::WindowsAndMessaging::*};

mod converters;
mod system;
pub mod window;

#[derive(Debug, thiserror::Error)]
pub enum BevyWin32Error {
    #[error("Window {0:?} not associated to an entity")]
    UnassociatedWindow(HWND),
}

#[derive(SystemParam)]
struct WindowAndInputEventWriters<'w> {
    keyboard_input: EventWriter<'w, KeyboardInput>,
    character_input: EventWriter<'w, ReceivedCharacter>,
    mouse_button_input: EventWriter<'w, MouseButtonInput>,
    mouse_wheel_input: EventWriter<'w, MouseWheel>,
    // cursor_moved: EventWriter<'w, CursorMoved>,
    // cursor_entered: EventWriter<'w, CursorEntered>,
    // cursor_left: EventWriter<'w, CursorLeft>,
    mouse_motion: EventWriter<'w, MouseMotion>,
}

#[derive(Debug, Event)]
pub struct WinMessageEvent {
    pub window: Entity,
    pub msg: u32,
    pub wparam: WPARAM,
    pub lparam: LPARAM,
}

fn process_message(
    mut event_writers: WindowAndInputEventWriters,
    mut wm_event: EventReader<WinMessageEvent>,
) {
    for event in wm_event.iter() {
        let (window, msg, wparam, lparam) = (event.window, event.msg, event.wparam, event.lparam);
        // TODO: IME Support & window support & some other stuff.
        match msg {
            WM_MOUSEMOVE => event_writers.mouse_motion.send(MouseMotion {
                // TODO: This is wrong, we get delta by taking new - old
                delta: converters::mouse_pos_from_lparam(lparam),
            }),
            WM_LBUTTONDOWN | WM_LBUTTONDBLCLK => {
                event_writers.mouse_button_input.send(MouseButtonInput {
                    button: MouseButton::Left,
                    state: ButtonState::Pressed,
                    window,
                })
            }
            WM_LBUTTONUP => event_writers.mouse_button_input.send(MouseButtonInput {
                button: MouseButton::Left,
                state: ButtonState::Released,
                window,
            }),
            WM_RBUTTONDOWN | WM_RBUTTONDBLCLK => {
                event_writers.mouse_button_input.send(MouseButtonInput {
                    button: MouseButton::Right,
                    state: ButtonState::Pressed,
                    window,
                })
            }
            WM_RBUTTONUP => event_writers.mouse_button_input.send(MouseButtonInput {
                button: MouseButton::Right,
                state: ButtonState::Released,
                window,
            }),
            WM_MBUTTONDOWN | WM_MBUTTONDBLCLK => {
                event_writers.mouse_button_input.send(MouseButtonInput {
                    button: MouseButton::Middle,
                    state: ButtonState::Pressed,
                    window,
                })
            }
            WM_MBUTTONUP => event_writers.mouse_button_input.send(MouseButtonInput {
                button: MouseButton::Middle,
                state: ButtonState::Released,
                window,
            }),
            WM_XBUTTONDOWN | WM_XBUTTONDBLCLK => {
                event_writers.mouse_button_input.send(MouseButtonInput {
                    button: MouseButton::Other(wparam.0 as u16), // TODO: Make sure wparam
                    state: ButtonState::Pressed,
                    window,
                })
            }
            WM_XBUTTONUP => event_writers.mouse_button_input.send(MouseButtonInput {
                button: MouseButton::Other(wparam.0 as u16), // TODO: Make sure wparam
                state: ButtonState::Pressed,
                window,
            }),
            WM_CHAR => {
                if let Some(char) = char::from_u32(wparam.0 as _) {
                    // TODO: Should we check this?
                    if !char.is_control() {
                        event_writers
                            .character_input
                            .send(ReceivedCharacter { window, char })
                    }
                }
            }
            WM_MOUSEWHEEL => event_writers.mouse_wheel_input.send(MouseWheel {
                unit: MouseScrollUnit::Pixel,
                x: 0.0,
                y: converters::mouse_wheel_delta_from_wparam(wparam),
                window,
            }),
            WM_MOUSEHWHEEL => event_writers.mouse_wheel_input.send(MouseWheel {
                unit: MouseScrollUnit::Pixel,
                x: converters::mouse_wheel_delta_from_wparam(wparam),
                y: 0.0,
                window,
            }),
            WM_KEYDOWN | WM_SYSKEYDOWN => event_writers.keyboard_input.send(KeyboardInput {
                scan_code: converters::scancode_from_lparam(lparam).0,
                key_code: converters::keycode_from_wparam(wparam),
                state: ButtonState::Pressed,
                window,
            }),
            WM_KEYUP | WM_SYSKEYUP => event_writers.keyboard_input.send(KeyboardInput {
                scan_code: converters::scancode_from_lparam(lparam).0,
                key_code: converters::keycode_from_wparam(wparam),
                state: ButtonState::Released,
                window,
            }),
            _ => (),
        };
    }
}

#[derive(Event)]
pub struct AddWindowEvent {
    pub handle: Win32WindowHandle,
    pub is_primary: bool,
}

fn add_windows(
    mut commands: Commands,
    mut aw_event: EventReader<AddWindowEvent>,
    mut windows: ResMut<Win32Windows>,
) {
    for ev in aw_event.iter() {
        // TODO: Actually get window
        // let window = Window::from(ev.handle);
        let window = Window::default();
        let window_title = window.title.clone();
        log::info!(
            "adding window ({}) with handle ({:?})",
            window_title,
            ev.handle
        );
        let mut entity_cmds = commands.spawn(window);
        windows.associate_handle_with_entity(entity_cmds.id(), ev.handle);
        if ev.is_primary {
            entity_cmds.insert(PrimaryWindow);
        }
    }
}

pub struct Win32Plugin;

impl Plugin for Win32Plugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Win32Windows>()
            .add_event::<AddWindowEvent>()
            .add_event::<WinMessageEvent>()
            .add_systems(Update, (add_windows, process_message));
    }
}
