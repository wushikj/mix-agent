use mix_agent_api::Result;
use mix_agent_common::Monitor;

mod lib;

fn main() {
    let api = Result::init();
    api.collect();
}
