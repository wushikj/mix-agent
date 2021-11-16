use log::info;
use mix_agent_common::mix_config::{init_logger, MixConfig};
use mix_agent_common::{init_log, mix_config, post_log, GlobalConfig, LogLevel, Monitor};
use serde::{Deserialize, Serialize};
use sysinfo::{ProcessExt, System, SystemExt};

const AGENT_NAME: &str = "mix_agent_process";

#[derive(Debug, Serialize, Default)]
pub struct Process {
    pid: i32,
    name: String,
    is_exist: bool,
    status: String,
    start_time: i64,
}

#[derive(Debug, Deserialize)]
pub struct ProcessAgentConfig {
    #[serde(default)]
    name: String,
    #[serde(default)]
    enabled: bool,
    #[serde(default = "default_cron")]
    cron: String,
    #[serde(default)]
    target: Vec<Target>,
}

#[derive(Debug, Deserialize)]
struct Target {
    #[serde(default)]
    name: String,
    #[serde(default)]
    remark: String,
}

impl Process {
    pub fn init() -> Process {
        init_logger(AGENT_NAME);
        info!("begin process data collect");
        Process::default()
    }
}

///默认每10分钟执行一次
fn default_cron() -> String {
    "0 0/10 * * * ?".to_string()
}

impl Default for ProcessAgentConfig {
    fn default() -> Self {
        ProcessAgentConfig {
            name: "".to_string(),
            enabled: false,
            cron: default_cron(),
            target: vec![],
        }
    }
}

impl MixConfig for ProcessAgentConfig {
    fn new() -> Self {
        ProcessAgentConfig::default()
    }
}

impl Monitor for Process {
    fn collect(&self) {
        let sys = System::new_all();
        let global_config = mix_config::load::<GlobalConfig>("global");
        let agent_config = mix_config::load::<ProcessAgentConfig>(AGENT_NAME);
        info!("{:?}", global_config);
        info!("{:?}", agent_config);
        let mut tags = vec![];
        tags.push("agent-desc|进程监控".to_owned());
        Self::begin(&agent_config.cron, || {
            let mut result: Vec<Process> = vec![];

            if agent_config.target.len() == 0 {
                let log = init_log("agent", "70001:未配置要监控的目标进程", LogLevel::Warn, Box::new(tags.clone()), "", AGENT_NAME);
                post_log(&log);
            } else {
                let process = sys.processes();
                for target in agent_config.target.iter() {
                    let current = &target;
                    let mut item = Process::default();
                    item.name = current.name.clone();
                    for (pid, process) in process.iter() {
                        #[cfg(debug_assertions)]
                        info!("{} {:?} {}", pid, process.name(), process.cmd().len());

                        let exe_path = process.exe().as_os_str().to_str().unwrap();
                        if process.name() == target.name || exe_path.contains(&current.name) {
                            item.pid = *pid as i32;
                            item.start_time = process.start_time() as i64 * 1000;
                            item.is_exist = true;
                            item.status = get_process_status(process.status().as_str());

                            #[cfg(debug_assertions)]
                            info!("{} {} {} {}", pid, process.status(), process.start_time(), item.start_time);
                            #[cfg(debug_assertions)]
                            info!("{:?}", process);

                            break;
                        } else {
                            item.pid = -1;
                            item.status = "Unknown".to_string();
                        }
                    }

                    result.push(item);
                }

                let log = init_log("process", "", LogLevel::Info, Box::new(tags.clone()), &result, AGENT_NAME);
                post_log(&log);
            }
        });
    }
}

fn get_process_status(status: &str) -> String {
    match status {
        "Idle" => "Idle".to_string(),
        "Runnable" => "Running".to_string(),
        "Sleeping" => "Sleeping".to_string(),
        "Stopped" => "Stopped".to_string(),
        "Zombie" => "Zombie".to_string(),
        "Unknown" => "Unknown".to_string(),
        _ => "_".to_string(),
    }
}
