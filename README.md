# lxc-rs

rust bindings to liblxc.

work in progress, definitely not ready for any kind of use other than experimental.

## example

```rust
extern crate lxc;

use lxc::Container;

fn main() {
	println!("using LXC version {}", lxc::get_version());

	let lxcpath = "/var/lib/lxc";
	let containers = Container::list(lxcpath).unwrap();

	for c in containers {
		println!("{} - {}", c.name, c.state());
	}	
}
```

see the ```lxc/examples``` directory for more usage examples.

## note

the original liblxc library needs to be available on the system to use this crate.
