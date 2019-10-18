extern crate pnet;
#[macro_use]
extern crate clap;

use pnet::datalink::{self, NetworkInterface};
use pnet::datalink::Channel::Ethernet;
use pnet::packet::{Packet, MutablePacket};
use pnet::packet::ethernet::{EthernetPacket, MutableEthernetPacket};
use std::error::Error;

fn generate_args<'a, 'b>() -> clap::App<'a, 'b> {
    clap_app!(bcrelay =>
        (version: "0.1")
        (author: "Stephan Henrichs <kilobyte@kilobyte22.de>")
        (@arg INPUT: -i --input +required +multiple +takes_value "Input Interfaces")
        (@arg OUTPUT: -o --output +required +multiple +takes_value "Output Interface")
    )
}

fn main() {
    let m = generate_args().get_matches();
    let in_if = m.value_of("INPUT").unwrap();
    let out_if = m.value_of("OUTPUT").unwrap();

    let in_iface_match = |iface: &&NetworkInterface| &iface.name == in_if;
    let out_iface_match = |iface: &&NetworkInterface| &iface.name == out_if;

    let interfaces = datalink::interfaces();
    let in_interface = interfaces
        .iter()
        .filter(in_iface_match)
        .next()
        .expect("Could not find interface");

    let out_interface = interfaces
        .iter()
        .filter(out_iface_match)
        .next()
        .expect("Could not find interface");

    process_interface(in_interface.clone(), vec![out_interface.clone()])

}

fn process_interface(input: NetworkInterface, output: Vec<NetworkInterface>) {
    let mut rx = match datalink::channel(&input, Default::default()) {
        Ok(Ethernet(_, rx)) => rx,
        Ok(_) => panic!("Unhandled channel type"),
        Err(e) => panic!("An error occurred when creating the datalink channel: {}", e)
    };

    let mut txes = output.into_iter().map(|out_if| match datalink::channel(&out_if, Default::default()) {
        Ok(Ethernet(tx, _)) => tx,
        Ok(_) => panic!("Unhandled channel type"),
        Err(e) => panic!("An error occurred when creating the datalink channel: {}", e)
    }).collect::<Vec<_>>();

    loop {
        match rx.next() {
            Ok(packet) => {
                if let Some(packet) = EthernetPacket::new(packet) {
                    if packet.get_destination().is_broadcast() {
                        for tx in &mut txes {
                            tx.build_and_send(1, packet.packet().len(),
                                              &mut |new_packet| {
                                                  let mut new_packet = MutableEthernetPacket::new(new_packet).unwrap();

                                                  new_packet.clone_from(&packet)
                                              }
                            );
                        }
                    }
                }
            },
            Err(e) => panic!(format!("Error: {}", e.description()))
        }
    }
}
