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

fn get_current_ip(configure: &Option<String>,
                  openwrt_client: Option<openwrt::api::Client>) -> String {
    match openwrt_client {
        Some(client) => client.get_current_ip(),
        None => String::from(reqwest::blocking::get(
            configure.as_ref().unwrap_or(&String::from("https://api-ipv4.ip.sb/ip")).as_str()
        ).unwrap()
            .text()
            .unwrap()
            .trim())
    }
}

fn main() {
    env_logger::init();
    let configure = configparser::parser::load("data/config.toml").unwrap();
    let cloudflare = cloudflare_api::api::Configure::new(
        configure.cloudflare.domain.unwrap(),
        configure.cloudflare.token.unwrap()
    );
    let openwrt_client = if configure.openwrt.enabled {
        Some(openwrt::api::Client::new(
            configure.openwrt.user.unwrap(),
            configure.openwrt.password.unwrap(),
            configure.openwrt.route.unwrap()
        ))
    } else {
        None
    };
    let current_ip = get_current_ip(&configure.account.extern_ip_uri, openwrt_client);
    info!("Current ip: {}", current_ip);
    cloudflare.update_dns_data(current_ip);
}
