#![feature(str_split_once)]
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

use log::{info, debug};


fn main() {
    env_logger::init();
    /*let mut config = configparser::ini::Ini::new();
    config.load("data/config.toml").unwrap();
    if config.getbool("openwrt", "enabled").unwrap_or(Some(false)).unwrap() {
        let client = openwrt::openwrt::Client::new(
            config.get("openwrt", "user").unwrap(),
            config.get("openwrt", "password").unwrap(),
            config.get("openwrt", "route").unwrap()
        );
        println!("{}", client.get_current_ip());
    }*/
    let configure = configparser::parser::load("data/config.toml").unwrap();
    dbg!(configure);
    /*let cloudflare = cloudflare_api::api::Configure::new(
        config.get("cloudflare", "domain").unwrap(),
        config.get("cloudflare", "token").unwrap()
    );*/
}
