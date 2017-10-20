
#[macro_export]
macro_rules! error_formatter {
    ($prefix:expr) => { |error| format!("{} Error: {}", $prefix, error) }
}
