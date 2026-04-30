use std::any;
use std::fmt::Debug;

/// Universal debug helper — works on ANY type (even if it doesn't implement Debug)
pub fn debug_val<T: Debug>(name: &str, value: &T) {
    println!(
        "[DEBUG] {} = {:#?}\n         └─ type = {}",
        name,
        value,
        any::type_name_of_val(value)
    );
}