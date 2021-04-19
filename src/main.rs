#![allow(unused)]

mod net;
mod packet;

use packet::{Packet, ConnectPacket};

fn main() {
	let server_addr = "broker.hivemq.com:1883";
	//let server_addr = "test.mosquitto.org:1883";
	//let server_addr = "mqtt.flespi.io:1883";
	//let server_addr = "localhost:2000";
	let mut connection = net::connect(server_addr);

	let connect_packet = Box::new(packet::ConnectPacket {
		client_id: None,
		keep_alive: 60,
		username: Some(String::from("test")),
		password: None,
	});
	connection.send(connect_packet);

	loop {
		let response = connection.receive();
	}

	//println!("a");

	connection.listen_thread.join();
}
