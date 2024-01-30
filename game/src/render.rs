use bevy_utils::HashMap;
use binsig::Pattern;
use windows::{
    core::*,
    Win32::Graphics::Dxgi::{Common::DXGI_FORMAT, IDXGISwapChain},
};

use bevy_app::prelude::*;
use bevy_ecs::{prelude::*, schedule::ScheduleLabel};
use bevy_schedule_hook::{
    prelude::{
        dispatch_input::{DispInABC, DispInABCDEF},
        DispOut,
    },
    Detours,
};

use epaint_dx11::DirectX11Renderer;

use crate::utils::{get_module, module_addr, module_to_bytes};
/// The schedule that assumes the role of present.
///
/// NOTE: This is a dispatcher.
#[derive(ScheduleLabel, Debug, Default, Hash, PartialEq, Eq, Clone)]
pub struct Present;
type PresentFn = extern "system" fn(IDXGISwapChain, u32, u32) -> HRESULT;
type PresentInput = DispInABC<Present, IDXGISwapChain, u32, u32>;
type PresentOutput = DispOut<Present, HRESULT>;

/// The schedule that assumes the role of resize buffers.
///
/// NOTE: This is a dispatcher.
#[derive(ScheduleLabel, Debug, Default, Hash, PartialEq, Eq, Clone)]
pub struct ResizeBuffers;
type ResizeBuffersFn =
    extern "system" fn(IDXGISwapChain, u32, u32, u32, DXGI_FORMAT, u32) -> HRESULT;
type ResizeBuffersInput =
    DispInABCDEF<ResizeBuffers, IDXGISwapChain, u32, u32, u32, DXGI_FORMAT, u32>;
type ResizeBuffersOutput = DispOut<ResizeBuffers, HRESULT>;

fn hook_present(mut detours: ResMut<Detours>) {
    log::debug!("hooking present...");
    detours.add_detour::<_, PresentFn>(Present, find_present_fn().unwrap());
    detours.enable_detour(Present);
}

pub fn present_original(
    input: NonSend<PresentInput>,
    mut output: NonSendMut<PresentOutput>,
    detours: Res<Detours>,
) {
    let original: PresentFn = detours.get_detour_original(Present).unwrap();
    output.ret = original(input.__arg_0.to_owned(), input.__arg_1, input.__arg_2);
}

fn hook_resize_buffers(input: NonSend<PresentInput>, mut detours: ResMut<Detours>) {
    // Setup resize buffer detour.
    let resize_buffers_fn = Interface::vtable(&input.__arg_0).ResizeBuffers;
    detours.add_detour::<_, ResizeBuffersFn>(ResizeBuffers, unsafe {
        std::mem::transmute(resize_buffers_fn)
    });
    detours.enable_detour(ResizeBuffers);
}

fn resize_buffers_original(
    input: NonSend<ResizeBuffersInput>,
    mut output: NonSendMut<ResizeBuffersOutput>,
    detours: Res<Detours>,
) {
    let original: ResizeBuffersFn = detours.get_detour_original(ResizeBuffers).unwrap();
    output.ret = original(
        input.__arg_0.to_owned(),
        input.__arg_1,
        input.__arg_2,
        input.__arg_3,
        input.__arg_4,
        input.__arg_5,
    );
}

fn setup_render_targets(input: NonSend<PresentInput>, mut render_targets: ResMut<RenderTargets>) {
    let swapchain = &input.__arg_0;
    render_targets.setup_renderers_from_swapchain(swapchain);
}

#[derive(Component, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Debug)]
pub struct RenderTargetHandle(usize);

#[derive(Resource, Default)]
pub struct RenderTargets {
    renderers: HashMap<RenderTargetHandle, Option<DirectX11Renderer>>,
}

impl RenderTargets {
    pub fn create_uninitialized_render_target(&mut self) -> RenderTargetHandle {
        let handle: RenderTargetHandle = RenderTargetHandle(self.renderers.len());
        log::debug!("created render target with handle {}", handle.0);
        self.renderers.insert(handle, None);
        handle
    }

    pub fn from_handle(&mut self, handle: RenderTargetHandle) -> Option<&mut DirectX11Renderer> {
        match self.renderers.get_mut(&handle)? {
            Some(r) => Some(r),
            None => None,
        }
    }

    pub fn setup_renderers_from_swapchain(&mut self, swapchain: &IDXGISwapChain) {
        self.renderers
            .iter_mut()
            .filter(|(_, r)| r.is_none())
            .for_each(|(_, r)| {
                *r = Some(unsafe {
                    epaint_dx11::DirectX11Renderer::init_from_swapchain(swapchain)
                        .expect("failed to create renderer from swapchain")
                })
            })
    }
}

#[derive(Event)]
pub struct RenderEvent {
    // TODO: Window handle as well...
    // pub window_handle: WindowRef,
    pub render_target_handle: RenderTargetHandle,
    pub primitives: Vec<epaint::ClippedPrimitive>,
    pub textures_delta: epaint::textures::TexturesDelta,
}

// TODO: register an event when the pixel_per_point changes to update text shapes.

fn resize_buffers_update_render(
    mut render_targets: ResMut<RenderTargets>,
    input: NonSend<ResizeBuffersInput>,
) {
    // let mut render_target = match render_target {
    //     Some(r) => r,
    //     None => return,
    // };

    // render_target.on_resize_buffers(&input.__arg_0, original)
}

pub fn present_render_primitives(
    mut render_targets: ResMut<RenderTargets>,
    input: NonSend<PresentInput>,
    mut ev_render: EventReader<RenderEvent>,
) {
    for re in ev_render.iter() {
        if let Some(render_target) = render_targets.from_handle(re.render_target_handle) {
            unsafe {
                render_target
                    .paint_primitives(
                        &input.__arg_0,
                        (1920.0, 1080.0),
                        // TODO: Cloning here :skull:
                        re.textures_delta.clone(),
                        re.primitives.clone(),
                    )
                    .expect("paint_primitives failed")
            }
        }
    }
}

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        // TODO: Why does `present_sched` crash... OHHH look at the dispatch...
        let mut present_sched = Schedule::new();
        present_sched.set_executor_kind(bevy_ecs::schedule::ExecutorKind::MultiThreaded);
        app.init_resource::<RenderTargets>()
            .init_schedule(Present)
            .init_schedule(ResizeBuffers)
            .add_event::<RenderEvent>()
            // TODO: PostStartup or PreStartup
            .add_systems(PostStartup, hook_present)
            .add_systems(
                Present,
                (
                    hook_resize_buffers.run_if(run_once()),
                    setup_render_targets.before(present_render_primitives),
                    present_render_primitives.before(present_original),
                    present_original,
                ),
            )
            .add_systems(
                ResizeBuffers,
                (resize_buffers_original, resize_buffers_update_render),
            );
    }
}

fn find_present_fn() -> Option<PresentFn> {
    let module = get_module("GameOverlayRenderer64.dll")?;
    Pattern::from_ida("48 89 6C 24 ?? 48 89 74 24 ?? 41 56 48 83 EC ?? 41 8B E8")
        .unwrap()
        .scan(module_to_bytes(module))
        .next()
        .map(|(offset, _)| unsafe { std::mem::transmute(module_addr(module) + offset) })
}
