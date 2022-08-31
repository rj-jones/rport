use std::str::from_utf8;

use to_binary::BinaryString;

#[allow(non_snake_case)]
pub mod TlvType {
    pub const END_OF_LLDP_PDU: usize = 0x00;
    pub const CHASSIS_ID: usize = 0x01;
    pub const PORT_ID: usize = 0x02;
    pub const TIME_TO_LIVE: usize = 0x03;
    pub const PORT_DESCRIPTION: usize = 0x04;
    pub const SYSTEM_NAME: usize = 0x05;
    pub const SYSTEM_DESCRIPTION: usize = 0x06;
    pub const SYSTEM_CAPABILITIES: usize = 0x07;
    pub const MANAGEMENT_ADDRESS: usize = 0x08;
    pub const VLAN_ID: usize = 0x7F;
}

pub struct Tlv {
    pub typ: usize,
    pub len: usize,
    pub val: String,
}

impl Tlv {
    pub fn new(typ: usize, len: usize, val: String) -> Self {
        Self { typ, len, val }
    }

    pub fn get_typ(typ: &usize) -> String {
        match *typ {
            TlvType::END_OF_LLDP_PDU => String::from("END_OF_LLDP_PDU"),
            TlvType::CHASSIS_ID => String::from("CHASSIS_ID"),
            TlvType::PORT_ID => String::from("PORT_ID"),
            TlvType::TIME_TO_LIVE => String::from("TIME_TO_LIVE"),
            TlvType::PORT_DESCRIPTION => String::from("PORT_DESCRIPTION"),
            TlvType::SYSTEM_NAME => String::from("SYSTEM_NAME"),
            TlvType::SYSTEM_DESCRIPTION => String::from("SYSTEM_DESCRIPTION"),
            TlvType::SYSTEM_CAPABILITIES => String::from("SYSTEM_CAPABILITIES"),
            TlvType::MANAGEMENT_ADDRESS => String::from("MANAGEMENT_ADDRESS"),
            TlvType::VLAN_ID => String::from("VLAN_ID"),
            _ => String::from("Reserved or Custom TLV"),
        }
    }
}

/// Represents a EthernetII layer 2 packet
#[allow(dead_code)]
pub struct LldpPdu<'a> {
    bytes: &'a [u8],                // Raw EthernetII packet bytes
    valid: bool,                    // Contains valid vlan info?
    destination_mac: String,        // Port MAC
    source_mac: String,             // Switch MAC
    chassis_id: String,             // Switch MAC
    port_id: String,                // Port MAC
    pub port_description: String,   // Port Number
    pub system_description: String, // Switch Name
    pub vlan: String,               // Vlan
    tlvs: Vec<Tlv>,
}

impl<'a> LldpPdu<'a> {
    // Attempt to create a new LLDP PDU with the given packet.
    pub fn new(bytes: &'a [u8]) -> Option<Self> {
        let bytes = bytes.clone();

        let mut pdu = Self {
            bytes,
            valid: false,
            destination_mac: format!("{:02X?}", &bytes[0..=5]),
            source_mac: format!("{:02X?}", &bytes[6..=11]),
            chassis_id: String::new(),
            port_id: String::new(),
            port_description: String::new(),
            system_description: String::new(),
            vlan: String::new(),
            tlvs: Vec::<Tlv>::new(),
        };

        // Parse TLVs (port, vlan)
        let mut index = 14;
        loop {
            if (index + 1) < bytes.len() {
                let typ_len_bytes = &bytes[index..=index + 1];
                let (typ, len) = Self::parse_typ_len(typ_len_bytes);
                if len > (bytes.len() - index) {
                    println!("Value length error...");
                    break;
                }
                let tlv_bytes = &bytes[index..=index + 1 + len];
                let tlv = Self::parse_value(&mut pdu, typ, len, tlv_bytes);
                pdu.tlvs.push(tlv);
                index += 2 + len; // len/val + len to get to next tlv
            } else {
                break;
            }
        }

        if pdu.valid {
            Some(pdu)
        } else {
            None
        }
    }

    pub fn print(&self) {
        println!("");
        println!("Hex:    {:02X?}", self.bytes);
        println!("Switch: {} {}", self.system_description, self.chassis_id);
        println!("Port:   {} {}", self.port_description, self.port_id);
        println!("Vlan:   {}", self.vlan);
        println!("");
        self.print_tlvs();
        println!("");
    }

    #[allow(dead_code)]
    pub fn print_tlvs(&self) {
        for tlv in &self.tlvs {
            if (tlv.typ < 9 && tlv.typ != 0) || tlv.typ == TlvType::VLAN_ID {
                println!(
                    "{}: {} ({} bytes)",
                    Tlv::get_typ(&tlv.typ),
                    tlv.val,
                    tlv.len
                );
            }
        }
    }

    fn parse_typ_len(typ_len_bytes: &[u8]) -> (usize, usize) {
        let bin_str = BinaryString::from(typ_len_bytes).0;
        let tlv_typ_b = &bin_str[0..=6]; // First 7 bits are type
        let tlv_len_b = &bin_str[7..=15]; // Next 9 bits are length
        let tlv_typ = usize::from_str_radix(tlv_typ_b, 2).unwrap();
        let tlv_len = usize::from_str_radix(tlv_len_b, 2).unwrap();
        // println!("Type Bytes:   {:02X?}", typ_len_bytes);
        // println!("Type:   {} ({})", tlv_typ, Tlv::get_typ(&tlv_typ));
        // println!("Length: {}", tlv_len);
        (tlv_typ, tlv_len)
    }

    // Parse the given slice
    fn parse_value(pdu: &mut LldpPdu<'_>, typ: usize, len: usize, tlv_bytes: &[u8]) -> Tlv {
        // println!("Type: {:02X?}", typ);
        // println!("Len: {:02X?}", len);
        // println!("Bytes: {:02X?}", tlv_bytes);
        match typ {
            TlvType::END_OF_LLDP_PDU => {
                let value = String::from("End of PDU");
                return Tlv::new(typ, len, value);
            }
            TlvType::CHASSIS_ID => {
                let value = format!("{:02X?}", &tlv_bytes[3..]) // 1-2 (typ-len), 3 (sub-type), 4.. (value)
                    .replace("[", "")
                    .replace("]", "")
                    .replace(", ", ":");
                pdu.chassis_id = value.clone();
                return Tlv::new(typ, len, value);
            }
            TlvType::PORT_ID => {
                let value = format!("{:02X?}", &tlv_bytes[3..]) // 1-2 (typ-len), 3 (sub-type), 4.. (value)
                    .replace("[", "")
                    .replace("]", "")
                    .replace(", ", ":");
                pdu.port_id = value.clone();
                return Tlv::new(typ, len, value);
            }
            TlvType::TIME_TO_LIVE => Tlv::new(
                typ,
                len,
                u16::from_be_bytes(tlv_bytes[2..=3].try_into().unwrap()).to_string(),
            ),
            TlvType::PORT_DESCRIPTION => {
                let mut value = from_utf8(&tlv_bytes[2..]).unwrap().to_string();
                value = Self::remove_chars(&value);
                pdu.port_description = value.clone();
                Tlv::new(typ, len, value)
            }
            TlvType::SYSTEM_NAME => {
                let value = from_utf8(&tlv_bytes[2..]).unwrap().to_string();
                pdu.system_description = value.clone();
                Tlv::new(typ, len, value)
            }
            TlvType::SYSTEM_DESCRIPTION => {
                Tlv::new(typ, len, from_utf8(&tlv_bytes[2..]).unwrap().to_string())
            }
            TlvType::SYSTEM_CAPABILITIES => {
                let value = format!("{:02X?}", &tlv_bytes[2..]) // 1-2 (typ-len), 3 (sub-type), 4.. (value)
                    .replace("[", "")
                    .replace("]", "");
                return Tlv::new(typ, len, value);
            }
            TlvType::MANAGEMENT_ADDRESS => {
                let value = format!("{:02X?}", &tlv_bytes[2..]) // 1-2 (typ-len), 3 (sub-type), 4.. (value)
                    .replace("[", "")
                    .replace("]", "");
                return Tlv::new(typ, len, value);
            }
            TlvType::VLAN_ID => {
                // Find the VLAN id by using a specific OUI (IEEE defined)
                let oui = &tlv_bytes[2..=4];
                if oui == &[0x00, 0x80, 0xC2] && len == 6 {
                    let mut value =
                        u16::from_be_bytes(tlv_bytes[6..=7].try_into().unwrap()).to_string();

                    // Make valid=true if there is VLAN info.
                    if value.len() != 0 {
                        pdu.valid = true;
                        value = Self::remove_chars(&value);
                    }
                    pdu.vlan = value.clone();
                    return Tlv::new(typ, len, value);
                } else {
                    Tlv::new(typ, len, String::from("Reserved or Custom TLV"))
                }
            }
            _ => Tlv::new(typ, len, String::from("Reserved or Custom TLV")),
        }
    }

    fn remove_chars(str: &String) -> String {
        let mut new_str = String::new();
        for c in str.chars() {
            if !c.is_alphabetic() {
                new_str.push(c);
            }
        }
        new_str
    }
}
