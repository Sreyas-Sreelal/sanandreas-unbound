macro_rules! log {
    ($string:literal) => {
        omp::core::Log(&format!($string));
    };
    ($string:literal,$($args:expr),*) => {
        omp::core::Log(&format!($string,$($args),*));
    };
}
