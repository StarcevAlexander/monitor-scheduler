use crate::models::Action;
use crate::monitor;

pub fn apply_action(a: Action) -> Result<(), String> {
    match a {
        Action::Off => monitor::monitors_off(),
        Action::On => monitor::monitors_on(),
    }
}
