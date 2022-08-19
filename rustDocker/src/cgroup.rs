use dbus::{
    arg::{self, Variant},
    blocking::Connection,
};

use std::time::Duration;

// set limit to how much memory it can use
const CONT_MEM_LIMIT:u64 = 2 * (1e6 as u64);
// st limit to how much cpu it can use
const CONT_CPU_LIMIT: f32 = 512 as f32;
// set limit to how many process it can spawn inside the container
const CONT_PID_LIMIT: i32 = 100;

pub fn create_cgroup(
    container_id: &str,
    target_pid: u32,
) {
    // create a dbus connection with systemd
    let conn = Connection::new_system().unwrap();
    let proxy = conn.with_proxy(
        "org.freedesktop.systemd1",
        "/org/freedesktop/systemd1",
        Duration::new(5, 0),
    );

    // use the auto-generated code to communicate with systemd
    use super::dbus_systemd::OrgFreedesktopSystemd1Manager;

    let properties = build_properties(target_pid, CONT_MEM_LIMIT, CONT_CPU_LIMIT, CONT_PID_LIMIT, container_id);
    let _r = proxy.start_transient_unit(
        &format!("rocker-{}.scope", container_id),
        "replace",
        properties,
        Vec::new(),
    ).unwrap();

}


fn build_properties(
    target_pid: u32,
    mem: u64,
    cpus: f32,
    pids: i32,
    container_id: &str,
) -> Vec<(&'static str, arg::Variant<Box<dyn arg::RefArg>>)> {
    let mut vec: Vec<(&str, arg::Variant<Box<dyn arg::RefArg>>)> = Vec::new();
    vec.push(("PIDs", Variant(Box::new(vec![target_pid]))));
    vec.push((
        "Description",
        Variant(Box::new(
            format!("rocker container: {}", container_id).to_string(),
        )),
    ));

    vec.push(("MemoryAccounting", Variant(Box::new(true))));
    vec.push(("MemoryMax", Variant(Box::new(mem))));

    vec.push(("CPUAccounting", Variant(Box::new(true))));
    vec.push((
        "CPUQuotaPerSecUSec",
        Variant(Box::new((cpus * 1000000.0).round() as u64)),
    ));

    vec.push(("TasksAccounting", Variant(Box::new(true))));
    vec.push(("TasksMax", Variant(Box::new(pids as u64))));

    return vec;
}

