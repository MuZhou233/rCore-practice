use crate::console::print;
use log::{self, Level, LevelFilter, Log, Metadata, Record};

pub fn init() {
    static LOGGER: SimpleLogger = SimpleLogger;
    log::set_logger(&LOGGER).unwrap();
    log::set_max_level(match option_env!("LOG") {
        Some("ERROR") => LevelFilter::Error,
        Some("WARN")  => LevelFilter::Warn,
        Some("INFO")  => LevelFilter::Info,
        Some("DEBUG") => LevelFilter::Debug,
        Some("TRACE") => LevelFilter::Trace,
        _ => LevelFilter::Off,
    });
}

struct SimpleLogger;

impl Log for SimpleLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }
    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        print(format_args!("\u{1B}[{}m{}\u{1B}[0m",
            level_to_color_code(record.level()),
            format_args!(
                "[{:>5}] {}\n",
                record.level(),
                record.args()
            ),
        ));
    }
    fn flush(&self) {}
}

fn level_to_color_code(level: Level) -> u8 {
    match level {
        Level::Error => 31, // Red
        Level::Warn  => 93, // BrightYellow
        Level::Info  => 34, // Blue
        Level::Debug => 32, // Green
        Level::Trace => 90, // BrightBlack
    }
}