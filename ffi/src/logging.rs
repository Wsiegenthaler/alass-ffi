use crate::result_codes::*;
use crate::util::*;
use crate::catch_panic;

use std::os::raw::c_char;

use log::LevelFilter;

pub type LogLevel = usize;

#[no_mangle] pub static ALASS_LOG_NONE:  LogLevel = 0;
#[no_mangle] pub static ALASS_LOG_ERROR: LogLevel = 1;
#[no_mangle] pub static ALASS_LOG_WARN:  LogLevel = 2;
#[no_mangle] pub static ALASS_LOG_INFO:  LogLevel = 3;
#[no_mangle] pub static ALASS_LOG_DEBUG: LogLevel = 4;
#[no_mangle] pub static ALASS_LOG_TRACE: LogLevel = 5;

///
/// Configure logging
/// 
/// Logging can only be configured once per application; subsequent calls will
/// fail with `ALASS_LOG_ALREADY_CONFIGURED`.
/// 
/// Also note that `stderr_level` takes precedence over `stdout_level`, so any
/// overlap will only go to `stderr`. For example, if `stdout` is `ALASS_LOG_DEBUG`
/// and `stderr` is `ALASS_LOG_WARN`, debug and info events will go to `stdout`
/// and warn and error to `stderr`).
/// 
/// * `stdout_level`: Threshold of log events that should be written to `stdout`.
/// * `stderr_level`: Threshold of log events that should be written to `stderr`.
/// * `log_file_level`: Threshold of log events that should be written to log file.
/// * `log_file`: Path of the file to recieve log events (or null if not applicable).
///
#[catch_panic(ALASS_INTERNAL_ERROR)]
#[no_mangle]
pub extern "C" fn alass_log_config(
    stdout_level: LogLevel,
    stderr_level: LogLevel,
    log_file_level: LogLevel,
    log_file: *const c_char
) -> ResultCode {

    let log_file = from_cstring(log_file);

    let stdout_filter = level_filter(stdout_level);
    let stderr_filter = level_filter(stderr_level);
    let file_filter = level_filter(log_file_level);

    let base_config = fern::Dispatch::new()
        .format(|out, msg, record| {
            let time = chrono::Local::now().format("%H:%M:%S%.3f");
            let level = format!("{}", record.level()).to_lowercase();
            out.finish(format_args!("{} [libalass][{}] {}", time, level, msg));
        });

    // Stdout
    let stdout_config = fern::Dispatch::new()
        .filter(move |metadata| metadata.level() > stderr_filter && metadata.level() <= stdout_filter)
        .chain(std::io::stdout());

    // Stderr
    let stderr_config = fern::Dispatch::new()
        .level(stderr_filter)
        .chain(std::io::stderr());

    // Log file
    let base_config = if log_file.is_some() && file_filter != LevelFilter::Off {
        match fern::log_file(log_file.unwrap()) {
            Ok(log_file) => base_config.chain(
                fern::Dispatch::new()
                    .format(|out, msg, _| {
                        let date = chrono::Local::now().format("%d/%m/%Y");
                        out.finish(format_args!("{} {}", date, msg));
                    })
                    .level(file_filter)
                    .chain(log_file)),
            Err(_) => return ALASS_INTERNAL_ERROR
        }
    } else {
        base_config
    };

    let base_config = base_config
        .chain(stdout_config)
        .chain(stderr_config);

    match base_config.apply() {
        Ok(()) => ALASS_SUCCESS,
        Err(_) => ALASS_LOG_ALREADY_CONFIGURED
    }
}

fn level_filter(level: LogLevel) -> LevelFilter {
    if level == ALASS_LOG_NONE {
        LevelFilter::Off
    } else if level == ALASS_LOG_ERROR {
        LevelFilter::Error
    } else if level == ALASS_LOG_WARN {
        LevelFilter::Warn
    } else if level == ALASS_LOG_INFO {
        LevelFilter::Info
    } else if level == ALASS_LOG_DEBUG {
        LevelFilter::Debug
    } else {
        LevelFilter::Trace
    }
}
