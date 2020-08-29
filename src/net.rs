use std::io::prelude::*;
use std::net::{TcpStream, ToSocketAddrs};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread::JoinHandle;
use std::time::Duration;

use std::io::BufReader;

use crate::packet::{self, Packet};

pub struct ConnectionHandle {
	pub listen_thread: JoinHandle<()>,
	outgoing_packet_send: Sender<Packet>,
	pub incoming_packet_recv: Receiver<Packet>,
}

impl ConnectionHandle {
	pub fn send(&mut self, packet: Packet) {
		self.outgoing_packet_send.send(packet);
	}

	pub fn receive(&mut self) -> Packet {
		self.incoming_packet_recv.recv().unwrap()
	}
}

pub fn connect<A: ToSocketAddrs>(address: A) -> ConnectionHandle {
	let (mut incoming_packet_send, mut incoming_packet_recv) =
		channel::<Packet>();
	let (mut outgoing_packet_send, mut outgoing_packet_recv) =
		channel::<Packet>();

	println!("Attempting TCP connection");
	let mut tcp_stream = TcpStream::connect(address).unwrap();

	let remote_address = tcp_stream.peer_addr().unwrap();
	println!("Connected to {}", remote_address);

	//let read_timeout = Duration::from_millis(500);
	//tcp_stream.set_read_timeout(Some(read_timeout));
	tcp_stream.set_read_timeout(None);

	let mut listen_tcp_stream = tcp_stream.try_clone().unwrap();
	//let mut buf_reader = BufReader::new(buf_tcp_stream);

	let _send_thread = std::thread::spawn(move || loop {
		let outgoing_packet = outgoing_packet_recv.recv().unwrap();
		let outgoing_bytes = outgoing_packet.to_bytes();

		println!("Sending {:?}", outgoing_packet);

		tcp_stream.write(outgoing_bytes);
	});

	let listen_thread = std::thread::spawn(move || loop {
		let packet = receive_packet(&mut listen_tcp_stream);

		println!("Received {:?}", packet);

		incoming_packet_send.send(packet);
	});

	ConnectionHandle {
		listen_thread,
		outgoing_packet_send,
		incoming_packet_recv,
	}
}

fn receive_packet(tcp_stream: &mut TcpStream) -> Packet {
	//let mut bytes = buf_reader.by_ref().bytes();

	let mut receive_buffer = Vec::new();

	let packet_type_byte = read_byte(tcp_stream, false).unwrap();

	let mut remaining_length: u64 = 0;
	loop {
		let remaining_length_byte = read_byte(tcp_stream, false).unwrap();
		let value_part = (remaining_length_byte & 0b0111_1111);
		let continue_part = (remaining_length_byte & 0b1000_0000);

		remaining_length += value_part as u64;

		if (continue_part == 0) {
			break;
		} else {
			remaining_length <<= 7;
		}
	}

	tcp_stream
		.take(remaining_length)
		.read_to_end(&mut receive_buffer);

	// match buf_reader.read_to_end(&mut receive_buffer) {
	// 	Ok(0) => {},
	// 	Ok(amount) => {
	// 		println!("received {} bytes", amount);
	// 		break;
	// 	},
	// 	Err(e) => {
	// 		panic!("uh oh")
	// 	},
	// };

	//let packet = Packet::from_bytes(receive_buffer);

	packet::decode(packet_type_byte, receive_buffer)
}

fn read_byte(tcp_stream: &mut TcpStream, block: bool) -> Option<u8> {
	let mut buffer = [0u8];

	loop {
		match tcp_stream.read(&mut buffer).unwrap() {
			0 if block == true => continue,
			0 if block == false => return None,
			1 => return Some(buffer[0]),
			_ => panic!("Received more than 1 byte somehow"),
		}
	}
}
