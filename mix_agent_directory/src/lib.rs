use chrono::{DateTime, Local};
use log::info;
use mix_agent_common::mix_config::{init_logger, MixConfig};
use mix_agent_common::{get_batch_id, get_global_config, get_local_ip, get_timestamp_millis, init_log, mix_config, post_log, GlobalConfig, Identity, Log, LogLevel, Monitor, Priority, Source};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

const AGENT_NAME: &str = "mix_agent_directory";

#[derive(Debug, Serialize)]
pub struct Summary<'a> {
    root_path: &'a str,
    results: Vec<Directory>,
}

#[derive(Default, Deserialize, Debug, Serialize)]
pub struct Directory {
    #[serde(default)]
    path: String,
    #[serde(default)]
    created: i64,
    #[serde(default)]
    modified: i64,
    #[serde(default)]
    app_name: String,
    #[serde(default)]
    app_version: String,
    #[serde(default)]
    app_desc: String,
    #[serde(default)]
    link_man: String,
}

impl Directory {
    pub fn init() -> Directory {
        init_logger(AGENT_NAME);
        info!("begin directory data collect");
        Directory {
            ..Default::default()
        }
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
struct AppScanConfig {
    #[serde(default)]
    root_path: String,
    #[serde(default = "default_cron")]
    cron: String,
}

impl Default for AppScanConfig {
    fn default() -> Self {
        AppScanConfig {
            cron: default_cron(),
            root_path: "".to_string(),
        }
    }
}

///每天执行一次
fn default_cron() -> String {
    "0 0 0 * * ?".to_string()
}

impl MixConfig for AppScanConfig {
    fn new() -> Self {
        AppScanConfig::default()
    }
}

impl Monitor for Directory {
    fn collect(&self) {
        let global_config = get_global_config();
        let agent_config = mix_config::load::<AppScanConfig>(AGENT_NAME);
        info!("{:?}", global_config);
        info!("{:?}", agent_config);

        let mut tags = vec![];
        tags.push("agent-desc|目录信息采集".to_owned());
        Self::begin(&agent_config.cron, || {
            let mut summary = Summary {
                root_path: "",
                results: vec![],
            };
            let _content = String::new();
            let path = Path::new(&agent_config.root_path);
            if !path.exists() {
                let log = init_log("agent", "50001:未配置采集根目录`root-path`", LogLevel::Warn, Box::new(tags.clone()), "", AGENT_NAME);
                post_log(&log);
            } else {
                let paths = Path::read_dir(path).unwrap();
                let mut results: Vec<Directory> = vec![];
                for path in paths {
                    let p = path.as_ref().unwrap();
                    let metadata = p.metadata().unwrap();

                    if metadata.is_file() {
                        continue;
                    }

                    let current_path = p.path().into_os_string().into_string().unwrap();

                    let mut created_time = 0;
                    if cfg!(target_os = "macos") || cfg!(target_os = "windows") {
                        let created: DateTime<Local> = metadata.created().unwrap().into();
                        created_time = created.timestamp_millis();
                    }

                    let modified: DateTime<Local> = metadata.modified().unwrap().into();
                    let mut app_scan = Directory {
                        path: current_path.clone(),
                        created: created_time,
                        modified: modified.timestamp_millis(),
                        app_name: "".to_string(),
                        app_version: "".to_string(),
                        app_desc: "".to_string(),
                        link_man: "".to_string(),
                    };

                    let app_info_path = Path::new(current_path.as_str()).join("app_info.yml");
                    if app_info_path.exists() {
                        let contents = fs::read_to_string(app_info_path).unwrap();
                        let app_info: Directory = serde_yaml::from_str::<Directory>(contents.as_str()).unwrap_or(Directory::default());

                        app_scan.app_name = app_info.app_name;
                        app_scan.app_desc = app_info.app_desc;
                        app_scan.app_version = app_info.app_version;
                        app_scan.link_man = app_info.link_man;
                    }

                    results.push(app_scan);
                }

                summary.root_path = path.to_str().unwrap();
                summary.results = results;

                let log = init_log("directory", "", LogLevel::Info, Box::new(tags.clone()), &summary, AGENT_NAME);
                post_log(&log);
            }
        })
    }
}
