use log::{debug, error, info, warn};
use mix_agent_common::mix_config::{init_logger, MixConfig};
use mix_agent_common::{get_batch_id, get_local_ip, get_timestamp_millis, init_log, mix_config, post_log, GlobalConfig, Identity, Log, LogLevel, Monitor, Priority, Source};
use serde::{Deserialize, Serialize};

use sysinfo::{ProcessorExt, System, SystemExt};

#[derive(Default, Deserialize, Serialize, Debug)]
pub struct Cpu {
    time: i64,
    usage: f32,
}

impl Cpu {
    ///init cpu collect
    pub fn init() -> Cpu {
        init_logger("mix_agent_cpu");
        info!("begin cpu data collect");
        Cpu::default()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CpuAgentConfig {
    #[serde(default = "default_cron")]
    cron: String,
}

impl Default for CpuAgentConfig {
    fn default() -> Self {
        CpuAgentConfig {
            cron: default_cron(),
        }
    }
}

///默认每5秒执行一次
fn default_cron() -> String {
    "*/5 * * * * ?".to_string()
}

impl MixConfig for CpuAgentConfig {
    fn new() -> Self {
        CpuAgentConfig::default()
    }
}

const AGENT_NAME: &str = "mix_agent_cpu";
impl Monitor for Cpu {
    fn collect(&self) {
        let mut sys = System::new_all();
        let global_config = mix_config::load::<GlobalConfig>("global");
        let agent_config = mix_config::load::<CpuAgentConfig>(AGENT_NAME);
        info!("{:?}", global_config);
        info!("{:?}", agent_config);
        let mut result: Vec<Cpu> = vec![];
        Self::begin(&agent_config.cron, || {
            //sys.refresh_all();
            sys.refresh_cpu();
            let used_cpu = sys.global_processor_info().cpu_usage();
            let cpu = Cpu {
                time: chrono::offset::Local::now().timestamp_millis(),
                usage: used_cpu,
            };

            #[cfg(debug_assertions)]
            println!("{:?}", cpu);

            result.push(cpu);

            if result.len() >= 10 {
                let mut tags = vec![];
                tags.push("agent-desc|cpu使用率".to_owned());
                let log = init_log("cpu", "", LogLevel::Warn, Box::new(tags), &result, AGENT_NAME);
                post_log(&log);

                result.clear();
            }
        });
    }
}
