use mix_agent_common::Monitor;
use mix_agent_cpu::Cpu;

fn main() {
    let cpu = Cpu::init();
    cpu.collect();
}
