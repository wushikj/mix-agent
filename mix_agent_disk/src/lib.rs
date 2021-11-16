use log::info;
use mix_agent_common::mix_config::{init_logger, MixConfig};
use mix_agent_common::{init_log, mix_config, post_log, GlobalConfig, LogLevel, Monitor};
use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Debug)]
pub struct Disk {
    total_space: u64,
    available_space: u64,
    used_space: u64,
    usage: f32,
    file_system: String,
    name: String,
}

impl Disk {
    pub fn init() -> Disk {
        init_logger("mix_agent_disk");
        info!("begin disk data collect");
        Disk::default()
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct DiskAgentConfig {
    #[serde(default = "default_cron")]
    cron: String,
    #[serde(default = "default_file_system")]
    file_system: Vec<String>,
}
///默认每30分钟执行一次
fn default_cron() -> String {
    "0 0/30 * * * ?".to_string()
}

fn default_file_system() -> Vec<String> {
    vec!["apfs".to_string(), "ntfs".to_string(), "fat32".to_string(), "xfs".to_string(), "ext3".to_string(), "ext4".to_string()]
}

impl Default for DiskAgentConfig {
    fn default() -> Self {
        DiskAgentConfig {
            cron: default_cron(),
            file_system: default_file_system(),
        }
    }
}

impl MixConfig for DiskAgentConfig {
    fn new() -> Self {
        DiskAgentConfig::default()
    }
}
use systemstat::{Platform, System};



const AGENT_NAME: &str = "mix_agent_disk";

impl Monitor for Disk {
    fn collect(&self) {
        //let mut sys = System::new_all();
        let global_config = mix_config::load::<GlobalConfig>("global");
        let agent_config = mix_config::load::<DiskAgentConfig>(AGENT_NAME);

        info!("{:?}", global_config);
        info!("{:?}", agent_config);

        let sys = System::new();

        Self::begin(&agent_config.cron, || {
            match sys.mounts() {
                Ok(mounts) => {
                    let mut result: Vec<Disk> = vec![];
                    for mount in mounts.iter() {
                        //println!("{} ---{}---> {} (available {} of {} {:?})", mount.fs_mounted_from, mount.fs_type, mount.fs_mounted_on, mount.avail.as_u64(), mount.total.as_u64(), mount);
                        let file_system = &mount.fs_type;
                        let fstype = &agent_config.file_system;
                        if !fstype.contains(&file_system.to_lowercase()) {
                            continue;
                        }

                        let total_space = mount.total.as_u64();
                        let available_space = mount.avail.as_u64();
                        let used_space = mount.total.as_u64() - mount.avail.as_u64();

                        let disk_name = mount.fs_mounted_on.clone();
                        let current = Disk {
                            total_space,
                            available_space,
                            used_space,
                            usage: used_space as f32 / total_space as f32 * 100f32.round(),
                            file_system: file_system.to_string(),
                            name: disk_name,
                        };

                        result.push(current);
                    }

                    //print!("{:?}", result);
                    let mut tags = vec![];
                    tags.push("agent-desc|磁盘使用情况".to_owned());
                    tags.push("data-unit|字节".to_owned());
                    let log = init_log("disk", "", LogLevel::Info, Box::new(tags.clone()), &result, AGENT_NAME);
                    post_log(&log);
                }
                Err(x) => println!("\nMounts: error: {}", x),
            }
        });
    }
}
