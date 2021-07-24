fn main() {
    println!("cargo:rerun-if-changed={}", WATCH_PATH);
    println!("cargo:rerun-if-changed={}", TARGET_PATH);
}

const WATCH_PATH: &str = option_env_with_default!("WATCH_PATH", "../user/src/");
const TARGET_PATH: &str = option_env_with_default!("TARGET_PATH", "../user/target/riscv64gc-unknown-none-elf/release/");

#[macro_export]
macro_rules! option_env_with_default {
    ($name:expr,$default:expr) => {
        {
            match option_env!($name) {
                Some(x) => x,
                None => $default
            }
        }
    };
}
