/*
 ** Copyright (C) 2021-2024 KunoiSayami
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
#[cfg(test)]
mod test;

use crate::configparser::NameServer;
use clap::arg;
use log::{error, info, warn};
use std::io::Write as _;
use std::time::Duration;
use tap::TapFallible;

async fn update_process(current_ip: &str, name_server: &dyn NameServer) -> bool {
    name_server
        .update_dns_result(current_ip)
        .await
        .tap_ok(|result| {
            if *result {
                info!("IP change detected, Changed dns ip to {current_ip}");
            }
        })
        .tap_err(|e| error!("Error in getting update from cloudflare: {e:#}"))
        .is_ok()
}

async fn async_main(configure_file: &str) -> anyhow::Result<()> {
    let cfg_values = configparser::parser::get_configure_value(configure_file).await?;
    let (name_server, ip_source, duration) = cfg_values;
    loop {
        let current_ip = ip_source.get_current_ip().await?;
        if !update_process(&current_ip, &*name_server).await {
            let mut v = true;
            for retry_times in &[5, 10, 60] {
                warn!("Sleep {retry_times}s for next request");
                tokio::time::sleep(Duration::from_secs(*retry_times)).await;
                if update_process(&current_ip, &*name_server).await {
                    v = false;
                    break;
                }
            }
            if v {
                panic!("Error while updating NameServer DNS record");
            }
        }
        tokio::time::sleep(Duration::from_secs(duration as u64)).await;
    }
}

fn main() -> anyhow::Result<()> {
    let matches = clap::command!()
        .args(&[
            arg!([CONFIG_FILE] "Configure file location").default_value("config.toml"),
            arg!(--systemd "Systemd mode, cut time in log output"),
        ])
        .get_matches();

    if matches.get_flag("systemd") {
        env_logger::Builder::from_default_env()
            .format(|buf, record| writeln!(buf, "[{}] - {}", record.level(), record.args()))
            .init();
    } else {
        env_logger::init();
    }

    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async_main(
            matches.get_one::<String>("CONFIG_FILE").unwrap(),
        ))
}
