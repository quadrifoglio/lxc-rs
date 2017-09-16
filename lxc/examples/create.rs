extern crate lxc;

use lxc::{Container, Template};

fn main() {
	let lxcpath = "/var/lib/lxc";
    let template = Template::new("download")
        .option("-d", "alpine")
        .option("-r", "3.6")
        .option("-a", "amd64");

	let container = Container::create(lxcpath, "test", template).unwrap();

    if container.start().is_ok() {
        println!("Container started");
        container.stop().unwrap();
    }
    else {
        println!("Container failed to start");
    }

    container.destroy().unwrap();
}
