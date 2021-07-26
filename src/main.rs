#![allow(dead_code)]

mod net;
mod packet;

#[allow(unused)]
use packet::{ConnackPacket, ConnectPacket, Packet};

fn main() {
	let server_addr = "broker.hivemq.com:1883";
	//let server_addr = "test.mosquitto.org:1883";
	//let server_addr = "mqtt.flespi.io:1883";
	//let server_addr = "localhost:2000";
	let mut connection = net::connect(server_addr);

	let connect_packet = Packet::Connect(packet::ConnectPacket {
		client_id: None,
		keep_alive: 60,
		username: Some(String::from("test")),
		password: None,
	});
	connection.send(connect_packet);

	let response = connection.receive();
	match response {
		Ok(Packet::Connack(ConnackPacket {
			return_code: packet::ConnackReturnCode::Accepted,
			..
		})) => {}
		Err(err) => {
			panic!("Errored while receiving packet: {}", err);
		}
		Ok(packet) => {
			panic!("Received some other packet: {:?}", packet);
		}
	};
	let response = connection.receive();
	println!("B: {:?}", response);

	//println!("a");

	connection.listen_thread.join().unwrap();
}
