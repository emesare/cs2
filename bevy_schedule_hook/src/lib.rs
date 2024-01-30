#![feature(fn_ptr_trait)]

use std::{any::TypeId, marker::FnPtr};

use bevy_app::prelude::*;
use bevy_ecs::{prelude::*, schedule::ScheduleLabel};
use bevy_schedule_dispatch::prelude::*;
use bevy_utils::HashMap;
use retour::RawDetour;

pub mod prelude {
    pub use crate::{DetourPlugin, Detours};
    pub use bevy_schedule_dispatch::prelude::*;
}

// Normally we could just make every hook its own component and attach it to something, but it needs to be queriable through the schedule label's type id.
#[derive(Resource, Debug, Default)]
pub struct Detours {
    pub detours: HashMap<TypeId, RawDetour>,
}

impl Detours {
    // TODO: Make result
    pub fn add_detour<
        S: ScheduleLabel + Default + AsRef<(dyn ScheduleLabel + 'static)>,
        D: Dispatchable + retour::Function + FnPtr,
    >(
        &mut self,
        schedule: S,
        target: D,
    ) {
        let detour = unsafe {
            retour::RawDetour::new(target.to_ptr(), D::dispatcher::<S>().addr()).unwrap()
        };
        self.detours.insert(schedule.inner_type_id(), detour);
    }

    // TODO: Make result
    pub fn enable_detour<S: ScheduleLabel>(&self, schedule: S) {
        unsafe { self.get_detour(schedule).unwrap().enable().unwrap() };
    }

    // TODO: Make result
    pub fn disable_detour<S: ScheduleLabel>(&self, schedule: S) {
        unsafe { self.get_detour(schedule).unwrap().disable().unwrap() };
    }

    pub fn get_detour<S: ScheduleLabel>(&self, schedule: S) -> Option<&RawDetour> {
        self.detours.get(&schedule.inner_type_id())
    }

    pub fn get_detour_original<S: ScheduleLabel, F: retour::Function>(
        &self,
        schedule: S,
    ) -> Option<F> {
        Some(unsafe { F::from_ptr(std::mem::transmute(self.get_detour(schedule)?.trampoline())) })
    }

    // NOTE: When this is removed, it will `drop` the detour, causing it to be disabled.
    pub fn remove_detour<S: ScheduleLabel>(&mut self, schedule: S) -> Option<RawDetour> {
        self.detours.remove(&schedule.inner_type_id())
    }

    pub fn remove_all_detours(&mut self) {
        self.detours.clear();
    }
}

pub struct DetourPlugin;

impl Plugin for DetourPlugin {
    fn build(&self, app: &mut bevy_app::App) {
        app.init_resource::<Detours>();
    }
}

// TODO: Proc macro
/*
#[derive(ScheduleHook)]
type ResizeBuffersFn =
    extern "system" fn(IDXGISwapChain, u32, u32, u32, DXGI_FORMAT, u32) -> HRESULT;

expands to:

/// The schedule that assumes the role of `ResizeBuffersFn`.
///
/// NOTE: This is a dispatchable hook.
#[derive(ScheduleLabel, Debug, Default, Hash, PartialEq, Eq, Clone)]
pub struct ResizeBuffers;
type ResizeBuffersFn =
    extern "system" fn(IDXGISwapChain, u32, u32, u32, DXGI_FORMAT, u32) -> HRESULT;
type ResizeBuffersInput = DispInABCDE<IDXGISwapChain, u32, u32, u32, DXGI_FORMAT, u32>;
type ResizeBuffersOutput = DispOut<ExampleHook, HRESULT>;

fn resize_buffers_original(
    input: NonSend<ResizeBuffersInput>,
    mut output: NonSendMut<ResizeBuffersOutput>,
    // Taking in detours like this in every original call sucks.
    detours: Res<Detours>,
) {
    let original: ResizeBuffersFn = detours.get_detour_original(ResizeBuffers).unwrap();
    output.ret = original(input.__arg_0, input.__arg_1, input.__arg_2, input.__arg_3, input.__arg_4, input.__arg_5);
}
*/
