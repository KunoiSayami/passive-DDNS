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
mod custom_target;
mod openwrt;

use crate::configparser::NameServer;
use log::{error, info, warn};
use std::io::Write;
use std::time::Duration;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn update_process(current_ip: &str, name_server: &dyn NameServer) -> bool {
    match name_server.update_dns_result(current_ip) {
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
    if std::env::args()
        .into_iter()
        .any(|x| x.eq("--version") || x.eq("-V"))
    {
        println!("passive-DDNS {}", VERSION);
        return;
    }

    if std::env::args().into_iter().any(|x| x.eq("--systemd")) {
        env_logger::Builder::from_default_env()
            .format(|buf, record| writeln!(buf, "[{}] - {}", record.level(), record.args()))
            .init();
    } else {
        env_logger::init();
    }

    let cfg_values = configparser::parser::get_configure_value("data/config.toml");
    let (name_server, ip_source, duration) = cfg_values;
    loop {
        let current_ip = ip_source.get_current_ip().unwrap();
        if !update_process(&current_ip, &name_server) {
            let mut v = true;
            for retry_times in &[5, 10, 60] {
                warn!("Sleep {}s for next request", retry_times);
                std::thread::sleep(Duration::from_secs(*retry_times));
                if update_process(&current_ip, &name_server) {
                    v = false;
                    break;
                }
            }
            if v {
                panic!("Error while updating Nameserver DNS record");
            }
        }
        std::thread::sleep(Duration::from_secs(duration as u64));
    }
}
