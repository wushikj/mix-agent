use config::{Config, File, FileFormat};
use log::{error, LevelFilter};
use log4rs::append::console::ConsoleAppender;
use log4rs::append::rolling_file::policy::compound::roll::fixed_window::FixedWindowRoller;
use log4rs::append::rolling_file::policy::compound::trigger::size::SizeTrigger;
use log4rs::append::rolling_file::policy::compound::CompoundPolicy;
use log4rs::append::rolling_file::RollingFileAppender;
use log4rs::config::Config as logConfig;
use log4rs::config::{Appender, Root};
use log4rs::encode::pattern::PatternEncoder;
use serde::Deserialize;
use std::env;
use std::path::Path;

pub trait MixConfig {
    fn new() -> Self;
}

pub fn init_logger(name: &str) {
    let config_dir = get_current_dir();
    env::set_current_dir(&config_dir).expect("error : set current directory ");

    let tpl = "{d} {h({l})} {M}[{L}] - {m}{n}";
    let stdout = ConsoleAppender::builder().encoder(Box::new(PatternEncoder::new(tpl))).build();

    let window_size = 5; // log0, log1, log2
    let mut pattern = format!("log/{}", name);
    pattern.push_str("_{}.gz");
    let fixed_window_roller = FixedWindowRoller::builder().build(pattern.as_str(), window_size).unwrap();
    let size_limit = 10 * 1024 * 1024; // 10mb as max log file size to roll
    let size_trigger = SizeTrigger::new(size_limit);
    let compound_policy = CompoundPolicy::new(Box::new(size_trigger), Box::new(fixed_window_roller));
    let file = RollingFileAppender::builder().encoder(Box::new(PatternEncoder::new(tpl))).build(format!("log/{}.log", name), Box::new(compound_policy)).unwrap();

    let config = logConfig::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .appender(Appender::builder().build("file", Box::new(file)))
        .build(Root::builder().appender("stdout").appender("file").build(LevelFilter::Info))
        .unwrap();
    let _handle = log4rs::init_config(config);
}

pub fn get_current_dir() -> String {
    let path = env::current_exe().unwrap().into_os_string().into_string().expect("error : get current work directory.");
    let work_dir = Path::new(&path).parent().expect("error : get parent directory of current work directory");
    String::from(work_dir.to_str().expect("error : Path to str"))
}

pub fn load<'a, T: Deserialize<'a> + MixConfig>(config_name: &str) -> T where {
    let mut exe_path = get_current_dir();
    let mut c = Config::default();
    exe_path.push_str("/config/");
    exe_path.push_str(config_name);
    exe_path.push_str(".yml");

    let final_path = exe_path.as_str();
    if Path::new(final_path).exists() {
        c.merge(File::new(final_path, FileFormat::Yaml));
        let config = c.try_into::<T>();
        return match config {
            Ok(c) => c,
            Err(e) => {
                error!("{:?}", e.to_string());
                T::new()
            }
        };
    }

    return T::new();
}
