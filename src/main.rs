/*
 ** Copyright (C) 2021 KunoiSayami
 **
 ** This file is part of passive-DDNS and is released under
 ** the AGPL v3 License: https://www.gnu.org/licenses/agpl-3.0.txt
 **
 ** This program is free software: you can redistribute it and/or modify
 ** it under the terms of the GNU Affero General Public License as published by
 ** the Free Software Foundation, either version 3 of the License, or
 ** any later version.
 **
 ** This program is distributed in the hope that it will be useful,
 ** but WITHOUT ANY WARRANTY; without even the implied warranty of
 ** MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 ** GNU Affero General Public License for more details.
 **
 ** You should have received a copy of the GNU Affero General Public License
 ** along with this program. If not, see <https://www.gnu.org/licenses/>.
 */
mod openwrt;
mod cloudflare_api;
mod configparser;

use log::info;

fn get_ip_from_extern_uris(uris: &Vec<String>) -> String {
    if uris.is_empty() {
        panic!("Uris should not empty");
    };
    for uri in uris {
        let result = reqwest::blocking::get(uri);
        match result {
            Ok(resp) => {
                return String::from(resp.text().unwrap().trim());
            }
            Err(_e) => continue
        }
    }
    panic!("Can't get ip from extern uris");
}

fn get_current_ip(configure: &Option<Vec<String>>,
                  openwrt_client: Option<openwrt::api::Client>) -> String {
    let mut default_uris: Vec<String> = Default::default();
    if configure.is_none() {
        default_uris.push(String::from("https://api-ipv4.ip.sb/ip"));
    }

    let used_uris = match configure {
        Some(uris) => uris,
        None => &default_uris
    };
    match openwrt_client {
        Some(client) => client.get_current_ip(),
        None => get_ip_from_extern_uris(used_uris)
    }
}

fn main() {
    env_logger::init();
    let cfg_values = configparser::parser::get_configure_value("data/config.toml");
    let extern_uri = cfg_values.0;
    let cloudflare = cfg_values.1;
    let openwrt_client = cfg_values.2;
    let current_ip = get_current_ip(&extern_uri, openwrt_client);
    info!("Current ip: {}", current_ip);
    cloudflare.update_dns_data(current_ip);
}
