#![allow(dead_code)]

mod client;
mod net;
mod packet;

use client::Client;

fn main() {
	let mut client = Client::connect();

	client.subscribe("a");
}
