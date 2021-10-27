use mix_agent_common::Monitor;
use mix_agent_memory::Memory;

mod lib;

fn main() {
    let memory = Memory::init();
    memory.collect();
}
