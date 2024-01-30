use bevy_app::prelude::*;
use bevy_ecs::{prelude::*, schedule::ScheduleLabel};
use bevy_schedule_hook::prelude::*;
use bevy_win32::{window::Win32Windows, WinMessageEvent};
use binsig::Pattern;
use windows::Win32::UI::WindowsAndMessaging::MSG;

use crate::utils::{get_module, module_addr, module_to_bytes};

/// The schedule that assumes the role of `CallWndProcFn`.
///
/// NOTE: This is a dispatchable hook.
#[derive(ScheduleLabel, Debug, Default, Hash, PartialEq, Eq, Clone)]
pub struct OverlayMessageHandler;
pub type OverlayMessageHandlerFn = extern "system" fn(*mut MSG, u64) -> u64;
type OverlayMessageHandlerInput = dispatch_input::DispInAB<OverlayMessageHandler, *mut MSG, u64>;
type OverlayMessageHandlerOutput = DispOut<OverlayMessageHandler, u64>;

// NOTE: Must be executed in a thread created by the game process.
fn hook_overlay_input(mut detours: ResMut<Detours>) {
    log::debug!("hooking overlay input...");
    detours.add_detour::<_, OverlayMessageHandlerFn>(
        OverlayMessageHandler,
        find_overlay_input_fn().unwrap(),
    );
    detours.enable_detour(OverlayMessageHandler);
}

pub fn message_handler(
    windows: Res<Win32Windows>,
    input: NonSend<OverlayMessageHandlerInput>,
    mut wm_event: EventWriter<bevy_win32::WinMessageEvent>,
) {
    // TODO: We need a way to stop game from receiving events...
    let msg = unsafe { input.__arg_0.read() };
    wm_event.send(WinMessageEvent {
        msg: msg.message,
        wparam: msg.wParam,
        lparam: msg.lParam,
        window: windows.get_window_entity(msg.hwnd.into()).unwrap(),
    });
}

// TODO: We cant otherwise it will go and call itself...
pub fn overlay_message_handler_original(
    input: NonSend<OverlayMessageHandlerInput>,
    mut output: NonSendMut<OverlayMessageHandlerOutput>,
    detours: Res<Detours>,
) {
    let original: OverlayMessageHandlerFn =
        detours.get_detour_original(OverlayMessageHandler).unwrap();
    output.ret = original(input.__arg_0, input.__arg_1);
}

pub struct OverlayInputPlugin;

impl Plugin for OverlayInputPlugin {
    fn build(&self, app: &mut App) {
        app.init_schedule(OverlayMessageHandler)
            // TODO: PostStartup or PreStartup
            .add_systems(PostStartup, hook_overlay_input)
            .add_systems(OverlayMessageHandler, message_handler);
    }
}

fn find_overlay_input_fn() -> Option<OverlayMessageHandlerFn> {
    let module = get_module("GameOverlayRenderer64.dll").unwrap();
    Pattern::from_ida("48 89 5C 24 ?? 48 89 74 24 ?? 48 89 7C 24 ?? 55 41 54 41 55 41 56 41 57 48 8D 6C 24 C9 48 81 EC ?? ?? ?? ?? 44 0F B6 E2")
        .unwrap()
        .scan(module_to_bytes(module))
        .next()
        .map(|(offset, _)| unsafe { std::mem::transmute(module_addr(module) + offset) })
}
