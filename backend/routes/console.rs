//! Schedule and named CLI command registrations (`command:list`).

use larust_support::console::CommandRegistry;
use larust_support::schedule::Schedule;

pub fn schedule() -> Schedule {
    Schedule::new()
}

pub fn commands() -> CommandRegistry {
    CommandRegistry::new()
}
