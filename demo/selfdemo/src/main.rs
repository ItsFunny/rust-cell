#[macro_use]
extern crate log;
extern crate log4rs;

use std::ptr::null;
use log::{error, LevelFilter, Log, Record};
use log4rs::append::console::ConsoleAppender;
use log4rs::append::file::FileAppender;
use log4rs::encode::pattern::PatternEncoder;
use log4rs::config::{Appender, Config, Logger, Root};

fn init_log() {
    let stdout = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new("[Console] {d} - {l} -{t} - {m}{n}")))
        .build();

    let file = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("[File] {d} - {l} - {t} - {m}{n}")))
        .build("log/test.log")
        .unwrap();

    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .appender(Appender::builder().build("file", Box::new(file)))
        .logger(Logger::builder()
            .appender("file")
            .additive(false)
            .build("app", LevelFilter::Info))
        .build(Root::builder().appender("stdout").build(LevelFilter::Info))
        .unwrap();

    let _ = log4rs::init_config(config).unwrap();

    let ll=log4rs::Logger::new(config);
    ll.log(&Record::builder()
        .build())
}


fn main() {
    init_log();

    trace!("A trace");
    debug!("A debug");
    info!("A info");
    info!(target:"app", "File info");
    warn!("A warn");
    error!("A error");
}
