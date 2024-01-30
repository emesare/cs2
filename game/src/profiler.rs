use crate::{
    render::Present,
    ui::{self, UiContext},
};
use bevy_app::{App, Plugin};
use bevy_ecs::prelude::*;

pub struct ProfilerPlugin;

fn mark_new_profiled_frame() {
    puffin::GlobalProfiler::lock().new_frame()
}

fn profiler_ui(mut ui_ctx: ResMut<UiContext>) {
    puffin_egui::profiler_window(&ui_ctx.get_mut());
}

/// Marker [`Component`] for the profilers ui context.
#[derive(Default, Debug, Component, PartialEq, Eq, PartialOrd, Ord, Copy, Clone)]
pub struct ProfilerUiContext;

impl Plugin for ProfilerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Present,
            (
                mark_new_profiled_frame,
                profiler_ui.after(ui::ui_begin_frame),
            ),
        );
    }

    fn finish(&self, app: &mut App) {
        // TODO: Be able to turn on and off.
        puffin::set_scopes_on(true);
    }
}
