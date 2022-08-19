
use std::env;

mod arg_parse;
mod container;
mod network;
mod cgroup;
mod dbus_systemd;

const BUSY_BOX_CONTAINER_ID: &str = "busybox";

fn main() {

    // collect and process the arguments
    let args: Vec<String> = env::args().collect();
    let bb_command =  arg_parse::process_options(&args);

    // create a chroot and cgroup container and exec the entrypoint
    if let Some(i) = bb_command {
        container::start_container(BUSY_BOX_CONTAINER_ID, &i);
    }

    // clean-up


}
