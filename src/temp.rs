use std::any::Any;

fn foo<T: Any>(x: T) {
	let x = &x as &dyn Any;

	if let Some(x) = x.downcast_ref::<String>() {
		println!("Got string {}", x);
	} else if let Some(x) = x.downcast_ref::<usize>() {
		println!("Got usize {}", x);
	}
}

fn main() {
	foo(9usize);
	foo(String::from("Hello"));
}
