use mix_agent_common::Monitor;
use mix_agent_machine::Machine;

mod lib;

fn main() {
    let machine = Machine::init();
    machine.collect();
}
