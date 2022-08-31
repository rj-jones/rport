use pnet_datalink::{self, Channel, NetworkInterface};
use std::{
    net::{IpAddr, Ipv4Addr},
    time::{Duration, Instant},
};

use crate::exit_codes::{UNABLE_TO_CREATE_CHANNEL, UNHANDLED_CHANNEL_TYPE};

/// Represents a NIC and all of it's available interfaces. It will filter out non-virtual,
/// wired interfaces. You can then listen on those interfaces and apply a closure to each
/// packet's raw bytes.
pub struct Nic {
    /// Represents all interfaces.
    _interfaces_all: Vec<NetworkInterface>,

    /// Represents all non-virtual, wired interfaces.
    interfaces_wired: Vec<NetworkInterface>,
}

impl Nic {
    pub fn new() -> Self {
        let mut nic = Self {
            _interfaces_all: Vec::new(),
            interfaces_wired: Vec::new(),
        };

        nic.filter_interfaces();

        return nic;
    }

    /// Listen on the filtered interfaces. Do something with each packet
    /// via a closure.
    pub fn listen_wired<F>(&self, duration: Duration, handle_packet: F)
    where
        F: Fn(&[u8]),
    {
        if self.interfaces_wired.len() == 0 {
            println!("No wired interfaces to listen on.");
            return;
        }

        // Listen on each interface for the given duration.
        for interface in self.interfaces_wired.iter() {
            // Create a channel
            let (_, mut rx) = match pnet_datalink::channel(&interface, Default::default()) {
                Ok(Channel::Ethernet(tx, rx)) => (tx, rx),
                Ok(_) => {
                    println!("Unhandled channel type");
                    std::process::exit(UNHANDLED_CHANNEL_TYPE);
                }
                Err(e) => {
                    println!("Unable to create channel: {}", e);
                    std::process::exit(UNABLE_TO_CREATE_CHANNEL);
                }
            };

            println!("\nListening on \"{}\"\n", &interface.name);

            // Listen for the given duration.
            let start = Instant::now();
            loop {
                if start.elapsed() >= duration {
                    println!("\nFinished listening on \"{}\"\n", &interface.name);
                    return;
                }

                // Handle packets.
                match rx.next() {
                    Ok(packet_bytes) => handle_packet(packet_bytes),
                    Err(e) => println!("Error - Unable to receive packet: {}", e),
                }
            }
        }
    }

    /// Processes all found interfaces puts them in their respective lists.
    /// For now, this just filters out non-virtual, wired interfaces.
    fn filter_interfaces(&mut self) {
        let interfaces: Vec<NetworkInterface> = pnet_datalink::interfaces();

        // List all
        println!("\nInterfaces: ");
        for i in &interfaces {
            println!("{:?}", i);
        }

        let interfaces_wired: Vec<NetworkInterface> = interfaces
            .into_iter()
            // Filter for wired, non-virtual interfaces
            .filter(|iface: &NetworkInterface| {
                !iface.description.to_lowercase().contains("hyper")
                    && !iface.description.to_lowercase().contains("virtual")
                    && !iface.description.to_lowercase().contains("loop")
                    && !iface.description.to_lowercase().contains("wi-fi")
                    && !iface.description.to_lowercase().contains("wifi")
                    && !iface.description.to_lowercase().contains("wireless")
                    && !iface.description.to_lowercase().contains("bluetooth")
            })
            // Filter out 0.0.0.0 addresses that are active.
            .filter(|iface: &NetworkInterface| {
                let mut valid = true;
                for ip in iface.ips.iter() {
                    let ip_address = ip.ip();
                    if ip_address == IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)) {
                        valid = false;
                    }
                }
                valid
            })
            .collect();

        println!("\nWired Non-Virtual: ");
        for i in &interfaces_wired {
            println!("{:?}", i);
        }

        self.interfaces_wired.clone_from(&interfaces_wired);
    }
}
