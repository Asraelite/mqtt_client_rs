use std::error::Error;
use std::io::prelude::*;
use std::net::{TcpStream, ToSocketAddrs};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread::JoinHandle;

use crate::packet::Packet;

pub struct ConnectionHandle {
	pub listen_thread: JoinHandle<()>,
	outgoing_packet_send: Sender<Packet>,
	pub incoming_packet_recv: Receiver<Packet>,
}

impl ConnectionHandle {
	pub fn send(&mut self, packet: Packet) {
		self.outgoing_packet_send.send(packet).unwrap();
	}

	pub fn receive(&mut self) -> Result<Packet, Box<dyn Error>> {
		Ok(self.incoming_packet_recv.recv()?)
	}
}

pub fn connect<'a, A: ToSocketAddrs>(address: A) -> ConnectionHandle {
	let (incoming_packet_send, incoming_packet_recv) = channel::<Packet>();
	let (outgoing_packet_send, outgoing_packet_recv) = channel::<Packet>();

	println!("Attempting TCP connection");
	let mut tcp_stream = TcpStream::connect(address).unwrap();

	let remote_address = tcp_stream.peer_addr().unwrap();
	println!("Connected to {}", remote_address);

	//tcp_stream.set_read_timeout(Some(Duration::from_millis(500)));
	tcp_stream.set_read_timeout(None).unwrap();

	let mut listen_tcp_stream = tcp_stream.try_clone().unwrap();
	//let mut buf_reader = BufReader::new(buf_tcp_stream);

	let _send_thread = std::thread::spawn(move || loop {
		let outgoing_packet = match outgoing_packet_recv.recv() {
			Ok(packet) => packet,
			Err(err) => panic!("{}", err),
		};
		let outgoing_bytes = outgoing_packet.encode();

		println!("Sending {:?}", outgoing_packet);

		tcp_stream.write(outgoing_bytes.as_slice()).unwrap();
	});

	let listen_thread = std::thread::spawn(move || loop {
		let packet = match receive_packet(&mut listen_tcp_stream) {
			Ok(packet) => packet,
			Err(err) => panic!("{}", err),
		};

		println!("Received {:?}", packet);

		incoming_packet_send.send(packet).unwrap();
	});

	ConnectionHandle {
		listen_thread,
		outgoing_packet_send,
		incoming_packet_recv,
	}
}

fn receive_packet(
	tcp_stream: &mut TcpStream,
) -> Result<Packet, Box<dyn Error>> {
	//let mut bytes = buf_reader.by_ref().bytes();

	let mut receive_buffer = Vec::new();

	let packet_type_byte = read_byte(tcp_stream, false)?
		.ok_or("Could not read from TCP stream")?;

	let mut remaining_length: u64 = 0;
	loop {
		let remaining_length_byte = read_byte(tcp_stream, false)?
			.ok_or("Could not read from stream")?;
		let value_part = remaining_length_byte & 0b0111_1111;
		let continue_part = remaining_length_byte & 0b1000_0000;

		remaining_length += value_part as u64;

		if continue_part == 0 {
			break;
		} else {
			remaining_length <<= 7;
		}
	}

	receive_buffer.push(packet_type_byte);
	tcp_stream
		.take(remaining_length)
		.read_to_end(&mut receive_buffer)?;

	Ok(Packet::from_bytes(receive_buffer))
}

fn read_byte(
	tcp_stream: &mut TcpStream,
	block: bool,
) -> Result<Option<u8>, Box<dyn Error>> {
	let mut buffer = [0u8];

	loop {
		match tcp_stream.read(&mut buffer).unwrap() {
			0 if block == true => continue,
			0 if block == false => return Ok(None),
			1 => return Ok(Some(buffer[0])),
			_ => panic!("Received more than 1 byte somehow"),
		}
	}
}
