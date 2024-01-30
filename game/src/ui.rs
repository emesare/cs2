use bevy_app::prelude::*;
use bevy_ecs::{prelude::*, system::SystemParam};
use bevy_input::{
    keyboard::{KeyCode, KeyboardInput},
    mouse::{MouseButton, MouseButtonInput, MouseMotion, MouseScrollUnit, MouseWheel},
};
use egui::RawInput;
use epaint::Shadow;

use crate::render::{Present, RenderEvent, RenderTargetHandle, RenderTargets};

// TODO: Should we make UiContext a resource again? then we can just attach the mouse pos directly to it...

// TODO: Attach this to a window.
// TODO: Add a "primary" ui context.
#[derive(Clone, Resource)]
pub struct UiContext {
    render_target_handle: RenderTargetHandle,
    ctx: egui::Context,
    mouse_pos: egui::Pos2,
}

impl UiContext {
    pub fn new(render_target_handle: RenderTargetHandle) -> Self {
        let mut ui_ctx = egui::Context::default();
        Self::build_ctx(&mut ui_ctx);
        Self {
            render_target_handle,
            ctx: ui_ctx,
            mouse_pos: Default::default(),
        }
    }

    pub fn build_ctx(ctx: &mut egui::Context) {
        // TODO: Change styling etc...
        ctx.style_mut(|style| {
            style.visuals.window_shadow = Shadow::NONE;
        });
    }

    /// Borrows the underlying Egui context mutably.
    ///
    /// When the context is queried with `&mut EguiContext`, the Bevy scheduler is able to make
    /// sure that the context isn't accessed concurrently and can perform other useful work
    /// instead of busy-waiting.
    #[must_use]
    pub fn get_mut(&mut self) -> &mut egui::Context {
        // TODO: Make rwlock or not needed?
        &mut self.ctx
    }
}

impl FromWorld for UiContext {
    fn from_world(world: &mut World) -> Self {
        let mut render_targets = world
            .get_resource_mut::<RenderTargets>()
            .expect("RenderTargets should be available");

        Self::new(render_targets.as_mut().create_uninitialized_render_target())
    }
}

// TODO: Subscribe to all bevy input.

// TODO: Lock input to UI... maybe reguister this with some function that will prevent messages from going to game.

#[derive(SystemParam)]
pub struct UiEvents<'w, 's> {
    kb_evr: EventReader<'w, 's, KeyboardInput>,
    mm_evr: EventReader<'w, 's, MouseMotion>,
    mb_evr: EventReader<'w, 's, MouseButtonInput>,
    mw_evr: EventReader<'w, 's, MouseWheel>,
}

pub fn ui_begin_frame(mut ui_ctx: ResMut<UiContext>, mut ui_events: UiEvents) {
    let mut raw_input = RawInput::default();

    // TODO: Modifiers
    // TODO: Screen size
    // TODO: PPI

    for mm_ev in ui_events.mm_evr.iter() {
        // TODO: Right now we can assume that delta is actually pos, but once we fix that we need to store the mouse position as a component.
        ui_ctx.mouse_pos = epaint::Pos2 {
            x: mm_ev.delta.x,
            y: mm_ev.delta.y,
        };

        raw_input
            .events
            .push(egui::Event::PointerMoved(ui_ctx.mouse_pos));
    }

    for mb_ev in ui_events.mb_evr.iter() {
        if let Some(button) = ui_pointer_button_from_mouse_button(mb_ev.button) {
            raw_input.events.push(egui::Event::PointerButton {
                pos: ui_ctx.mouse_pos,
                button: button,
                pressed: mb_ev.state.is_pressed(),
                modifiers: Default::default(),
            });
        }
    }

    for mw_ev in ui_events.mw_evr.iter() {
        raw_input.events.push(egui::Event::MouseWheel {
            unit: match mw_ev.unit {
                MouseScrollUnit::Line => egui::MouseWheelUnit::Line,
                MouseScrollUnit::Pixel => egui::MouseWheelUnit::Point,
            },
            delta: (mw_ev.x, mw_ev.y).into(),
            modifiers: Default::default(),
        });

        // TODO: Handle fucking horizontal.

        raw_input
            .events
            .push(egui::Event::Scroll((mw_ev.x, mw_ev.y).into()));
    }

    for kb_ev in ui_events.kb_evr.iter() {
        if let Some(key_code) = kb_ev.key_code {
            // TODO: Check if modifier, if so add it to the modifiers

            if let Some(key) = ui_key_from_key_code(key_code) {
                raw_input.events.push(egui::Event::Key {
                    key,
                    pressed: kb_ev.state.is_pressed(),
                    repeat: false,
                    modifiers: Default::default(),
                });
            }
        }
    }

    ui_ctx.get_mut().begin_frame(raw_input.clone())
}

pub fn ui_end_frame(mut ev_render: EventWriter<RenderEvent>, mut ui_ctx: ResMut<UiContext>) {
    let ctx = ui_ctx.get_mut();
    let out = ctx.end_frame();
    // Use the egui contexts own tesselation, so that the ui's registered fonts exist.
    let primitives = ctx.tessellate(out.shapes);
    ev_render.send(RenderEvent {
        render_target_handle: ui_ctx.render_target_handle,
        primitives,
        textures_delta: out.textures_delta,
    });
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub struct UiUpdate;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UiContext>()
            .add_systems(
                Present,
                (ui_begin_frame, ui_end_frame.after(ui_begin_frame)),
            )
            .configure_set(Present, UiUpdate.after(ui_begin_frame).before(ui_end_frame));
    }
}

pub fn ui_pointer_button_from_mouse_button(
    mouse_button: MouseButton,
) -> Option<egui::PointerButton> {
    match mouse_button {
        bevy_input::mouse::MouseButton::Left => Some(egui::PointerButton::Primary),
        bevy_input::mouse::MouseButton::Right => Some(egui::PointerButton::Secondary),
        bevy_input::mouse::MouseButton::Middle => Some(egui::PointerButton::Middle),
        bevy_input::mouse::MouseButton::Other(id) => todo!("support extra"),
    }
}

pub fn ui_key_from_key_code(keycode: KeyCode) -> Option<egui::Key> {
    match keycode {
        KeyCode::Key1 => Some(egui::Key::Num1),
        KeyCode::Key2 => Some(egui::Key::Num2),
        KeyCode::Key3 => Some(egui::Key::Num3),
        KeyCode::Key4 => Some(egui::Key::Num4),
        KeyCode::Key5 => Some(egui::Key::Num5),
        KeyCode::Key6 => Some(egui::Key::Num6),
        KeyCode::Key7 => Some(egui::Key::Num7),
        KeyCode::Key8 => Some(egui::Key::Num8),
        KeyCode::Key9 => Some(egui::Key::Num9),
        KeyCode::Key0 => Some(egui::Key::Num0),
        KeyCode::A => Some(egui::Key::A),
        KeyCode::B => Some(egui::Key::B),
        KeyCode::C => Some(egui::Key::C),
        KeyCode::D => Some(egui::Key::D),
        KeyCode::E => Some(egui::Key::E),
        KeyCode::F => Some(egui::Key::F),
        KeyCode::G => Some(egui::Key::G),
        KeyCode::H => Some(egui::Key::H),
        KeyCode::I => Some(egui::Key::I),
        KeyCode::J => Some(egui::Key::J),
        KeyCode::K => Some(egui::Key::K),
        KeyCode::L => Some(egui::Key::L),
        KeyCode::M => Some(egui::Key::M),
        KeyCode::N => Some(egui::Key::N),
        KeyCode::O => Some(egui::Key::O),
        KeyCode::P => Some(egui::Key::P),
        KeyCode::Q => Some(egui::Key::Q),
        KeyCode::R => Some(egui::Key::R),
        KeyCode::S => Some(egui::Key::S),
        KeyCode::T => Some(egui::Key::T),
        KeyCode::U => Some(egui::Key::U),
        KeyCode::V => Some(egui::Key::V),
        KeyCode::W => Some(egui::Key::W),
        KeyCode::X => Some(egui::Key::X),
        KeyCode::Y => Some(egui::Key::Y),
        KeyCode::Z => Some(egui::Key::Z),
        KeyCode::Escape => Some(egui::Key::Escape),
        KeyCode::F1 => Some(egui::Key::F1),
        KeyCode::F2 => Some(egui::Key::F2),
        KeyCode::F3 => Some(egui::Key::F3),
        KeyCode::F4 => Some(egui::Key::F4),
        KeyCode::F5 => Some(egui::Key::F5),
        KeyCode::F6 => Some(egui::Key::F6),
        KeyCode::F7 => Some(egui::Key::F7),
        KeyCode::F8 => Some(egui::Key::F8),
        KeyCode::F9 => Some(egui::Key::F9),
        KeyCode::F10 => Some(egui::Key::F10),
        KeyCode::F11 => Some(egui::Key::F11),
        KeyCode::F12 => Some(egui::Key::F12),
        KeyCode::F13 => Some(egui::Key::F13),
        KeyCode::F14 => Some(egui::Key::F14),
        KeyCode::F15 => Some(egui::Key::F15),
        KeyCode::F16 => Some(egui::Key::F16),
        KeyCode::F17 => Some(egui::Key::F17),
        KeyCode::F18 => Some(egui::Key::F18),
        KeyCode::F19 => Some(egui::Key::F19),
        KeyCode::F20 => Some(egui::Key::F20),
        KeyCode::Insert => Some(egui::Key::Insert),
        KeyCode::Home => Some(egui::Key::Home),
        KeyCode::Delete => Some(egui::Key::Delete),
        KeyCode::End => Some(egui::Key::End),
        KeyCode::PageDown => Some(egui::Key::PageDown),
        KeyCode::PageUp => Some(egui::Key::PageUp),
        KeyCode::Left => Some(egui::Key::ArrowLeft),
        KeyCode::Up => Some(egui::Key::ArrowUp),
        KeyCode::Right => Some(egui::Key::ArrowRight),
        KeyCode::Down => Some(egui::Key::ArrowDown),
        KeyCode::Back => Some(egui::Key::Backspace),
        KeyCode::Return => Some(egui::Key::Enter),
        KeyCode::Space => Some(egui::Key::Space),
        KeyCode::Compose => None,
        KeyCode::Caret => None,
        KeyCode::Numlock => None,
        KeyCode::Numpad0 => Some(egui::Key::Num0),
        KeyCode::Numpad1 => Some(egui::Key::Num1),
        KeyCode::Numpad2 => Some(egui::Key::Num2),
        KeyCode::Numpad3 => Some(egui::Key::Num3),
        KeyCode::Numpad4 => Some(egui::Key::Num4),
        KeyCode::Numpad5 => Some(egui::Key::Num5),
        KeyCode::Numpad6 => Some(egui::Key::Num6),
        KeyCode::Numpad7 => Some(egui::Key::Num7),
        KeyCode::Numpad8 => Some(egui::Key::Num8),
        KeyCode::Numpad9 => Some(egui::Key::Num9),
        KeyCode::Equals => Some(egui::Key::PlusEquals),
        KeyCode::Tab => Some(egui::Key::Tab),
        _ => None,
    }
}
