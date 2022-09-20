use winreg::{enums::HKEY_LOCAL_MACHINE, RegKey};

use crate::exit_codes::REGISTRY_CREATE_OPEN_FAILURE;

pub struct Entry {
    pub key: &'static str,
    pub value: String,
}

impl Entry {
    pub fn new(key: &'static str, value: &String) -> Self {
        Self {
            key,
            value: value.to_owned(),
        }
    }
}

/// Print any existing registry values.
pub fn print_hklm(path: &'static str) {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);

    if let Ok(key) = hklm.open_subkey(path) {
        println!("\nExisting Registry Values");
        if let Ok(switch) = key.get_value::<String, _>("LastWrite") {
            println!("  LastWrite: {}", switch);
        }
        if let Ok(switch) = key.get_value::<String, _>("Switch") {
            println!("  Switch:    {}", switch);
        }
        if let Ok(port) = key.get_value::<String, _>("Port") {
            println!("  Port:      {}", port);
        }
        if let Ok(vlan) = key.get_value::<String, _>("Vlan") {
            println!("  Vlan:      {}", vlan);
        }
    }
}

pub fn write_hklm(entries: &[Entry], path: &'static str) -> std::io::Result<()> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);

    match hklm.create_subkey(path) {
        Ok((key, _disp)) => {
            println!("Writing registry values at \"{}\"\n", path);
            for entry in entries {
                println!("Key:    {}", &entry.key);
                println!("Value:  {}", &entry.value);
                match key.set_value(&entry.key, &entry.value) {
                    Ok(_v) => println!("Status: Successful Write\n"),
                    Err(e) => return Err(e),
                }
            }
            Ok(())
        }
        Err(e) => {
            println!("Failure creating/opening path \"{}\"\n{}", path, e);
            std::process::exit(REGISTRY_CREATE_OPEN_FAILURE);
        }
    }
}
