#![feature(const_option)]
use std::io::{Result, Write};
use std::fs::{File, read_dir};

fn main() {
    println!("cargo:rerun-if-changed={}", WATCH_PATH);
    println!("cargo:rerun-if-changed={}", TARGET_PATH);
    insert_app_data().unwrap();
}

const WATCH_PATH: &str = option_env_with_default!("WATCH_PATH", "../user/src/");
const SOURCE_PATH: &str = option_env_with_default!("SOURCE_PATH", "../user/src/bin");
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

fn insert_app_data() -> Result<()> {
    let mut f = File::create("src/link_app.S").unwrap();
    let mut apps: Vec<_> = read_dir(SOURCE_PATH)
        .unwrap()
        .into_iter()
        .map(|dir_entry| {
            let mut name_with_ext = dir_entry.unwrap().file_name().into_string().unwrap();
            name_with_ext.drain(name_with_ext.find('.').unwrap()..name_with_ext.len());
            name_with_ext
        })
        .collect();
    apps.sort();

    writeln!(f, r#"
    .align 3
    .section .data
    .global _num_app
_num_app:
    .quad {}"#, apps.len())?;

    for i in 0..apps.len() {
        writeln!(f, r#"    .quad app_{}_start"#, i)?;
    }
    writeln!(f, r#"    .quad app_{}_end"#, apps.len() - 1)?;

    for (idx, app) in apps.iter().enumerate() {
        println!("app_{}: {}", idx, app);
        writeln!(f, r#"
    .section .data
    .global app_{0}_start
    .global app_{0}_end
app_{0}_start:
    .incbin "{2}{1}.bin"
app_{0}_end:"#, idx, app, TARGET_PATH)?;
    }
    Ok(())
}