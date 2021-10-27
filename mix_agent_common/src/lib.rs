pub mod mix_config;
pub mod mix_scheduler;

use crate::mix_config::MixConfig;

//use job_scheduler::{Job, JobScheduler};
use log::{error, info, warn};
use reqwest::blocking::{Client, Response};
use reqwest::Error;
use serde::Deserialize;
use serde::Serialize;
use serde_json;

use std::thread;
use std::time::Duration;

#[derive(Debug, Serialize, Deserialize)]
pub enum LogLevel {
    Info,
    Warn,
    Error,
    Debug,
    Trace,
}

impl LogLevel {
    pub fn to_lower(&self) -> String {
        match *self {
            LogLevel::Info => "info".to_string(),
            LogLevel::Warn => "warn".to_string(),
            LogLevel::Error => "error".to_string(),
            LogLevel::Debug => "debug".to_string(),
            LogLevel::Trace => "trace".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Priority {
    High,
    Middle,
    Low,
}

impl Priority {
    pub fn to_lower(&self) -> String {
        use Priority::*;
        match *self {
            High => "high".to_string(),
            Middle => "middle".to_string(),
            Low => "low".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Log<T> {
    pub batch_id: String,
    pub identity: Identity,
    pub time: i64,
    pub level: String,
    pub tags: Box<Vec<String>>,
    pub category: String,
    pub content: String,
    pub raw_data: T,
    pub remark: String,
    pub priority: String,
    pub env: String,
    pub source: Source,
}

pub fn init_log<T: Serialize>(category: &str, content: &str, level: LogLevel, tags: Box<Vec<String>>, data: T, agent_name: &str) -> Log<T> {
    let global_config = get_global_config();
    let batch_id = get_batch_id();
    let target_ip = get_local_ip();
    Log {
        batch_id,
        identity: Identity {
            customer_id: global_config.customer_id,
            project_id: global_config.project_id,
            target_ip,
        },
        time: get_timestamp_millis(),
        level: level.to_lower(),
        tags,
        category: category.to_string(),
        content: content.to_string(),
        raw_data: data,
        remark: "".to_string(),
        priority: Priority::Low.to_lower(),
        env: global_config.env,
        source: Source {
            name: agent_name.to_string(),
            ..Default::default()
        },
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Identity {
    pub customer_id: String,
    pub project_id: String,
    pub target_ip: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Source {
    pub from: String,
    pub name: String,
    pub version: String,
    pub lang: String,
    pub ip: String,
}

impl<'a> Default for Source {
    fn default() -> Self {
        let source_ip = get_local_ip();
        Source {
            from: "agent".to_string(),
            name: "".to_string(),
            version: "v1.0.0".to_string(),
            lang: "rust".to_string(),
            ip: source_ip,
        }
    }
}

pub trait Monitor {
    fn collect(&self) {}

    fn begin<T: FnMut()>(cron: &String, mut action: T) {
        //运行一次，不用等到cron触发
        action();

        let scheduler_result = cron.parse::<Schedule>();
        match scheduler_result {
            Ok(scheduler) => {
                let mut sched = JobScheduler::new();
                sched.add(Job::new(scheduler, || {
                    action();
                }));
                loop {
                    sched.tick();
                    thread::sleep(Duration::from_millis(500))
                }
            }
            Err(e) => {
                error!("{}", e)
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct GlobalConfig {
    pub customer_id: String,
    pub project_id: String,
    pub mix_endpoint: String,
    pub mix_endpoint_key: String,
    #[serde(default = "default_env")]
    pub env: String,
    #[serde(default = "default_timeout")]
    pub timeout: u64,
    #[serde(default = "default_print_log_json")]
    pub print_log_json: bool,
    #[serde(default = "default_push_log")]
    pub push_log: bool,
}
fn default_env() -> String {
    "dev".to_string()
}

fn default_timeout() -> u64 {
    5000
}

fn default_print_log_json() -> bool {
    true
}

fn default_push_log() -> bool {
    true
}

impl Default for GlobalConfig {
    fn default() -> Self {
        GlobalConfig {
            customer_id: "".to_string(),
            project_id: "".to_string(),
            mix_endpoint: "http://mix.wushiai.com".to_string(),
            mix_endpoint_key: "".to_string(),
            env: default_env(),
            timeout: default_timeout(), //毫秒
            print_log_json: default_print_log_json(),
            push_log: default_push_log(),
        }
    }
}

impl MixConfig for GlobalConfig {
    fn new() -> Self {
        GlobalConfig::default()
    }
}

pub fn get_batch_id() -> String {
    chrono::Local::now().format("%Y%m%d%H%M%S%f").to_string()
}

pub fn get_local_ip() -> String {
    match local_ipaddress::get() {
        None => "无法解析IP".to_owned(),
        Some(ip) => ip.to_owned(),
    }
}

pub fn get_timestamp_millis() -> i64 {
    chrono::Local::now().timestamp_millis()
}

pub fn get_global_config() -> GlobalConfig {
    mix_config::load::<GlobalConfig>("global")
}
use crate::mix_scheduler::{Job, JobScheduler};
use cron::Schedule;
use lazy_static::lazy_static;
use std::sync::Mutex;
lazy_static! {
    static ref mutex_client: Mutex<Client> = Mutex::new(reqwest::blocking::Client::new());
}

pub fn post_log<T: Serialize>(log: &Log<T>) {
    //let client = reqwest::blocking::Client::new();
    let global_config = mix_config::load::<GlobalConfig>("global");

    if global_config.mix_endpoint == "" {
        error!("mix_endpoint配置无效")
    }

    let json = serde_json::to_string(&log);
    if global_config.print_log_json {
        info!("{}", json.expect("解析失败"));
    }
    let mut mix_endpoint = global_config.mix_endpoint;
    mix_endpoint.push_str("/mix/api/v1/monitor/collect");
    //info!("{}", mix_endpoint);

    let client_guard = mutex_client.lock();
    match client_guard {
        Ok(client) => {
            if global_config.push_log {
                let res = client.post(mix_endpoint.as_str()).json(log).timeout(Duration::from_millis(global_config.timeout)).send();
                response_handler(log.category.clone(), res);
            }
        }
        Err(e) => {
            error!("get reqwest client error:{}", e.to_string());
        }
    }
}

fn response_handler(category: String, res: Result<Response, Error>) {
    match res {
        Ok(res) => {
            if res.status().is_success() {
                info!("日志提交成功, 类型:{}, 响应:{}", category, res.text().expect(""));
            } else {
                warn!("日志提交异常, 类型:{}, 响应:{}", category, res.text().expect(""));
            }
        }
        Err(e) => {
            error!("日志提交失败, 类型:{}, 错误:{}", category, e.to_string());
        }
    }
}

pub trait StripBom {
    fn strip_bom(&self) -> &str;
}

impl StripBom for str {
    fn strip_bom(&self) -> &str {
        if self.starts_with("\u{feff}") {
            &self[3..]
        } else {
            &self[..]
        }
    }
}

impl StripBom for String {
    fn strip_bom(&self) -> &str {
        &self[..].strip_bom()
    }
}

#[test]
pub fn test_get_batch_id() {
    let batch_id = get_batch_id();
    println!("{}", batch_id);
}

#[test]
pub fn test_get_local_ip() {
    let local_ip = get_local_ip();
    println!("{}", local_ip);
}
