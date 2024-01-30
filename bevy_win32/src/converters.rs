use bevy_input::keyboard::{KeyCode, ScanCode};
use bevy_math::Vec2;
use windows::Win32::{
    Foundation::{LPARAM, WPARAM},
    UI::{Input::KeyboardAndMouse::*, WindowsAndMessaging::WHEEL_DELTA},
};

pub fn mouse_pos_from_lparam(lparam: LPARAM) -> Vec2 {
    let x = (lparam.0 & 0xFFFF) as i16 as f32;
    let y = (lparam.0 >> 16 & 0xFFFF) as i16 as f32;
    Vec2::new(x, y)
}

// TODO: Read lower (wparam cast to i16) and see if that is ever more than 0 its the X coordinate.
pub fn mouse_wheel_delta_from_wparam(wparam: WPARAM) -> f32 {
    let _x_coord = wparam.0 as i16;
    let y_coord = (wparam.0 >> 16) as i16;
    y_coord as f32 * 10. / WHEEL_DELTA as f32 // TODO: Why do we multiply by 10.?
}

pub fn scancode_from_lparam(lparam: LPARAM) -> ScanCode {
    let scancode = (lparam.0 >> 16) as u8;
    ScanCode(scancode.into())
}

pub fn keycode_from_wparam(wparam: WPARAM) -> Option<KeyCode> {
    // TODO: Dont listen to the comment below
    // TODO: Handle modifiers in here, i.e. params should include modifiers and then use it in match to get key.
    match VIRTUAL_KEY(wparam.0 as u16) {
        VK_LBUTTON | VK_RBUTTON | VK_CANCEL | VK_MBUTTON | VK_XBUTTON1 | VK_XBUTTON2 | VK_CLEAR
        | VK_MENU | VK_HANGUL | VK_IME_ON | VK_JUNJA | VK_FINAL | VK_HANJA | VK_IME_OFF
        | VK_ACCEPT | VK_MODECHANGE | VK_SELECT | VK_PRINT | VK_EXECUTE | VK_HELP
        | VK_SEPARATOR | VK_LMENU | VK_RMENU | VK_LAUNCH_APP1 | VK_LAUNCH_APP2 | VK_OEM_8
        | VK_PROCESSKEY | VK_PACKET | VK_ATTN | VK_CRSEL | VK_EXSEL | VK_EREOF | VK_PLAY
        | VK_ZOOM | VK_NONAME | VK_PA1 | VK_OEM_CLEAR => None,
        VK_BACK => Some(KeyCode::Back),
        VK_TAB => Some(KeyCode::Tab),
        VK_RETURN => Some(KeyCode::Return),
        VK_SHIFT => Some(KeyCode::ShiftLeft), // TODO: How do we differentiate?
        VK_CONTROL => Some(KeyCode::ControlLeft), // TODO: How do we differentiate?
        VK_PAUSE => Some(KeyCode::Pause),
        VK_CAPITAL => Some(KeyCode::Capital),
        VK_KANA => Some(KeyCode::Kana),
        VK_KANJI => Some(KeyCode::Kanji),
        VK_ESCAPE => Some(KeyCode::Escape),
        VK_CONVERT => Some(KeyCode::Convert),
        VK_NONCONVERT => Some(KeyCode::NoConvert),
        VK_SPACE => Some(KeyCode::Space),
        VK_PRIOR => Some(KeyCode::PageUp),
        VK_NEXT => Some(KeyCode::PageDown),
        VK_END => Some(KeyCode::End),
        VK_HOME => Some(KeyCode::Home),
        VK_LEFT => Some(KeyCode::Left),
        VK_UP => Some(KeyCode::Up),
        VK_RIGHT => Some(KeyCode::Right),
        VK_DOWN => Some(KeyCode::Down),
        VK_SNAPSHOT => Some(KeyCode::Snapshot),
        VK_INSERT => Some(KeyCode::Insert),
        VK_DELETE => Some(KeyCode::Delete),
        VK_0 => Some(KeyCode::Key0),
        VK_1 => Some(KeyCode::Key1),
        VK_2 => Some(KeyCode::Key2),
        VK_3 => Some(KeyCode::Key3),
        VK_4 => Some(KeyCode::Key4),
        VK_5 => Some(KeyCode::Key5),
        VK_6 => Some(KeyCode::Key6),
        VK_7 => Some(KeyCode::Key7),
        VK_8 => Some(KeyCode::Key8),
        VK_9 => Some(KeyCode::Key9),
        VK_A => Some(KeyCode::A),
        VK_B => Some(KeyCode::B),
        VK_C => Some(KeyCode::C),
        VK_D => Some(KeyCode::D),
        VK_E => Some(KeyCode::E),
        VK_F => Some(KeyCode::F),
        VK_G => Some(KeyCode::G),
        VK_H => Some(KeyCode::H),
        VK_I => Some(KeyCode::I),
        VK_J => Some(KeyCode::J),
        VK_K => Some(KeyCode::K),
        VK_L => Some(KeyCode::L),
        VK_M => Some(KeyCode::M),
        VK_N => Some(KeyCode::N),
        VK_O => Some(KeyCode::O),
        VK_P => Some(KeyCode::P),
        VK_Q => Some(KeyCode::Q),
        VK_R => Some(KeyCode::R),
        VK_S => Some(KeyCode::S),
        VK_T => Some(KeyCode::T),
        VK_U => Some(KeyCode::U),
        VK_V => Some(KeyCode::V),
        VK_W => Some(KeyCode::W),
        VK_X => Some(KeyCode::X),
        VK_Y => Some(KeyCode::Y),
        VK_Z => Some(KeyCode::Z),
        VK_LWIN => Some(KeyCode::SuperLeft),
        VK_RWIN => Some(KeyCode::SuperRight),
        VK_APPS => Some(KeyCode::Apps),
        VK_SLEEP => Some(KeyCode::Sleep),
        VK_NUMPAD0 => Some(KeyCode::Numpad0),
        VK_NUMPAD1 => Some(KeyCode::Numpad1),
        VK_NUMPAD2 => Some(KeyCode::Numpad2),
        VK_NUMPAD3 => Some(KeyCode::Numpad3),
        VK_NUMPAD4 => Some(KeyCode::Numpad4),
        VK_NUMPAD5 => Some(KeyCode::Numpad5),
        VK_NUMPAD6 => Some(KeyCode::Numpad6),
        VK_NUMPAD7 => Some(KeyCode::Numpad7),
        VK_NUMPAD8 => Some(KeyCode::Numpad8),
        VK_NUMPAD9 => Some(KeyCode::Numpad9),
        VK_MULTIPLY => Some(KeyCode::NumpadMultiply),
        VK_ADD => Some(KeyCode::NumpadAdd),
        VK_SUBTRACT => Some(KeyCode::NumpadSubtract),
        VK_DECIMAL => Some(KeyCode::NumpadDecimal),
        VK_DIVIDE => Some(KeyCode::NumpadDivide),
        VK_F1 => Some(KeyCode::F1),
        VK_F2 => Some(KeyCode::F2),
        VK_F3 => Some(KeyCode::F3),
        VK_F4 => Some(KeyCode::F4),
        VK_F5 => Some(KeyCode::F5),
        VK_F6 => Some(KeyCode::F6),
        VK_F7 => Some(KeyCode::F7),
        VK_F8 => Some(KeyCode::F8),
        VK_F9 => Some(KeyCode::F9),
        VK_F10 => Some(KeyCode::F10),
        VK_F11 => Some(KeyCode::F11),
        VK_F12 => Some(KeyCode::F12),
        VK_F13 => Some(KeyCode::F13),
        VK_F14 => Some(KeyCode::F14),
        VK_F15 => Some(KeyCode::F15),
        VK_F16 => Some(KeyCode::F16),
        VK_F17 => Some(KeyCode::F17),
        VK_F18 => Some(KeyCode::F18),
        VK_F19 => Some(KeyCode::F19),
        VK_F20 => Some(KeyCode::F20),
        VK_F21 => Some(KeyCode::F21),
        VK_F22 => Some(KeyCode::F22),
        VK_F23 => Some(KeyCode::F23),
        VK_F24 => Some(KeyCode::F24),
        VK_NUMLOCK => Some(KeyCode::Numlock),
        VK_SCROLL => Some(KeyCode::Scroll),
        VK_LSHIFT => Some(KeyCode::ShiftLeft),
        VK_RSHIFT => Some(KeyCode::ShiftRight),
        VK_LCONTROL => Some(KeyCode::ControlLeft),
        VK_RCONTROL => Some(KeyCode::ControlRight),
        VK_BROWSER_BACK => Some(KeyCode::WebBack),
        VK_BROWSER_FORWARD => Some(KeyCode::WebForward),
        VK_BROWSER_REFRESH => Some(KeyCode::WebRefresh),
        VK_BROWSER_STOP => Some(KeyCode::WebStop),
        VK_BROWSER_SEARCH => Some(KeyCode::WebSearch),
        VK_BROWSER_FAVORITES => Some(KeyCode::WebFavorites),
        VK_BROWSER_HOME => Some(KeyCode::Home),
        VK_VOLUME_MUTE => Some(KeyCode::Mute),
        VK_VOLUME_DOWN => Some(KeyCode::VolumeDown),
        VK_VOLUME_UP => Some(KeyCode::VolumeUp),
        VK_MEDIA_NEXT_TRACK => Some(KeyCode::NextTrack),
        VK_MEDIA_PREV_TRACK => Some(KeyCode::PrevTrack),
        VK_MEDIA_STOP => Some(KeyCode::MediaStop),
        VK_MEDIA_PLAY_PAUSE => Some(KeyCode::PlayPause),
        VK_LAUNCH_MAIL => Some(KeyCode::Mail),
        VK_LAUNCH_MEDIA_SELECT => Some(KeyCode::MediaSelect),
        VK_OEM_1 => Some(KeyCode::Semicolon),
        VK_OEM_PLUS => Some(KeyCode::Plus),
        VK_OEM_COMMA => Some(KeyCode::Comma),
        VK_OEM_MINUS => Some(KeyCode::Minus),
        VK_OEM_PERIOD => Some(KeyCode::Period),
        VK_OEM_2 => Some(KeyCode::Slash),
        VK_OEM_3 => Some(KeyCode::Grave),
        VK_OEM_4 => Some(KeyCode::BracketLeft),
        VK_OEM_5 => Some(KeyCode::Backslash),
        VK_OEM_6 => Some(KeyCode::BracketRight),
        VK_OEM_7 => Some(KeyCode::Apostrophe),
        VK_OEM_102 => Some(KeyCode::Oem102),
        _ => None,
    }
}
