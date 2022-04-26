use std::intrinsics::fabsf32;
use cell_base_common::events::IEventResult;
use crate::consumer::LogConsumer::ILogEventConsumer;
use crate::event::event::ILogEvent;
use crate::filter::LogFilter;
use crate::logenums::LogTypeEnums;
use crate::loglevel::LogLevel;
use crate::module::ModuleTrait;

pub struct CellLogger {
    log_level: LogLevel,
    modulee: ModuleTrait,
    filter: LogFilter,
}

impl CellLogger {
    pub fn log(&self, module: &'static dyn ModuleTrait, l: LogLevel, enu: LogTypeEnums) {}

    pub fn log_able(&self, l: LogLevel) -> bool {
        // TODO ,修改为pipeline
        if self.log_level.is_bigger(l) {
            false
        }

        true
    }

    pub fn get_log_consumers(m: &'static dyn ModuleTrait, l: LogLevel, typeE: LogTypeEnums) -> Vec<ILogEventConsumer<ILogEvent, IEventResult>> {

    }
}