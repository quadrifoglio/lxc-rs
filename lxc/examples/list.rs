extern crate lxc;

use lxc::Container;

fn main() {
	let lxcpath = "/var/lib/lxc";
	let containers = Container::list(lxcpath).unwrap();

	for c in containers {
		println!("{} - {}", c.name, c.state());
	}	
}
