/* sys_info_v2/observatory-daemon/src/logging.rs
 *
 * Copyright 2023 Romeo Calota
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use lazy_static::lazy_static;

#[allow(unused)]
macro_rules! error {
    ($domain:literal, $($arg:tt)*) => {{
        $crate::logging::Logger::log_error($domain, format_args!($($arg)*));
    }}
}
pub(crate) use error;

#[allow(unused)]
macro_rules! critical {
    ($domain:literal, $($arg:tt)*) => {{
        $crate::logging::Logger::log_critical($domain, format_args!($($arg)*));
    }}
}
pub(crate) use critical;

#[allow(unused)]
macro_rules! warning {
    ($domain:literal, $($arg:tt)*) => {{
        $crate::logging::Logger::log_warn($domain, format_args!($($arg)*));
    }}
}
pub(crate) use warning;

#[allow(unused)]
macro_rules! message {
    ($domain:literal, $($arg:tt)*) => {{
        $crate::logging::Logger::log_message($domain, format_args!($($arg)*));
    }}
}
pub(crate) use message;

#[allow(unused)]
macro_rules! info {
    ($domain:literal, $($arg:tt)*) => {{
        $crate::logging::Logger::log_info($domain, format_args!($($arg)*));
    }}
}
pub(crate) use info;

#[allow(unused)]
macro_rules! debug {
    ($domain:literal, $($arg:tt)*) => {{
        $crate::logging::Logger::log_debug($domain, format_args!($($arg)*));
    }}
}
pub(crate) use debug;

macro_rules! now {
    () => {{
        let now = std::time::SystemTime::now();
        let now = now
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or(std::time::Duration::new(0, 0));

        let hours = (now.as_secs() / 3600) as u32;
        let minutes = ((now.as_secs() - (hours as u64 * 3600)) / 60) as u16;
        let seconds = (now.as_secs() - (hours as u64 * 3600) - (minutes as u64 * 60)) as u16;
        let milliseconds = now.subsec_millis() as u16;

        Timestamp {
            hours: hours % 24,
            minutes,
            seconds,
            milliseconds,
        }
    }};
}

lazy_static! {
    static ref PID: u32 = unsafe { libc::getpid() } as _;
    static ref G_MESSAGES_DEBUG: Vec<std::sync::Arc<str>> = std::env::var("G_MESSAGES_DEBUG")
        .unwrap_or_default()
        .split(";")
        .map(|s| std::sync::Arc::<str>::from(s))
        .collect();
}

const F_COL_LIGHT_BLUE: &str = "\x1b[2;34m";
const F_RESET: &str = "\x1b[0m";

struct Timestamp {
    hours: u32,
    minutes: u16,
    seconds: u16,
    milliseconds: u16,
}

#[allow(dead_code)]
enum LogLevel {
    Error,
    Critical,
    Warning,
    Message,
    Info,
    Debug,
}

pub struct Logger;

#[allow(dead_code)]
impl Logger {
    pub fn log_error(domain: &str, args: std::fmt::Arguments<'_>) {
        let color = Self::log_level_to_color(LogLevel::Error);
        let now = now!();
        eprintln!(
            "\n(observatory-daemon:{}): {}-{}{}{} **: {}{}:{}:{}.{}{}: {}",
            *PID,
            domain,
            color,
            "ERROR",
            F_RESET,
            F_COL_LIGHT_BLUE,
            now.hours,
            now.minutes,
            now.seconds,
            now.milliseconds,
            F_RESET,
            args
        );
    }

    pub fn log_critical(domain: &str, args: std::fmt::Arguments<'_>) {
        let color = Self::log_level_to_color(LogLevel::Critical);
        let now = now!();
        eprintln!(
            "\n(observatory-daemon:{}): {}-{}{}{} **: {}{}:{}:{}.{}{}: {}",
            *PID,
            domain,
            color,
            "CRITICAL",
            F_RESET,
            F_COL_LIGHT_BLUE,
            now.hours,
            now.minutes,
            now.seconds,
            now.milliseconds,
            F_RESET,
            args
        );
    }

    pub fn log_warn(domain: &str, args: std::fmt::Arguments<'_>) {
        let color = Self::log_level_to_color(LogLevel::Warning);
        let now = now!();
        println!(
            "\n(observatory-daemon:{}): {}-{}{}{} **: {}{}:{}:{}.{}{}: {}",
            *PID,
            domain,
            color,
            "WARNING",
            F_RESET,
            F_COL_LIGHT_BLUE,
            now.hours,
            now.minutes,
            now.seconds,
            now.milliseconds,
            F_RESET,
            args
        );
    }

    pub fn log_message(domain: &str, args: std::fmt::Arguments<'_>) {
        let color = Self::log_level_to_color(LogLevel::Message);
        let now = now!();
        println!(
            "(observatory-daemon:{}): {}-{}{}{}: {}{}:{}:{}.{}{}: {}",
            *PID,
            domain,
            color,
            "MESSAGE",
            F_RESET,
            F_COL_LIGHT_BLUE,
            now.hours,
            now.minutes,
            now.seconds,
            now.milliseconds,
            F_RESET,
            args
        );
    }

    pub fn log_info(domain: &str, args: std::fmt::Arguments<'_>) {
        if !G_MESSAGES_DEBUG.is_empty()
            && (!G_MESSAGES_DEBUG.contains(&domain.into())
                && !G_MESSAGES_DEBUG.contains(&"all".into()))
        {
            return;
        }

        let color = Self::log_level_to_color(LogLevel::Info);
        let now = now!();
        println!(
            "(observatory-daemon:{}): {}-{}{}{}: {}{}:{}:{}.{}{}: {}\n",
            *PID,
            domain,
            color,
            "INFO",
            F_RESET,
            F_COL_LIGHT_BLUE,
            now.hours,
            now.minutes,
            now.seconds,
            now.milliseconds,
            F_RESET,
            args
        );
    }

    pub fn log_debug(domain: &str, args: std::fmt::Arguments<'_>) {
        if !G_MESSAGES_DEBUG.is_empty()
            && (!G_MESSAGES_DEBUG.contains(&domain.into())
                && !G_MESSAGES_DEBUG.contains(&"all".into()))
        {
            return;
        }

        let color = Self::log_level_to_color(LogLevel::Debug);
        let now = now!();
        println!(
            "(observatory-daemon:{}): {}-{}{}{}: {}{}:{}:{}.{}{}: {}",
            *PID,
            domain,
            color,
            "INFO",
            F_RESET,
            F_COL_LIGHT_BLUE,
            now.hours,
            now.minutes,
            now.seconds,
            now.milliseconds,
            F_RESET,
            args
        );
    }

    const fn log_level_to_color(level: LogLevel) -> &'static str {
        match level {
            LogLevel::Error => "\x1b[1;31m",    /* red */
            LogLevel::Critical => "\x1b[1;35m", /* magenta */
            LogLevel::Warning => "\x1b[1;33m",  /* yellow */
            LogLevel::Message => "\x1b[1;32m",  /* green */
            LogLevel::Info => "\x1b[1;32m",     /* green */
            LogLevel::Debug => "\x1b[1;32m",    /* green */
        }
    }
}
