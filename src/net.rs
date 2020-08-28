use std::io::prelude::*;
use std::net::{TcpStream, ToSocketAddrs};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread::JoinHandle;
use std::time::Duration;

use std::io::BufReader;

use crate::packet::Packet;

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

	let mut tcp_stream = TcpStream::connect(address).unwrap();

	let remote_address = tcp_stream.peer_addr().unwrap();
	println!("Connected to {}", remote_address);

	let read_timeout = Duration::from_millis(500);
	tcp_stream.set_read_timeout(Some(read_timeout));

	let buf_tcp_stream = tcp_stream.try_clone().unwrap();
	let mut buf_reader = BufReader::new(buf_tcp_stream);

	let _send_thread = std::thread::spawn(move || {
		loop {
			let outgoing_packet = outgoing_packet_recv.recv().unwrap();
			let outgoing_bytes = outgoing_packet.to_bytes();

			println!("Sending {:?}", outgoing_packet);

			tcp_stream.write(outgoing_bytes);
		}
	});

	let listen_thread = std::thread::spawn(move || {
		loop {
			let packet = receive_packet(&mut buf_reader);

			println!("Received {:?}", packet);

			incoming_packet_send.send(packet);
		}
	});

	ConnectionHandle {
		listen_thread,
		outgoing_packet_send,
		incoming_packet_recv,
	}
}

fn receive_packet(buf_reader: &mut BufReader<TcpStream>) -> Packet {
	let mut bytes = buf_reader.by_ref().bytes();

	let mut receive_buffer = Vec::new();

	loop {
		match buf_reader.read_to_end(&mut receive_buffer) {
			Ok(0) => {},
			Ok(amount) => {
				println!("received {} bytes", amount);
				break;
			},
			Err(e) => {
				panic!("uh oh")
			},
		};
	}

	//let packet = Packet::from_bytes(receive_buffer);

	Packet::new_test(receive_buffer)
}
