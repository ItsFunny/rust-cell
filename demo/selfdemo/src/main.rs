#[macro_use]
extern crate log;
extern crate log4rs;

use backtrace::Backtrace;

use std::borrow::Borrow;
use std::fmt::Arguments;
use std::ptr::null;
use log::{error, Level, LevelFilter, Log, Metadata, Record};
use log4rs::append::console::ConsoleAppender;
use log4rs::append::file::FileAppender;
use log4rs::encode::pattern::PatternEncoder;
use log4rs::config::{Appender, Config, Logger, Root};

fn init_log() {
    let _ = log4rs::init_config(build_cfg()).unwrap();
    let ll = log4rs::Logger::new(build_cfg());
    ll.log(&Record::builder()
        .module_path(Some("asdd"))
        .file(Some("filepath"))
        .metadata(Metadata::builder().level(Level::Error).target("metadata").build())
        .build());
    ll.flush();
}

fn build_cfg() -> Config {
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
    config
}

pub struct Mylog {}

impl log::Log for Mylog {
    fn enabled(&self, metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        println!("{},{},{}", record.args(), record.target(), record.level())
    }

    fn flush(&self) {
        todo!()
    }
}


// fn main() {
//     let bt = Backtrace::new();
//     // do_some_work();
//
//     println!("{:?}", bt);
// }

fn main() {
    // init_log();


    log::set_max_level(log::LevelFilter::Trace);
    log::set_logger(&Mylog {});
    // env_logger::init();

    // trace!("A trace");
    // debug!("A debug");
    // info!("A info");
    info!("asddd {},",1);
    info!(target:"阿萨德理论框架", "File info");
    warn!("A warn");
    error!("A error");
    let bt = Backtrace::new();
    // do_some_work();
    println!("{:?}", bt);
}
