#[derive(Debug)]
pub enum PacketType {
	Connect,
	Connack,
	Test,
}

#[derive(Debug)]
pub struct Packet {
	packet_type: PacketType,
	body: Vec<u8>,
}

impl Packet {
	pub fn new_connect(client_id: Option<String>) -> Self {
		if !client_id_valid(&client_id) {
			panic!("Invalid client ID ({:?})", client_id);
		}

		let mut packet_bytes = Vec::new();

		// Create tail (variable header + payload)
		let mut tail = Self::create_connect_packet_tail(client_id);
		let tail_length = tail.len();

		// Create head (fixed header)
		let packet_type_id = 1;
		let control_packet_type_byte = (packet_type_id << 4);
		let mut remaining_length_encoded = encode_variable_int(tail_length);

		// Push head and tail
		packet_bytes.push(control_packet_type_byte);
		packet_bytes.append(&mut remaining_length_encoded);
		packet_bytes.append(&mut tail);

		Packet {
			packet_type: PacketType::Connect,
			body: packet_bytes,
		}
	}

	fn create_connect_packet_tail(client_id: Option<String>) -> Vec<u8> {
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
		//let keep_alive_time: u16 = 0xff;
		let keep_alive_time: u16 = 60;
		let keep_alive_time = &keep_alive_time.to_be_bytes();
		bytes.extend_from_slice(keep_alive_time);

		// Payload

		// Client ID section
		let client_id = match client_id {
			Some(x) => x,
			None => String::from(""),
		};
		let client_id_length = client_id.len() as u16;
		let client_id_length = &client_id_length.to_be_bytes();
		let client_id_data = client_id.as_bytes();
		bytes.extend_from_slice(client_id_length);
		bytes.extend_from_slice(client_id_data);

		bytes
	}

	pub fn new_test(values: Vec<u8>) -> Self {
		Packet {
			packet_type: PacketType::Test,
			body: values,
		}
	}

	pub fn to_bytes(&self) -> &Vec<u8> {
		&self.body
	}
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


pub fn decode(packet_type_byte: u8, tail: Vec<u8>) -> Packet {
	println!("Decoding {}", packet_type_byte);
	Packet::new_test(tail)
}
