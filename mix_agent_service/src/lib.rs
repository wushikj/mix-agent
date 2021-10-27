use log::{info, warn};
use mix_agent_common::mix_config::{init_logger, MixConfig};
use mix_agent_common::{init_log, mix_config, post_log, GlobalConfig, LogLevel, Monitor};
use serde::{Deserialize, Serialize};

use std::ffi::OsStr;

#[cfg(target_os = "windows")]
use windows_service::service::{ServiceAccess, ServiceState, ServiceStatus};
#[cfg(target_os = "windows")]
use windows_service::service_manager::{ServiceManager, ServiceManagerAccess};
#[cfg(target_os = "windows")]
use windows_service::Error;

#[derive(Debug, Deserialize)]
pub struct ServiceAgentConfig {
    #[serde(default)]
    name: String,
    #[serde(default)]
    enabled: bool,
    #[serde(default = "default_cron")]
    cron: String,
    #[serde(default)]
    services: Vec<Target>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Target {
    name: String,
    desc: String,
    force_restart: bool,
}

impl Default for ServiceAgentConfig {
    fn default() -> Self {
        ServiceAgentConfig {
            name: "".to_string(),
            enabled: false,
            cron: default_cron(),
            services: vec![],
        }
    }
}

impl MixConfig for ServiceAgentConfig {
    fn new() -> Self {
        ServiceAgentConfig::default()
    }
}

///默认每10分钟执行一次
fn default_cron() -> String {
    "0 0/10 * * * ?".to_string()
}

#[derive(Default, Debug, Serialize)]
pub struct Service {
    pid: i32,
    is_exist: bool,
    force_restart: bool,
    status: String,
    name: String,
    display_name: String,
    remark: String,
}

impl Service {
    pub fn init() -> Service {
        init_logger(AGENT_NAME);
        info!("begin service data collect");
        Service::default()
    }
}

const AGENT_NAME: &str = "mix_agent_service";

#[cfg(not(windows))]
impl Monitor for Service {
    fn collect(&self) {
        warn!("The agent is only for windows, this platform({}) is not support!", std::env::consts::OS);
    }
}
#[cfg(target_os = "windows")]
impl Monitor for Service {
    fn collect(&self) {
        let global_config = mix_config::load::<GlobalConfig>("global");
        let agent_config = mix_config::load::<ServiceAgentConfig>(AGENT_NAME);
        info!("{:?}", global_config);
        info!("{:?}", agent_config);

        let mut tags = vec![];
        tags.push("agent-desc|windows服务监控".to_owned());

        Self::begin(&agent_config.cron, || {
            let mut result: Vec<Service> = vec![];
            if agent_config.services.len() == 0 {
                let log = init_log("agent", "60001:未配置要监控的windows服务", LogLevel::Warn, Box::new(tags.clone()), "", AGENT_NAME);
                post_log(&log);
            } else {
                for service in agent_config.services.iter() {
                    let service_name = &service.name;
                    let mut item = Service::default();
                    item.pid = -1;
                    item.name = service_name.to_string();
                    item.force_restart = service.force_restart;

                    let manager_access = ServiceManagerAccess::CONNECT;
                    let service_manager = ServiceManager::local_computer(None::<&str>, manager_access).unwrap();
                    let service_result = service_manager.open_service(service_name, ServiceAccess::QUERY_STATUS);

                    match service_result {
                        Ok(s) => {
                            let status = s.query_status().unwrap();
                            let state = status.current_state;

                            item.is_exist = true;
                            item.display_name = get_display_name(service_name, &service_manager);
                            item.status = "Unknown".to_string();

                            if state == ServiceState::Running {
                                item.pid = status.process_id.unwrap() as i32;
                                item.status = state.to_string();
                            }

                            if state == ServiceState::Stopped {
                                Service::restart_service(service, service_name, &mut item, service_manager)
                            }
                        }
                        Err(e) => {
                            item.pid = -1;
                            item.remark = format!("{:?}", e).to_string();
                            item.status = "Unknown".to_string();
                            println!("{:?}", e);
                        }
                    }

                    result.push(item);
                }

                println!("{:?}", result);
                let log = init_log("service-windows", "", LogLevel::Info, Box::new(tags.clone()), &result, AGENT_NAME);
                post_log(&log);
            }
        });
    }
}

trait Formatter {
    fn to_string(&self) -> String;
}

#[cfg(target_os = "windows")]
impl Formatter for ServiceState {
    fn to_string(&self) -> String {
        match *self {
            ServiceState::Stopped | ServiceState::StopPending => "Stopped".to_string(),
            ServiceState::Running => "Running".to_string(),
            ServiceState::Paused => "Paused".to_string(),
            _ => "Unknown".to_string(),
        }
    }
}

#[cfg(target_os = "windows")]
fn get_display_name(service_name: &String, service_manager: &ServiceManager) -> String {
    let query_config = service_manager.open_service(service_name, ServiceAccess::QUERY_CONFIG);
    if let Ok(s) = query_config {
        return String::from(s.query_config().unwrap().display_name.to_str().unwrap());
    }

    String::new()
}

#[cfg(target_os = "windows")]
impl Service {
    fn restart_service(service: &Target, service_name: &String, mut item: &mut Service, service_manager: ServiceManager) {
        if service.force_restart {
            let start_result = service_manager.open_service(service_name, ServiceAccess::START);
            match start_result {
                Ok(s) => {
                    s.start(&[] as &[&OsStr]);
                    let status_result = service_manager.open_service(service_name, ServiceAccess::QUERY_STATUS);
                    if let Ok(s) = status_result {
                        let status = s.query_status();
                        if let Ok(s) = status {
                            let state = s.current_state;
                            item.status = state.to_string();
                            item.remark = format!("检测到服务已停止，但通过`force-restart`配置，已对其重启(Stopped -> {:?})", state).to_string();
                            info!("{}", item.remark);
                        }
                    }
                }
                Err(_e) => {
                    item.remark = format!("{:?}", _e).to_string();
                    info!("{}", item.remark);
                }
            }
        } else {
            item.remark = "服务未启动".to_string();
        }
    }
}
