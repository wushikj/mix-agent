use mix_agent_common::Monitor;
use mix_agent_disk::Disk;
//use sysinfo::SystemExt;

mod lib;
fn main() {
    let disk = Disk::init();
    disk.collect();
}
