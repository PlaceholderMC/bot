use std::sync::OnceLock;

use log::trace;
use regex::Regex;

// in future, we can add extra data to display in log analysis like Java version
pub enum Info {
	Game,
	Launcher,
}

pub fn find(log: &str) -> Option<Info> {
	if looks_like_launcher_log(log) {
		// launcher logs can sometimes seem like a game log
		Some(Info::Launcher)
	} else if looks_like_game_log(log) {
		Some(Info::Game)
	} else {
		None
	}
}

fn looks_like_launcher_log(log: &str) -> bool {
	static QT_LOG_REGEX: OnceLock<Regex> = OnceLock::new();

	trace!("Guessing whether log is launcher log");

	let qt_log = QT_LOG_REGEX.get_or_init(|| Regex::new(r"\d+\.\d{3} [CDFIW] \|").unwrap());
	qt_log.is_match(log)
}

fn looks_like_game_log(log: &str) -> bool {
	static LOG4J_REGEX: OnceLock<Regex> = OnceLock::new();

	trace!("Guessing whether log is Minecraft log");

	if log.contains("Prism Launcher version: ") || log.contains("Minecraft process ID: ") {
		return true;
	}

	// present in almost every Minecraft version
	if log.contains("Setting user: ") || log.contains("Minecraft Version: ") {
		return true;
	}

	if log.contains("Exception in thread ")
		|| log.contains("Exception: ")
		|| log.contains("Error: ")
		|| log.contains("Throwable: ")
		|| log.contains("Caused by: ")
	{
		return true;
	}

	if log.contains("org.prismlauncher.EntryPoint.main(EntryPoint.java")
		|| log.contains("java.lang.Thread.run(Thread.java")
	{
		return true;
	}

	let log4j = LOG4J_REGEX.get_or_init(|| {
		Regex::new(r"\[\d{2}:\d{2}:\d{2}\] \[.+?/(FATAL|ERROR|WARN|INFO|DEBUG|TRACE)\] ").unwrap()
	});

	if log4j.is_match(log) {
		return true;
	}

	if log.contains("[INFO]")
		|| log.contains("[CONFIG]")
		|| log.contains("[FINE]")
		|| log.contains("[FINER]")
		|| log.contains("[FINEST]")
		|| log.contains("[SEVERE]")
		|| log.contains("[STDERR]")
		|| log.contains("[WARNING]")
		|| log.contains("[DEBUG]")
	{
		return true;
	}

	false
}
