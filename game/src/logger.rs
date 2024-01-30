use bevy_app::{App, Plugin};
use log::Level;

pub struct LogPlugin {
    /// Filters logs using the [`EnvFilter`] format
    pub filter: String,

    /// Filters out logs that are "less than" the given level.
    /// This can be further filtered using the `filter` setting.
    pub level: Level,
}

impl Default for LogPlugin {
    fn default() -> Self {
        Self {
            filter: "".to_string(),
            level: Level::Trace,
        }
    }
}

impl Plugin for LogPlugin {
    fn build(&self, _app: &mut App) {
        simple_logger::init_with_level(self.level).unwrap();
    }

    // TODO: On destroy we need to do stuff.
}
