use std::u128;
use sysinfo::{MacAddr, Networks};

/// get mac addr
pub fn get_mac() -> MacAddr {
    let networks = Networks::new_with_refreshed_list();
    for data in networks.values() {
        let mac = data.mac_address();
        if mac.0[4] == 0 && mac.0[5] == 0 { continue; }
        return mac;
    }

    MacAddr::UNSPECIFIED
}

/// will extract the last 10 bits of the MAC addr as workers
pub fn get_mac_worker() -> u16 {
    let mac = get_mac().0;
    let one = u16::from(mac[4] << 6) << 2;
    let two = u16::from(mac[5]);
    one | two
}

/// retrieve the MAC worker and convert it to the worker ID in the snowflake algorithm
pub fn mac_worker_u128() -> u128 {
    let mac = get_mac_worker();
    u128::from(mac) << 12
}
