# rport
A simple layer 2 packet sniffer that can extract switch information via LLDP and Foundry Discovery Protocol (FDP). Rport will provide a switch's MAC, IP, and port specific information like its VLAN or MAC address. These values are written to the windows registry in "HKLM\SOFTWARE\rport" and can then be queried from something like PowerShell or BGInfo.
