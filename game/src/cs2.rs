use bevy_app::{App, Plugin};

mod schema;
mod tier0;
mod tier1;
mod utils;

pub struct Cs2Plugin;

impl Plugin for Cs2Plugin {
    fn build(&self, app: &mut App) {}

    fn finish(&self, app: &mut App) {}
}
