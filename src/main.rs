/*
 ** Copyright (C) 2021 KunoiSayami
 **
 ** This file is part of passive-DDNS and is released under
 ** the AGPL v3 License: https://www.gnu.org/licenses/agpl-3.0.txt
 **
 ** This program is free software: you can redistribute it and/or modify
 ** it under the terms of the GNU Affero General Public License as published by
 ** the Free Software Foundation, either version 3 of the License, or
 ** 6any later version.
 **
 ** This program is distributed in the hope that it will be useful,
 ** but WITHOUT ANY WARRANTY; without even the implied warranty of
 ** MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 ** GNU Affero General Public License for more details.
 **
 ** You should have received a copy of the GNU Affero General Public License
 ** along with this program. If not, see <https://www.gnu.org/licenses/>.
 */
mod cloudflare_api;
mod configparser;
mod openwrt;

use log::{debug, error, info};
use std::io::Write;
use std::time::Duration;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn get_ip_from_extern_uris(uris: &[String]) -> String {
    if uris.is_empty() {
        panic!("Uris should not empty");
    };
    for uri in uris {
        let result = reqwest::blocking::get(uri);
        match result {
            Ok(resp) => {
                return String::from(resp.text().unwrap().trim());
            }
            Err(_e) => continue,
        }
    }
    panic!("Can't get ip from extern uris");
}

fn get_current_ip(
    configure: &Option<Vec<String>>,
    openwrt_client: &Option<openwrt::api::Client>,
) -> String {
    let mut default_uris: Vec<String> = Default::default();
    if configure.is_none() {
        default_uris.push(String::from("https://api-ipv4.ip.sb/ip"));
    }

    let used_uris = match configure {
        Some(uris) => uris,
        None => &default_uris,
    };
    match openwrt_client {
        Some(client) => client.get_current_ip(),
        None => get_ip_from_extern_uris(used_uris),
    }
}

fn update_process(current_ip: &str, cloudflare: &cloudflare_api::api::Configure) -> bool {
    match cloudflare.update_dns_data(current_ip) {
        Ok(result) => {
            if result {
                info!("IP change detected, Changed dns ip to {}", current_ip);
            }
            true
        }
        Err(e) => {
            error!("Error in getting update from cloudflare: {:#}", e);
            false
        }
    }
}

fn main() {
    if std::env::args().into_iter().any(|x| x.eq("--version") || x.eq("-V")) {
        println!("passive-DDNS {}", VERSION);
        return
    }

    if std::env::args().into_iter()
        .any(|x| x.eq("--systemd"))
    {
        env_logger::Builder::from_default_env()
            .format(|buf, record| writeln!(buf, "[{}] - {}", record.level(), record.args()))
            .init();
    } else {
        env_logger::init();
    }


    let cfg_values = configparser::parser::get_configure_value("data/config.toml");
    let extern_uri = cfg_values.0;
    let cloudflare = cfg_values.1;
    let openwrt_client = cfg_values.2;
    let duration = cfg_values.3;
    loop {
        let current_ip = get_current_ip(&extern_uri, &openwrt_client);
        if !update_process(&current_ip, &cloudflare) {
            let mut v = true;
            for retry_times in &[5, 10, 60] {
                debug!("Sleep {}s for next request", retry_times);
                std::thread::sleep(Duration::from_secs(*retry_times));
                if update_process(&current_ip, &cloudflare) {
                    v = false;
                    break
                }
            }
            if v {
                panic!("Error while updating cloudflare ns DNS record");
            }
        }
        std::thread::sleep(Duration::from_secs(duration as u64));
    }
}
