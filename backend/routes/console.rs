//! Schedule and named CLI command registrations (`command:list`).

use larust_support::console::CommandRegistry;
use larust_support::schedule::Schedule;

pub fn schedule() -> Schedule {
    Schedule::new()
        // Cache entries and failed-job history (30-day retention) don't
        // need anything faster than once a day, and `on_one_server` keeps a
        // rolling deploy's overlapping scheduler processes from both
        // running it for the same due instant.
        .daily(|| larust_support::maintenance::clean())
        .name("maintenance:clean")
        .on_one_server()
}

pub fn commands() -> CommandRegistry {
    CommandRegistry::new()
}
