use futures::executor::block_on;

use rtnetlink::NetworkNamespace;

pub fn create_network_namespace(container_id: &str) {
    println!("attempting to network namespace");
    let ns_future = NetworkNamespace::add(format!("ns-{}", container_id));
    block_on(ns_future).unwrap();
    println!("network namespace created successfully");
}

pub fn delete_network_namespace(container_id: &str) {
    println!("attempting to delete network namespace");
    let ns_future = NetworkNamespace::del(format!("ns-{}", container_id));
    block_on(ns_future).unwrap();
    println!("network namespace delete successfully");
}
