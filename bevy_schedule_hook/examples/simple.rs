use bevy_app::prelude::*;
use bevy_ecs::{prelude::*, schedule::ScheduleLabel};
use bevy_schedule_dispatch::{prelude::dispatch_input, DispOut, DispatchPlugin};
use bevy_schedule_hook::*;

#[derive(ScheduleLabel, Debug, Default, Hash, PartialEq, Eq, Clone)]
pub struct ExampleHook;

fn example_original_fn(p0: bool) -> i32 {
    println!("original has been called with: {}", p0);
    return 5;
}

fn example_system(input: NonSend<dispatch_input::DispInA<ExampleHook, bool>>) {
    println!("example hook called! {:?}", input);
}

fn example_system_orig(
    input: NonSend<dispatch_input::DispInA<ExampleHook, bool>>,
    mut output: NonSendMut<DispOut<ExampleHook, i32>>,
    // Taking in detours like this in every original call sucks.
    detours: Res<Detours>,
) {
    // TODO: This sucks, we might need more codegen to do DispInA + DispOut -> fn pointer.
    // TODO: Maybe a impl block on fn pointers that destructures it into types.
    // TODO: Maybe we could store the original when the detour is first added?
    // TODO: Like: `generate_ctx!(ScheduleName, fn(i32) -> i32)` will give us a few things like this function here for free?
    let original: fn(bool) -> i32 = detours.get_detour_original(ExampleHook).unwrap();
    output.ret = original(input.__arg_0);
}

fn example_system_ret(mut output: NonSendMut<DispOut<ExampleHook, i32>>) {
    println!("example hook called, multiplying return value!");
    output.ret = output.ret * 2;
}

fn create_detours(mut detours: ResMut<Detours>) {
    detours.add_detour(ExampleHook, example_original_fn as fn(bool) -> i32);
    detours.enable_detour(ExampleHook);
}

pub fn main() {
    App::new()
        // TODO: Does DetourPlugin add Dispatch plugin? we NEED Dispatch to be called first prolly :/
        .add_plugins((DispatchPlugin, DetourPlugin))
        .init_schedule(ExampleHook)
        .add_systems(
            ExampleHook,
            (
                example_system,
                example_system_ret,
                example_system_orig.before(example_system_ret),
            ),
        )
        .add_systems(Update, create_detours)
        .set_runner(|mut app| {
            app.finish();
            app.cleanup();

            app.update();

            let _global_app = DispatchPlugin::globalize_app(app);

            std::thread::spawn(|| {
                let ret = example_original_fn(true);
                println!("example hook returned {}!", ret);
            })
            .join()
            .unwrap();
        })
        .run();
}
