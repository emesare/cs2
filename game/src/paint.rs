use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use egui::{LayerId, RawInput};

use crate::render::{Present, RenderEvent, RenderTargetHandle, RenderTargets};

// TODO: How to do layering? We need to offer a way to say "this needs to be on top of all other Paintable"

// TODO: Take in events that would change either and apply!
pub fn painter_begin_frame(mut query: Query<&mut PainterContext>) {
    for mut ctx in query.iter_mut() {
        ctx.get_mut().ctx().begin_frame(RawInput::default())
    }
}

pub fn painter_end_frame(
    mut ev_render: EventWriter<RenderEvent>,
    mut query: Query<&mut PainterContext>,
) {
    ev_render.send_batch(
        query
            .iter_mut()
            .map(|mut ui_ctx| {
                let ctx = ui_ctx.get_mut().ctx();
                let out = ctx.end_frame();
                // Use the egui contexts own tesselation, so that the ui's registered fonts exist.
                let primitives = ctx.tessellate(out.shapes);
                // TODO: support repaint_after?
                RenderEvent {
                    render_target_handle: ui_ctx.render_target_handle,
                    primitives,
                    textures_delta: out.textures_delta,
                }
            })
            .collect::<Vec<_>>(),
    );
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub struct PainterUpdate;

pub struct PaintPlugin;

impl Plugin for PaintPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Present,
            (
                painter_begin_frame,
                painter_end_frame.after(painter_begin_frame),
            ),
        )
        .configure_set(
            Present,
            PainterUpdate
                .after(painter_begin_frame)
                .before(painter_end_frame),
        );
    }
}

// TODO: Add primarypaintercontext

// TODO: Paint has its own EVERYTHING to prevent the scheduler from serializing everything one after another...
#[derive(Component)]
pub struct PainterContext {
    painter: egui::Painter,
    render_target_handle: RenderTargetHandle,
}

impl FromWorld for PainterContext {
    fn from_world(world: &mut World) -> Self {
        let mut render_targets = world
            .get_resource_mut::<RenderTargets>()
            .expect("RenderTargets should be available");

        Self::new(render_targets.as_mut().create_uninitialized_render_target())
    }
}

// Make the painter like a window, have a primary painter thats easy to get.
// But in cases where we need a second we can have one.
// Fonts should be shared between painters? IDK, just know that the ui has its own set of fonts.

// TODO: Make a builder thing for this?
// TODO: FromWorld to get pixels_per_point and MAX_TEXTURE_SIDE?
impl PainterContext {
    pub fn new(render_target_handle: RenderTargetHandle) -> Self {
        let paint_ctx = egui::Context::default();
        let painter = paint_ctx.layer_painter(LayerId::background());
        Self {
            render_target_handle,
            painter: painter,
        }
    }

    /// Borrows the underlying egui::Painter context mutably.
    ///
    /// When the context is queried with `&mut egui::Painter`, the Bevy scheduler is able to make
    /// sure that the context isn't accessed concurrently and can perform other useful work
    /// instead of busy-waiting.
    #[must_use]
    pub fn get_mut(&mut self) -> &mut egui::Painter {
        // TODO: Make rwlock or not needed?
        &mut self.painter
    }
}
