use log::info;
use mix_agent_common::mix_config::{init_logger, MixConfig};
use mix_agent_common::{init_log, mix_config, post_log, GlobalConfig, LogLevel, Monitor};
use serde::{Deserialize, Serialize};
use sysinfo::{System, SystemExt};

#[derive(Debug, Serialize, Default)]
pub struct Memory {
    total_memory: u64,
    available_memory: u64,
    free_memory: u64,
    used_memory: u64,
    usage: f32,
}

impl Memory {
    ///init memory collect
    pub fn init() -> Memory {
        init_logger("mix_agent_memory");
        info!("begin memory data collect");
        Memory::default()
    }
}

#[derive(Deserialize, Debug)]
pub struct MemoryAgentConfig {
    #[serde(default = "default_cron")]
    cron: String,
}

///默认每15秒执行一次
fn default_cron() -> String {
    "*/10 * * * * ?".to_string()
}

impl Default for MemoryAgentConfig {
    fn default() -> Self {
        MemoryAgentConfig {
            cron: default_cron(),
        }
    }
}

impl MixConfig for MemoryAgentConfig {
    fn new() -> Self {
        MemoryAgentConfig::default()
    }
}

const AGENT_NAME: &str = "mix_agent_memory";
impl Monitor for Memory {
    fn collect(&self) {
        let mut sys = System::new_all();
        let global_config = mix_config::load::<GlobalConfig>("global");
        let agent_config = mix_config::load::<MemoryAgentConfig>(AGENT_NAME);
        info!("{:?}", global_config);
        info!("{:?}", agent_config);
        Self::begin(&agent_config.cron, || {
            sys.refresh_all();
            let total_memory = sys.total_memory();
            let available_memory = sys.available_memory(); //.available_memory();
            let free_memory = sys.free_memory();
            let used_memory = sys.used_memory();
            let usage = used_memory as f32 / total_memory as f32 * 100f32;

            let result = Memory {
                total_memory,
                available_memory,
                free_memory,
                used_memory,
                usage,
            };

            let mut tags = vec![];
            tags.push("agent-desc|内存使用率".to_owned());
            tags.push("data-unit|千字节".to_owned());
            let log = init_log("memory", "", LogLevel::Info, Box::new(tags.clone()), &result, AGENT_NAME);
            post_log(&log);
        })
    }
}
