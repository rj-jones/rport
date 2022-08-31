#[allow(non_snake_case)]
pub mod TlvType {
    pub const DEVICE_ID: u16 = 0x0001;
    pub const NET: u16 = 0x0002;
    pub const INTERFACE: u16 = 0x0003;
    pub const CAPABILITIES: u16 = 0x0004;
    pub const VERSION: u16 = 0x0005;
    pub const PLATFORM: u16 = 0x0006;
    pub const VLAN: u16 = 0x0102;
    pub const TAG_INFO: u16 = 0x0108;
}

pub struct FdpPdu<'a> {
    bytes: &'a [u8],
    valid: bool,
    pub switch_name: String,
    pub switch_ip: String,
    pub switch_port: String,
    pub switch_vlan_d: String,
    pub switch_vlan_v: String,
}

impl<'a> FdpPdu<'a> {
    // Attempt to create a new FDP PDU with the given packet.
    pub fn new(bytes: &'a [u8]) -> Option<Self> {
        let bytes = bytes.clone();

        let mut pdu = Self {
            bytes,
            valid: false,
            switch_name: String::new(),
            switch_ip: String::new(),
            switch_port: String::new(),
            switch_vlan_d: String::new(),
            switch_vlan_v: String::new(),
        };

        // Parse TLVs
        let mut index = 26;
        loop {
            if (index + 1) < bytes.len() {
                // Type/Length
                let t = u16::from_be_bytes([bytes[index], bytes[index + 1]]) as usize;
                let l = u16::from_be_bytes([bytes[index + 2], bytes[index + 3]]) as usize;

                if (index + l) > bytes.len() {
                    break;
                }

                // Parse Value
                // Note: Avoid index out of bounds by using = and -1 for the slices
                match t as u16 {
                    TlvType::DEVICE_ID => {
                        if let Ok(value) = std::str::from_utf8(&bytes[index + 4..=(index + l - 1)])
                        {
                            pdu.switch_name = value.to_string();
                            pdu.valid = true;
                        }
                    }
                    TlvType::NET => {
                        // The IP of the switch is located in the last 4 bytes.
                        let mut ip = String::new();
                        for byte in &bytes[(index + l - 4)..=(index + l - 1)] {
                            if ip.len() == 0 {
                                ip.push_str(byte.to_string().as_str());
                            } else {
                                ip.push('.');
                                ip.push_str(byte.to_string().as_str());
                            }
                        }
                        pdu.switch_ip = ip;
                    }
                    TlvType::INTERFACE => {
                        if let Ok(value) = std::str::from_utf8(&bytes[index + 4..=(index + l - 1)])
                        {
                            pdu.switch_port = Self::remove_chars(&value.to_string());
                        }
                    }
                    TlvType::CAPABILITIES => {}
                    TlvType::VERSION => {}
                    TlvType::PLATFORM => {}
                    TlvType::VLAN => {
                        let vlan_data = u16::from_be_bytes([bytes[index + 4], bytes[index + 5]]);
                        if pdu.switch_vlan_d.len() == 0 && vlan_data != 0 {
                            pdu.switch_vlan_d = vlan_data.to_string();
                        }
                    }
                    TlvType::TAG_INFO => {
                        // Bytes 2,3 & 7,8 contain vlan info.
                        let vlan_bytes = bytes[(index + 4)..=(index + l - 1)].to_vec();
                        let vlan_data = u16::from_be_bytes([vlan_bytes[2], vlan_bytes[3]]);
                        let vlan_voice = u16::from_be_bytes([vlan_bytes[7], vlan_bytes[8]]);
                        pdu.switch_vlan_d = format!("{}", vlan_data);
                        pdu.switch_vlan_v = format!("{}", vlan_voice);
                    }
                    _ => {}
                }

                // Go to the next TLV
                index += l;
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

    pub fn switch(&self) -> String {
        let mut switch = String::new();
        if self.switch_name.len() > 0 {
            switch.push_str(&self.switch_name);
        }
        if self.switch_ip.len() > 0 {
            if switch.len() > 0 {
                switch.push_str(" ");
            }
            switch.push_str(&format!("({})", &self.switch_ip));
        }
        switch
    }

    /// Return the VLANs in a String separated by a comma if there is
    /// more than one.
    pub fn vlan(&self) -> String {
        let mut vlan = String::new();
        if self.switch_vlan_d.len() > 0 {
            vlan.push_str(&self.switch_vlan_d);
        }
        if self.switch_vlan_v.len() > 0 {
            if vlan.len() > 0 {
                vlan.push_str(",");
            }
            vlan.push_str(&self.switch_vlan_v);
        }
        vlan
    }

    pub fn print(&self) {
        println!("");
        println!("Switch: {} ({})", self.switch_name, self.switch_ip);
        println!("Port:   {}", self.switch_port);
        println!("Data:   {}", self.switch_vlan_d);
        println!("Voice:  {}", self.switch_vlan_v);
        println!("Bytes:  {:02X?}", self.bytes);
        println!("");
    }

    /// Remove alphabetic characters from a given string.
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
