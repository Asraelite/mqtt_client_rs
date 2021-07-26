use std::convert::TryFrom;

#[derive(Debug)]
pub struct UnknownPacket {
	type_id: u8,
	bytes: Vec<u8>,
}

impl UnknownPacket {
	pub fn new(bytes: Vec<u8>) -> Self {
		Self {
			type_id: bytes[0] >> 4,
			bytes,
		}
	}

	pub fn bytes(&self) -> &Vec<u8> {
		&self.bytes
	}
}

#[derive(Debug)]
pub struct ConnackPacket {
	pub flags: u8,
	pub return_code: ConnackReturnCode,
}

#[derive(Debug)]
pub enum ConnackReturnCode {
	Accepted,
	UnacceptableProtocolVersion,
	IdentifierRejected,
	ServerUnavailable,
	BadCredentials,
	NotAuthorized,
	Unknown(u8),
}

#[derive(Debug, Copy, Clone)]
pub enum Qos {
	ZERO,
	ONE,
	TWO,
}

#[derive(Debug)]
pub enum Packet {
	Connect(ConnectPacket),
	Connack(ConnackPacket),
	Subscribe(SubscribePacket),
	Unknown(UnknownPacket),
}

impl Packet {
	pub fn encode(&self) -> Vec<u8> {
		use Packet::*;
		match self {
			Connect(inner) => inner.bytes(),
			Connack(inner) => inner.bytes(),
			Subscribe(inner) => inner.bytes(),
			Unknown(inner) => inner.bytes().clone(),
		}
	}

	pub fn from_bytes(bytes: Vec<u8>) -> Self {
		println!("Decoding {}", bytes[0]);

		let packet_type_id = bytes[0] >> 4;
		let tail = &bytes[1..];
		match packet_type_id {
			2 => Packet::Connack(ConnackPacket::decode(tail)),
			_ => Packet::Unknown(UnknownPacket::new(bytes)),
		}
	}
}

// TODO: Use username and password in encoding
#[derive(Debug)]
pub struct ConnectPacket {
	pub client_id: Option<String>,
	pub username: Option<String>,
	pub password: Option<String>,
	pub keep_alive: u16,
}

impl ConnectPacket {
	fn bytes(&self) -> Vec<u8> {
		if !client_id_valid(&self.client_id) {
			panic!("Invalid client ID ({:?})", self.client_id);
		}

		let mut packet_bytes = Vec::new();

		// Create tail (variable header + payload)
		let mut tail = create_connect_packet_tail(&self);
		let tail_length = tail.len();

		// Create head (fixed header)
		let packet_type_id = 1;
		let control_packet_type_byte = packet_type_id << 4;
		let mut remaining_length_encoded = encode_variable_int(tail_length);

		// Push head and tail
		packet_bytes.push(control_packet_type_byte);
		packet_bytes.append(&mut remaining_length_encoded);
		packet_bytes.append(&mut tail);

		packet_bytes
	}

	fn decode(_tail: Vec<u8>) -> ConnectPacket {
		unimplemented!();
	}
}

fn create_connect_packet_tail(packet: &ConnectPacket) -> Vec<u8> {
	let mut bytes = Vec::new();

	// Variable header

	// Protocol name, 6 bytes, [0, 4, 'M', 'Q', 'T', 'T']
	let protocol_name_length: u16 = 4;
	let protocol_name_length = &protocol_name_length.to_be_bytes();
	let protocol_name_data = b"MQTT";
	bytes.extend_from_slice(protocol_name_length);
	bytes.extend_from_slice(protocol_name_data);

	// Protocol level, 1 byte, [4]
	let protocol_level_byte: u8 = 4;
	bytes.push(protocol_level_byte);

	// Connect flags, 1 byte
	#[allow(unused)]
	let connect_flags_byte: u8 = {
		const USERNAME: u8 = 0b1000_0000;
		const PASSWORD: u8 = 0b0100_0000;
		const WILL_RETAIN: u8 = 0b0010_0000;
		const WILL_QOS_LEVEL1: u8 = 0b0000_1000;
		const WILL_QOS_LEVEL2: u8 = 0b0001_0000;
		const WILL_FLAG: u8 = 0b0000_0100;
		const CLEAN_SESSION: u8 = 0b0000_0010;

		0 | CLEAN_SESSION
	};
	bytes.push(connect_flags_byte);

	// Keep alive, 2 bytes
	let keep_alive_time: u16 = packet.keep_alive;
	let keep_alive_time = &keep_alive_time.to_be_bytes();
	bytes.extend_from_slice(keep_alive_time);

	// Payload

	let default_client_id = String::from("");
	// Client ID section
	let client_id = match &packet.client_id {
		Some(x) => x,
		None => &default_client_id,
	};
	let client_id_length = client_id.len() as u16;
	let client_id_length = &client_id_length.to_be_bytes();
	let client_id_data = client_id.as_bytes();
	bytes.extend_from_slice(client_id_length);
	bytes.extend_from_slice(client_id_data);

	bytes
}

#[derive(Debug)]
pub struct SubscribePacket {
	pub subscriptions: Vec<Subscription>,
}

#[derive(Debug, Clone)]
pub struct Subscription {
	pub topic: String,
	// TODO: Subscribe options
}

impl SubscribePacket {
	fn bytes(&self) -> Vec<u8> {
		let mut packet_bytes = Vec::new();

		// Create tail (variable header + payload)
		let mut tail = create_subscribe_packet_tail(&self.subscriptions);
		let tail_length = tail.len();

		// Create head (fixed header)
		let packet_type_id = 8;
		let reserved_bits = 0b0010;
		let control_packet_type_byte = packet_type_id << 4 | reserved_bits;
		let mut remaining_length_encoded = encode_variable_int(tail_length);

		// Push head and tail
		packet_bytes.push(control_packet_type_byte);
		packet_bytes.append(&mut remaining_length_encoded);
		packet_bytes.append(&mut tail);

		println!("x: {:?}", packet_bytes);

		packet_bytes
	}

	fn decode(_tail: Vec<u8>) -> ConnectPacket {
		unimplemented!();
	}
}

fn create_subscribe_packet_tail(subscriptions: &Vec<Subscription>) -> Vec<u8> {
	let mut bytes = Vec::new();

	// Variable header

	// Packet identifier, 2 bytes
	// TODO: Make this dynamically assigned. Each subscribe packet must use
	// currently unused packet identifier. Identifiers become available for
	// re-use after a suback, puback etc.
	let packet_identifier: u16 = 1;
	bytes.extend_from_slice(&mut packet_identifier.to_be_bytes());

	// // Option flags, 1 byte
	// #[allow(unused)]
	// let connect_flags_byte: u8 = {
	// 	const USERNAME: u8 = 0b1000_0000;
	// 	const PASSWORD: u8 = 0b0100_0000;
	// 	const WILL_RETAIN: u8 = 0b0010_0000;
	// 	const WILL_QOS_LEVEL1: u8 = 0b0000_1000;
	// 	const WILL_QOS_LEVEL2: u8 = 0b0001_0000;
	// 	const WILL_FLAG: u8 = 0b0000_0100;
	// 	const CLEAN_SESSION: u8 = 0b0000_0010;

	// 	0 | CLEAN_SESSION
	// };
	// bytes.push(connect_flags_byte);

	// // TODO: Properties
	// let properties_length = 0;
	// let mut properties_length_encoded = encode_variable_int(properties_length);
	// bytes.append(&mut properties_length_encoded);
	// TODO: Why

	// Payload

	for subscription in subscriptions {
		let topic_length =
			u16::try_from(subscription.topic.len()).expect(format!(
				"Subscription topic is too long ({} bytes)",
				subscription.topic.len(),
			).as_str());
		let topic = subscription.topic.as_bytes();
		// TODO
		let subscribe_options_bitfield = 2u8;
		bytes.extend_from_slice(&topic_length.to_be_bytes());
		bytes.extend_from_slice(&topic);
		bytes.push(subscribe_options_bitfield);
	}

	bytes
}

fn client_id_valid(client_id: &Option<String>) -> bool {
	let client_id = match client_id {
		Some(x) => x,
		None => return true,
	};

	if client_id.len() == 0 || client_id.len() > 23 {
		return false;
	}

	client_id
		.chars()
		.all(|letter| letter.is_ascii_alphanumeric())
}

pub fn encode_variable_int(value: usize) -> Vec<u8> {
	let maximum_bytes = 4;
	let maximum_size = (1 << (maximum_bytes * 7)) - 1;

	if value > maximum_size {
		panic!("Maximum encodable int size exceeded (got {})", value);
	}

	let mut bytes = Vec::with_capacity(4);
	let mut remaining_value = value;

	loop {
		let mut encoded_byte = (remaining_value & 0b0111_1111) as u8;
		remaining_value >>= 7;

		if remaining_value > 0 {
			encoded_byte |= 0b1000_0000;
			bytes.push(encoded_byte);
		} else {
			bytes.push(encoded_byte);
			break;
		}
	}

	bytes
}

pub fn decode(bytes: Vec<u8>) -> Packet {
	Packet::from_bytes(bytes)
}

impl ConnackPacket {
	fn bytes(&self) -> Vec<u8> {
		unimplemented!();
	}

	fn decode<B: AsRef<[u8]>>(tail: B) -> Self {
		let mut bytes = tail.as_ref().iter();

		let &flag_byte = bytes.next().unwrap();
		let &return_code_byte = bytes.next().unwrap();

		let return_code = match return_code_byte {
			0 => ConnackReturnCode::Accepted,
			1 => ConnackReturnCode::UnacceptableProtocolVersion,
			2 => ConnackReturnCode::IdentifierRejected,
			3 => ConnackReturnCode::ServerUnavailable,
			4 => ConnackReturnCode::BadCredentials,
			5 => ConnackReturnCode::NotAuthorized,
			x => ConnackReturnCode::Unknown(x),
		};

		ConnackPacket {
			flags: flag_byte,
			return_code,
		}
	}
}
