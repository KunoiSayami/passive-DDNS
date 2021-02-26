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
pub(crate) mod api {
    use serde_derive::{Deserialize, Serialize};
    use std::collections::HashMap;

    #[derive(Deserialize)]
    struct DNSRecord {
        id: String,
        zone_id: String,
        name: String,
        content: String,
        proxied: bool,
        ttl: i32,
    }

    impl DNSRecord {
        /*fn update_content<T>(&mut self, content: T)
            where T: Into<String> {
            self.content = content.into();
        }*/

        fn update_ns_record(&self, session: &reqwest::blocking::Client) -> bool {
            let resp = session
                .put(
                    format!(
                        "https://api.cloudflare.com/client/v4/zones/{}/dns_records/{}",
                        &self.zone_id, &self.id
                    )
                    .as_str(),
                )
                .json(&PutDNSRecord::from_dns_record(self))
                .send()
                .unwrap();
            resp.status().is_success()
        }
    }

    #[derive(Serialize)]
    struct PutDNSRecord {
        #[serde(rename = "type")]
        t: String,
        name: String,
        content: String,
        proxied: bool,
        ttl: i32,
    }

    impl PutDNSRecord {
        fn from_dns_record(dns_record: &DNSRecord) -> PutDNSRecord {
            PutDNSRecord {
                t: String::from('A'),
                name: String::from(&dns_record.name),
                content: String::from(&dns_record.content),
                proxied: dns_record.proxied,
                ttl: dns_record.ttl,
            }
        }
    }

    struct Zone {
        zone_id: String,
        domains: Vec<String>,
    }

    impl Zone {
        pub fn new<T>(original_string: T) -> Zone
        where
            T: Into<String>,
        {
            let basic_re =
                regex::Regex::new(r"'([a-f\d]+)':\s*\[(('[\w\.]+',\s*)*'[\w\.]+')\]").unwrap();
            let domain_re = regex::Regex::new(r"([\w\.]+)").unwrap();
            let original_string = original_string.into();
            log::debug!("Parse string: {}", &original_string);
            let cap = basic_re.captures(original_string.as_str()).unwrap();
            let zone_id = String::from(&cap[1]);
            log::debug!("Processing zone: {}", zone_id);
            let mut domains: Vec<String> = Default::default();
            for cap in domain_re.captures_iter(&cap[2]) {
                let domain = String::from(&cap[1]);
                domains.push(domain.clone());
                log::debug!("Push {} to {}", domain, zone_id);
            }
            Zone { zone_id, domains }
        }

        pub fn request_domain_record(
            &self,
            session: &reqwest::blocking::Client,
        ) -> Result<Vec<DNSRecord>, reqwest::Error> {
            let mut records: Vec<DNSRecord> = Default::default();
            //let form: HashMap::<_, _>::from_iter = (("test", "test"), ("test", "test"));

            for domain in &self.domains {
                let name = domain.as_str();
                let query: HashMap<&str, &str> =
                    [("type", "A"), ("name", name)].iter().cloned().collect();
                let resp = session
                    .get(
                        format!(
                            "https://api.cloudflare.com/client/v4/zones/{}/dns_records",
                            &self.zone_id
                        )
                        .as_str(),
                    )
                    .query(&query)
                    .send()?;
                let resp_json: serde_json::Value = resp.json().unwrap();
                let dns_record: DNSRecord =
                    serde_json::from_value(resp_json["result"][0].to_owned()).unwrap();
                records.push(dns_record);
            }
            Ok(records)
        }
    }

    pub struct Configure {
        zones: Vec<Zone>,
        session: reqwest::blocking::Client,
    }

    impl Configure {
        pub fn new<T>(domains: T, api_token: T) -> Configure
        where
            T: Into<String>,
        {
            let re = regex::Regex::new(r"('[a-f\d]+':\s*\[('[\w\.]+',\s*)*'[\w\.]+'\])").unwrap();
            let original_domain_string = domains.into();
            let mut zones: Vec<Zone> = Default::default();
            for cap in re.captures_iter(&original_domain_string.as_str()) {
                let domain_configure = String::from(&cap[1]);
                zones.push(Zone::new(domain_configure));
            }
            let api_token = api_token.into();
            let mut header_map = reqwest::header::HeaderMap::new();
            header_map.insert(
                "Authorization",
                format!("Bearer {}", api_token).parse().unwrap(),
            );
            header_map.insert(
                "Content-Type",
                String::from("application/json").parse().unwrap(),
            );
            header_map.insert("Connection", String::from("close").parse().unwrap());

            let session = reqwest::blocking::Client::builder()
                .default_headers(header_map)
                .build()
                .unwrap();

            Configure { zones, session }
        }

        fn fetch_data(&self) -> Result<Vec<DNSRecord>, reqwest::Error> {
            let mut result: Vec<DNSRecord> = Default::default();
            for zone in &self.zones {
                result.extend(zone.request_domain_record(&self.session)?);
            }
            Ok(result)
        }

        pub fn update_dns_data(&self, new_data: &str) -> Result<bool, reqwest::Error> {
            let mut need_updated: Vec<DNSRecord> = Default::default();
            for record in self.fetch_data()? {
                if !record.content.eq(new_data) {
                    let mut mut_record = record;
                    mut_record.content = String::from(new_data);
                    need_updated.push(mut_record);
                }
            }
            let rt = !need_updated.is_empty();
            for record in need_updated {
                record.update_ns_record(&self.session);
            }
            Ok(rt)
        }
    }
}
