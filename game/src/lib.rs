#![feature(slice_take)]

use std::ffi::c_void;

use bevy_app::{App, PostStartup};
use bevy_ecs::{
    event::EventWriter,
    schedule::IntoSystemConfigs,
    system::{Query, ResMut},
    world::{FromWorld, World},
};
use bevy_schedule_hook::{prelude::DispatchPlugin, DetourPlugin};
use bevy_utils::Duration;
use bevy_win32::Win32Plugin;
use egui::Align2;
use epaint::Color32;
use ui::UiPlugin;
use windows::Win32::{
    Foundation::{CloseHandle, BOOL, HINSTANCE},
    System::{
        LibraryLoader::DisableThreadLibraryCalls,
        SystemServices::{DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH},
        Threading::{CreateThread, THREAD_CREATION_FLAGS},
    },
    UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_DELETE, VK_INSERT},
};

use crate::{
    input::OverlayInputPlugin,
    logger::LogPlugin,
    paint::{PaintPlugin, PainterContext, PainterUpdate},
    profiler::ProfilerPlugin,
    render::{Present, RenderPlugin},
    ui::{UiContext, UiUpdate},
    utils::get_window_hwnd,
};

mod cs2;
mod input;
mod logger;
mod paint;
mod profiler;
mod render;
mod ui;
mod utils;

#[no_mangle]
#[allow(non_snake_case)]
unsafe extern "system" fn DllMain(module: HINSTANCE, reason: u32, _reserved: *const u8) -> BOOL {
    match reason {
        DLL_PROCESS_ATTACH => {
            let _ = DisableThreadLibraryCalls(module);

            if let Ok(thread_handle) = CreateThread(
                None,
                0,
                Some(attach_thread),
                Some(module.0 as _),
                THREAD_CREATION_FLAGS::default(),
                None,
            ) {
                let _ = CloseHandle(thread_handle);
            } else {
                return false.into();
            }
        }
        DLL_PROCESS_DETACH => {
            // TODO: Make this safely unhook and also unpatch game.
        }
        _ => {}
    }

    true.into()
}

unsafe extern "system" fn attach_thread(_lp_module: *mut c_void) -> u32 {
    utils::alloc_console();
    while GetAsyncKeyState(VK_INSERT.0.into()) == 0_i16 {}

    let mut app = App::new();

    fn run_until_exit(app: App) {
        log::debug!("globalizing app...");
        let arc_app = DispatchPlugin::globalize_app(app);
        {
            let mut locked_app = arc_app.lock().unwrap();
            locked_app.finish();
            locked_app.cleanup();

            locked_app.update();
        }

        log::info!("press DELETE to uninject");
        unsafe {
            while GetAsyncKeyState(VK_DELETE.0.into()) == 0_i16 {
                std::thread::sleep(Duration::from_millis(16));
                arc_app.lock().unwrap().update();
            }
        }
    }

    fn paint_test(mut query: Query<&mut PainterContext>) {
        let mut p = query.get_single_mut().unwrap();
        let mut p = p.get_mut();

        p.rect_filled(
            epaint::Rect::from_center_size(
                epaint::Pos2 {
                    x: 1000.0,
                    y: 1000.0,
                },
                epaint::vec2(250.0, 250.0),
            ),
            5.0,
            Color32::GREEN,
        );

        p.text(
            epaint::Pos2 {
                x: 1000.0,
                y: 900.0,
            },
            Align2::LEFT_TOP,
            "HELLO FROM PAINT",
            epaint::FontId::proportional(20.0),
            Color32::RED,
        );
    }

    fn create_painter(world: &mut World) {
        let t = PainterContext::from_world(world);
        world.spawn(t);
    }

    fn test_ui(mut ui_ctx: ResMut<UiContext>) {
        ui_ctx.get_mut().debug_painter().circle_filled(
            (100.0, 100.0).into(),
            50.0,
            egui::Color32::LIGHT_RED,
        );

        egui::Window::new("drain")
            .fixed_pos((100.0 as f32, 200.0))
            .show(ui_ctx.get_mut(), |ui| {
                ui.label("Hello world!");
                // ui_ctx.get_mut().settings_ui(ui);
            });
    }

    fn add_primary_window(mut aw_event: EventWriter<bevy_win32::AddWindowEvent>) {
        log::debug!("adding primary game window...");
        aw_event.send(bevy_win32::AddWindowEvent {
            handle: get_window_hwnd().unwrap().into(),
            is_primary: true,
        })
    }

    app.set_runner(run_until_exit)
        .add_plugins((
            LogPlugin::default(),
            bevy_input::InputPlugin,
            bevy_window::WindowPlugin {
                primary_window: None,
                exit_condition: bevy_window::ExitCondition::OnPrimaryClosed,
                close_when_requested: true,
            },
            Win32Plugin,
            OverlayInputPlugin,
            ProfilerPlugin,
            DetourPlugin,
            RenderPlugin,
            UiPlugin,
            PaintPlugin,
        ))
        .add_systems(PostStartup, (create_painter, add_primary_window))
        .add_systems(
            Present,
            (paint_test.in_set(PainterUpdate), test_ui.in_set(UiUpdate)),
        );

    // TODO: THIS NEEDS TO BE REDONE, WE NEED TO MAKE OUR OWN RUNNER THAT MIMICKS THE REGULAR ONE... OR REUSE REGULAR ONE...

    // SEE WINIT: https://github.com/bevyengine/bevy/blob/main/crates/bevy_winit/src/lib.rs
    // TODO: THIS EXITS EVERYTHING IS CLEARED, WE NEED A PLUGIN to manage run by pacing the app
    app.run();

    log::info!("exiting...");

    1
}
