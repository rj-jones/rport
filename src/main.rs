use std::time::Duration;

use exit_codes::SUCCESS;
use fdp::FdpPdu;
use lldp::LldpPdu;
use nic::Nic;
use pnet::packet::ethernet::EthernetPacket;
use reg::{print_hklm, Entry};

use crate::exit_codes::REGISTRY_WRITE_FAILURE;

mod exit_codes;
mod fdp;
mod lldp;
mod nic;
mod reg;

const REGISTRY_PATH: &'static str = r"SOFTWARE\rport";

fn main() {
    print_hklm(REGISTRY_PATH);

    let nic = Nic::new();

    // First try FDP for more details vlan information.
    nic.listen_wired(Duration::from_secs(62), |bytes| {
        if let Some(packet) = EthernetPacket::new(bytes) {
            match packet.get_destination().octets() {
                // FDP
                [0x01, 0xE0, 0x52, 0xCC, 0xCC, 0xCC] => {
                    handle_fdp(bytes);
                }
                _ => {}
            }
        }
    });

    // Try LLDP if FDP doesn't work.
    nic.listen_wired(Duration::from_secs(32), |bytes| {
        if let Some(packet) = EthernetPacket::new(bytes) {
            match packet.get_destination().octets() {
                // LLDP
                [0x01, 0x80, 0xC2, 0x00, 0x00, 0x0E] => {
                    handle_lldp(bytes);
                }
                _ => {}
            }
        }
    });
}

fn handle_fdp(bytes: &[u8]) {
    if let Some(pdu) = FdpPdu::new(bytes) {
        pdu.print();
        let mut entries = Vec::<Entry>::new();
        entries.push(Entry::new("Switch", &pdu.switch()));
        entries.push(Entry::new("Port", &pdu.switch_port));
        entries.push(Entry::new("Vlan", &pdu.vlan()));

        match reg::write_hklm(&entries, REGISTRY_PATH) {
            Ok(_) => {
                std::process::exit(SUCCESS);
            }
            Err(e) => {
                println!("Failure writing to registry \"{}\"\n{}", REGISTRY_PATH, e);
                std::process::exit(REGISTRY_WRITE_FAILURE);
            }
        }
    }
}

fn handle_lldp(bytes: &[u8]) {
    if let Some(pdu) = LldpPdu::new(bytes) {
        pdu.print();
        let mut entries = Vec::<Entry>::new();
        entries.push(Entry::new("Switch", &pdu.system_description));
        entries.push(Entry::new("Port", &pdu.port_description));
        entries.push(Entry::new("Vlan", &pdu.vlan));

        match reg::write_hklm(&entries, REGISTRY_PATH) {
            Ok(_) => {
                std::process::exit(SUCCESS);
            }
            Err(e) => {
                println!("Failure writing to registry \"{}\"\n{}", REGISTRY_PATH, e);
                std::process::exit(REGISTRY_WRITE_FAILURE);
            }
        }
    }
}
