use log::info;
use mix_agent_common::mix_config::{init_logger, MixConfig};
use mix_agent_common::{get_batch_id, get_local_ip, get_timestamp_millis, init_log, mix_config, post_log, GlobalConfig, Identity, Log, LogLevel, Monitor, Priority, Source};
use serde::{Deserialize, Serialize};

use sysinfo::{System, SystemExt};

const AGENT_NAME: &str = "mix_agent_machine";

#[derive(Default, Debug, Serialize)]
pub struct Machine {
    os_name: String,
    os_arch: String,
    os_family: String,
    os_version: String,
    os_edition: String,
    os_core: String,
    ///内核版本
    os_core_version: String,
    boot_time: u64,
    up_time: u64,
    host_name: String,
    ip: String,
    machine_name: String,
}

impl Machine {
    #[allow(dead_code)]
    ///init cpu collect
    pub fn init() -> Machine {
        init_logger("mix_agent_machine");
        info!("begin machine data collect");
        Machine {
            ..Default::default()
        }
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct MachineAgentConfig {
    #[serde(default = "default_cron")]
    cron: String,
    machine_name: String,
}

///每天执行一次
fn default_cron() -> String {
    "0 0 0 * * ?".to_string()
}

impl Default for MachineAgentConfig {
    fn default() -> Self {
        MachineAgentConfig {
            cron: default_cron(),
            machine_name: "".to_string(),
        }
    }
}

impl MixConfig for MachineAgentConfig {
    fn new() -> Self {
        MachineAgentConfig::default()
    }
}

impl Monitor for Machine {
    fn collect(&self) {
        let sys = System::new_all();
        let global_config = mix_config::load::<GlobalConfig>("global");
        let agent_config = mix_config::load::<MachineAgentConfig>(AGENT_NAME);

        info!("{:?}", global_config);
        info!("{:?}", agent_config);

        let mut tags = vec![];
        tags.push("agent-desc|服务器信息采集".to_owned());

        Self::begin(&agent_config.cron, || {
            if agent_config.machine_name.is_empty() {
                let log = init_log("agent", "40001:未配置服务器名称`machine-name`", LogLevel::Warn, Box::new(tags.clone()), "", AGENT_NAME);
                post_log(&log);
            } else {
                let os_info = os_info::get();
                let machine = Machine {
                    os_name: std::env::consts::OS.to_string(),
                    os_arch: std::env::consts::ARCH.to_string(),
                    os_family: std::env::consts::FAMILY.to_string(),
                    os_version: os_info.version().to_string(),
                    os_edition: os_info.edition().unwrap_or("未知").to_string(),
                    os_core: sys_info::os_type().unwrap(),
                    os_core_version: sys_info::os_release().unwrap(),
                    boot_time: sys.boot_time() * 1000,
                    up_time: sys.uptime(),
                    host_name: sys_info::hostname().unwrap(),
                    ip: local_ipaddress::get().unwrap(),
                    machine_name: agent_config.machine_name.clone(),
                };

                let log = init_log("machine", "", LogLevel::Info, Box::new(tags.clone()), &machine, AGENT_NAME);
                post_log(&log);
            }
        })
    }
}
