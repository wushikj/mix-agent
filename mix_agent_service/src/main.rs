use mix_agent_common::Monitor;
use mix_agent_service::Service;

mod lib;
fn main() {
    let service = Service::init();
    service.collect();
}
