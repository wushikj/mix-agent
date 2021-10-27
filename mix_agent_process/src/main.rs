use mix_agent_common::Monitor;
use mix_agent_process::Process;

mod lib;

fn main() {
    let process = Process::init();
    process.collect();
}
