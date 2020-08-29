#![allow(unused)]

mod net;
mod packet;

use packet::Packet;

fn main() {
	//let server_addr = "broker.hivemq.com:1883";
	//let server_addr = "test.mosquitto.org:1883";
	let server_addr = "mqtt.flespi.io:1883";
	//let server_addr = "localhost:2000";
	let mut connection = net::connect(server_addr);

	let connect_packet = Packet::new_connect(None);
	connection.send(connect_packet);

	loop {
		let response = connection.receive();
	}

	connection.listen_thread.join();
}
