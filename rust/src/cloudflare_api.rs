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
extern crate log;
pub(crate) mod api {
    use std::collections::HashMap;

    struct Zone {
        zone_id: String,
        domains: Vec<String>
    }

    impl Zone {
        pub fn new<T>(original_string: &T) -> Zone
            where T: Into<str> {
            let basic_re = regex::Regex::new(r"\'([a-f\d]+)\':\s*\[((\'[\w\.]+\',\s*)*\'[\w\.]+\')\]").unwrap();
            let domain_re = regex::Regex::new(r"([\w\.]+)").unwrap();
            let cap = basic_re.captures(original_string.into()).unwrap();
            let zone_id = String::from(&cap[1]);
            log::debug!("Processing zone: {}", zone_id);
            let mut domains: Vec<String> = vec![];
            for cap in domain_re.captures_iter(&cap[2]) {
                domains.push(String::from(&cap[1]));
                log::debug!("Push {} to {}", cap[1], zone_id);
            }
            Zone{zone_id, domains}
        }
    }

    struct Configure {
        zones: Vec<Zone>,
        api_token: String
    }

    impl Configure {
        pub fn new<T>(domains: T, api_token: T) -> Configure
            where T: Into<String> {
            let re = regex::Regex::new(r"\'([a-f\d]+\':\s*\[(\'[\w\.]+\',\s*)*\'[\w\.]+\'\])").unwrap();
            let original_domain_string = domains.into().as_str();
            let mut zones: Vec<Zone> = vec![];
            for cap in re.captures_iter(&original_domain_string) {
                zones.push(Zone::new(&cap[1]));
            }
            Configure{zones, api_token: api_token.into()}
        }
    }
}