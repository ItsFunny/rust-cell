// #[macro_export]
// macro_rules! cinfo {
//     ($m:expr,$e:expr) => {
//         $crate::log_impl!($m,$crate::LogLevel::Info,($e));
//     };
//
//     ($m:expr,$e:expr, $($rest:tt)*) => {
//         $crate::log_impl!($m,$crate::LogLevel::Info,($e) $($rest)*);
//     };
// }
//
// #[macro_export]
// macro_rules! cdebug {
//     ($m:expr,$e:expr) => {
//         $crate::log_impl!($m,$crate::LogLevel::Debug,($e));
//     };
//
//     ($m:expr,$e:expr, $($rest:tt)*) => {
//         $crate::log_impl!($m,$crate::LogLevel::Debug,($e) $($rest)*);
//     };
// }
//
//
// #[macro_export]
// macro_rules! cwarn {
//     ($m:expr,$e:expr) => {
//         $crate::log_impl!($m,$crate::LogLevel::Warn,($e));
//     };
//
//     ($m:expr,$e:expr, $($rest:tt)*) => {
//         $crate::log_impl!($m,$crate::LogLevel::Warn,($e) $($rest)*);
//     };
// }
//
// #[macro_export]
// macro_rules! cerror {
//     ($m:expr,$e:expr) => {
//         $crate::log_impl!($m,$crate::LogLevel::Error,($e));
//     };
//
//     ($m:expr,$e:expr, $($rest:tt)*) => {
//         $crate::log_impl!($m,$crate::LogLevel::Error,($e) $($rest)*);
//     };
// }
//
// #[macro_export]
// #[doc(hidden)]
// macro_rules! log_impl {
//     ($m:expr,$lvl:expr,($($e:expr),*)) => {
//         crate::log4rs::DEFAULT_LOGGER.log($m,$lvl,file!(),line!(),format!("{}",format!($($e),*)).as_str())
//     };
//
//     ($m:expr,$lvl:expr,($($e:expr),*) { $($key:ident : $value:expr),* }) => {
//         let mut msg=format!("{}",format!($($e),*));
//         msg.push_str(",");
//         $(
//             msg.push_str(format!("{}={:?},",stringify!($key), $value).as_str());
//         )*
//         crate::log4rs::DEFAULT_LOGGER.log($m,$lvl,file!(),line!(),msg.as_str())
//     };
//
//     ($m:expr,$lvl:expr,($($e:expr),*) { $($key:ident : $value:expr,)* }) => {
//         $crate::log_impl!($m,$lvl,($($e),*) { $($key : $value),* });
//     };
//
//     ($m:expr,$lvl:expr,($($e:expr),*) $arg:expr) => {
//         $crate::log_impl!($m,$lvl,($($e,)* $arg));
//     };
//
//     ($m:expr,$lvl:expr,($($e:expr),*) $arg:expr, $($rest:tt)*) => {
//         $crate::log_impl!($m,$lvl,($($e,)* $arg) $($rest)*);
//     };
// }
//
//
//
// #[cfg(test)]
// mod tests {
//     use crate::{CellModule, LogLevel, module, set_global_level_info};
//     use crate::log4rs::DEFAULT_LOGGER;
//
//     #[test]
//     fn test_macros() {
//         static M: &CellModule = &module::CellModule::new(1, "MACROS", &LogLevel::Info);
//         cinfo!(M,"hello",);
//         cdebug!(M,"hello {}", "cats");
//         cwarn!(M,"hello {}", "cats",);
//         cerror!(M,"hello {}", "cats", {
//             cat_1: "chashu",
//             cat_2: "nori",
//         });
//     }
// }
