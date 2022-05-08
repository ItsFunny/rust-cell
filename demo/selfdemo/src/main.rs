#[macro_use]
extern crate log;
extern crate log4rs;

use backtrace::Backtrace;

use std::borrow::{Borrow, BorrowMut};
use std::fmt::Arguments;
use std::ptr::null;
use std::{thread, time};
use std::cell::RefCell;
use std::collections::HashMap;
use std::thread::LocalKey;
use log::{error, Level, LevelFilter, Log, Metadata, Record};
use log4rs::append::console::ConsoleAppender;
use log4rs::append::file::FileAppender;
use log4rs::encode::pattern::PatternEncoder;
use log4rs::config::{Appender, Config, Logger, Root};
use term_painter::{Color, ToStyle};

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

thread_local! {
    // Note lack of pub
    static FOO: RefCell<usize> = RefCell::new(0);
}
struct Bar {
    // Visibility here changes what can see `foo`.
    foo: &'static LocalKey<RefCell<usize>>,
    // Rest of your data.
}

impl Bar {
    fn constructor() -> Self {
        Self {
            foo: &FOO,
            // Rest of your data.
        }
    }
}


fn main() {
    let mut a = "asd";
    // p(a);
    // init_log();
    // test_thread_local();
    // let b = Bar::constructor();
    // b.foo.with(|x| x.replace(123));
    // println!("{:?}", b.foo)

    testlog();

    // testH();
}


pub struct thread_local_demo {}

impl thread_local_demo {
    thread_local! {
 // Could add pub to make it public to whatever Foo already is public to.
        static FOO: RefCell<usize> = RefCell::new(0);
    }
}

fn test_thread_local() {
    // thread_local_demo::FOO.with(|x| println!(":%?", x)
    // )
}

pub struct A {
    pub m: HashMap<i32, i32>,
}

fn testH() {
    let mut a = &mut A { m: HashMap::<i32, i32>::new() };
    a.m.insert(4, 5);
    let mut m = &mut a.m;
    m.insert(1, 2);
    // a.m = HashMap::<i32, i32>::new();
    a.m.drain();
    // a.m.insert(1, 2);
    // let mut m = a.m.borrow_mut();
    // m.insert(1, 2);
    // a.m.insert(1, 2);
    println!("{:?}", a.m)
}

// fn test_env_log() {
//     let env = env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "trace");
//     let mut builder = env_logger::Builder::from_env(env);
//     println!("builder = {:?}", builder);
//     builder
//         .format(|buf, record| {
//             let level = { buf.default_styled_level(record.level()) };
//             writeln!(buf, "{}", format_args!("{:>5}", level));
//             writeln!(buf, " {}", &record.args())
//         })
//         .init();
// }

#[cfg(not(target_os = "windows"))]
pub const DIMM_COLOR: Color = Color::Custom(123);
#[cfg(target_os = "windows")]
pub const DIMM_COLOR: Color = Color::White;

fn test_painter() {
    let g = Color::Green;
}

// fn hashed_color(item: &str) -> Color {
//     match item.bytes().fold(42u16, |c: u32, x: u32| (c ^ x) as u16) {
//         c @ 0...1 => Color::Custom((c + 2) as u32),
//         c @ 16...21 => Color::Custom((c + 6) as u32),
//         c @ 52...55 | c @ 126...129 => Color::Custom((c + 4) as u32),
//         c @ 163...165 | c @ 200...201 => Color::Custom((c + 3) as u32),
//         c @ 207 => Color::Custom((c + 1) as u32),
//         c @ 232...240 => Color::Custom((c + 9) as u32),
//         c => Color::Custom(c as u32),
//     }
// }

fn testlog() {
    log::set_max_level(log::LevelFilter::Trace);
    log::set_logger(&Mylog {});
    // env_logger::init();

    // trace!("A trace");
    // debug!("A debug");
    // info!("A info");
    // let f = file!();
    // let v = module_path!();
    // println!("{},{}", v, f);
    // info!("asddd {},",1);
    info!(target:"阿萨德理论框架", "File info");
    warn!("A warn");
    error!("A error");
    let bt = Backtrace::new();
    let frames = bt.frames();
    for v in frames {
        println!("{:?}", v);
        for s in v.symbols() {
            println!("{},{:?},{}", s.name().unwrap(), s.addr(), s.lineno().unwrap())
        }
    }
    // do_some_work();
    println!("{:?}", bt);
}


#[cfg(test)]
mod tests {
    use term_painter::{Color, ToStyle};
    use crate::{DIMM_COLOR};

    #[test]
    fn test_color() {
        let a = Color::Red.paint("aaaa");
        println!("{}", Color::Red.paint("aaaa"));
        println!("{}", Color::Yellow.paint("aaaa"));
        println!("{}", Color::Black.paint("aaaa"));
    }

    #[test]
    fn test_asd() {}
}