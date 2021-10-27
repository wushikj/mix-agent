use mix_agent_common::Monitor;
use mix_agent_directory::Directory;

mod lib;

fn main() {
    let app_scan = Directory::init();
    app_scan.collect();
}
