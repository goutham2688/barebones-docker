use std::path::Path;
use std::fs::create_dir_all;
use std::ffi::CString;

use nix::{
    fcntl::{open, OFlag},
    mount::{umount, MsFlags},
    sched::{clone, setns, CloneFlags},
    sys::{signal::Signal, wait::waitpid},
    unistd::{chdir, chroot, execv}, 
};

use crate::network::{create_network_namespace, delete_network_namespace};
use crate::cgroup::create_cgroup;

const CONTAINER_MNT_PATH: &str = "/mnt/c/Users/gtamils1/Documents/personal/prog/minDocker/rustDocker/sample_containers/busybox";
const NETNS_PATH: &str = "/run/netns";

pub fn start_container(container_id: &str, exec_cmd: &str){
    println!("starting container: \"{}\" to exec cmd: \"{}\"",container_id,exec_cmd);
    
    // in a normal run for a actual docker image, 
    // you'd need a overlay file system to merge multiple layers within docker,
    // into a single looking file system. and its mounted within the system.
    // https://www.datalight.com/blog/2016/01/27/explaining-overlayfs-%E2%80%93-what-it-does-and-how-it-works
    // but since this a simple busybox demo, overlay fs mounting is skipped.

    // create a network namespace
    create_network_namespace(container_id);

    // typically a virtual (veth) interface would be created with each container
    // so that the container can communicate with other containers via a bridge nw
    // https://aly.arriqaaq.com/linux-networking-bridge-iptables-and-docker/
    // so, we're going to skip it, since this is a simple application.

    // create a clone process
    // https://man7.org/linux/man-pages/man2/clone.2.html
    // create a seperate stack memory for child
    const CONTAINER_STACK_SIZE: usize = 1024 * 1024;
    let mut stack = Box::new([0; CONTAINER_STACK_SIZE]);

    // child process function
    let child_process_function = Box::new(|| {
        let netns_path = format!("{}/{}", NETNS_PATH, &format!("ns-{}", &container_id));
        setns_with_flags(&netns_path, CloneFlags::CLONE_NEWNET);

        nix::unistd::sethostname(&container_id).unwrap();

        // change the root directory of the calling process to specified
        // https://man7.org/linux/man-pages/man2/chroot.2.html
        chroot(Path::new(CONTAINER_MNT_PATH)).unwrap();
        // change working directory to root
        chdir("/").unwrap();

        // mount dirs that would be otherwise present in the system
        mount_container_fs();

        // execute the users command, this is entrypoint in docker
        // so far the chile process process was a clone of its parents process (which was this code)
        // but with execv, we change that to the actual process that the user set as entrypoint
        execv(
            &CString::new(exec_cmd.to_string()).unwrap(),
            &[CString::new(exec_cmd.to_string()).unwrap()],
        )
        .unwrap();

        return 0;
    });

    let clone_flags = CloneFlags::CLONE_NEWNS
        | CloneFlags::CLONE_NEWPID
        | CloneFlags::CLONE_NEWUTS
        | CloneFlags::CLONE_NEWIPC;
    let pid = clone(child_process_function, &mut *stack, clone_flags, Some(Signal::SIGCHLD as i32)).unwrap();
    create_cgroup(&container_id, pid.as_raw() as u32);
    waitpid(pid, None).unwrap();

    
    //un mount the directories
    umount_container_fs(CONTAINER_MNT_PATH);

    // delete the network namespace created
    delete_network_namespace(container_id);
}


pub fn setns_with_flags(ns_path: &str, ns_type: CloneFlags) {
    // change namespace of the calling thread
    //https://man7.org/linux/man-pages/man2/setns.2.html
    
    // flags for opening the file
    let mut oflag = OFlag::empty();
    oflag.insert(OFlag::O_RDONLY);
    oflag.insert(OFlag::O_EXCL);

    //open netns with readonly (O_RDONLY) and error out if the files not open (O_EXCL)
    let netns_fd = open(ns_path, oflag, nix::sys::stat::Mode::empty()).unwrap();
    setns(netns_fd, ns_type).unwrap();
}

fn mount_container_fs() {
    // TODO: understand and explain these
    create_dir_all("/proc").unwrap();
    nix::mount::mount::<str, Path, [u8], str>(
        Some("proc"),
        Path::new("/proc"),
        Some(b"proc".as_ref()),
        MsFlags::empty(),
        Some(""),
    )
    .unwrap();

    nix::mount::mount::<str, Path, [u8], str>(
        Some("tmpfs"),
        Path::new("/tmp"),
        Some(b"tmpfs".as_ref()),
        MsFlags::empty(),
        Some(""),
    )
    .unwrap();

    nix::mount::mount::<str, Path, [u8], str>(
        Some("tmpfs"),
        Path::new("/dev"),
        Some(b"tmpfs".as_ref()),
        MsFlags::empty(),
        Some(""),
    )
    .unwrap();

    create_dir_all("/dev/pts").unwrap();
    nix::mount::mount::<str, Path, [u8], str>(
        Some("devpts"),
        Path::new("/dev/pts"),
        Some(b"devpts".as_ref()),
        MsFlags::empty(),
        Some(""),
    )
    .unwrap();

    create_dir_all("/sys").unwrap();
    nix::mount::mount::<str, Path, [u8], str>(
        Some("sysfs"),
        Path::new("/sys"),
        Some(b"sysfs".as_ref()),
        MsFlags::empty(),
        Some(""),
    )
    .unwrap();
}

fn umount_container_fs(container_mount_path: &str) {
    umount(Path::new(&format!("{}/dev/pts", &container_mount_path))).unwrap();
    umount(Path::new(&format!("{}/dev", &container_mount_path))).unwrap();
    umount(Path::new(&format!("{}/sys", &container_mount_path))).unwrap();
    umount(Path::new(&format!("{}/proc", &container_mount_path))).unwrap();
    umount(Path::new(&format!("{}/tmp", &container_mount_path))).unwrap();
}
