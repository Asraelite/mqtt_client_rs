use crate::net::{self, ConnectionHandle};
#[allow(unused)]
use crate::packet::{
	self, ConnackPacket, ConnectPacket, Packet, SubscribePacket, Subscription,
};

pub struct Client {
	connection: ConnectionHandle,
}

impl Client {
	pub fn connect() -> Self {
		let server_addr = "broker.hivemq.com:1883";
		// let server_addr = "test.mosquitto.org:1883";
		// let server_addr = "mqtt.flespi.io:1883";
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
				panic!("Error while receiving connack: {}", err);
			}
			Ok(packet) => {
				panic!("Expected connack but received: {:?}", packet);
			}
		};

		Self { connection }
	}

	pub fn subscribe<S: Into<String>>(&mut self, topic: S) {
		let topic = topic.into();
		let subscribe_packet = SubscribePacket {
			subscriptions: vec![Subscription { topic }],
		};
		let subscribe_packet = Packet::Subscribe(subscribe_packet);
		self.connection.send(subscribe_packet);

		self.connection.receive().unwrap();
		self.connection.receive().unwrap();
	}

	pub fn receive(&mut self) -> Packet {
		self.connection.receive().unwrap()
	}
}
