/*
 ** Copyright (C) 2021-2024 KunoiSayami
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
const DEFAULT_TIMEOUT: u64 = 10;
pub(crate) mod api {
    use super::DEFAULT_TIMEOUT;
    use crate::configparser::NameServer;
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;
    use std::time::Duration;

    #[derive(Deserialize)]
    pub(crate) struct DNSRecord {
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

        async fn update_ns_record(&self, session: &reqwest::Client) -> reqwest::Result<bool> {
            let resp = session
                .put(format!(
                    "https://api.cloudflare.com/client/v4/zones/{}/dns_records/{}",
                    self.zone_id, self.id
                ))
                .json(&PutDNSRecord::from_dns_record(self))
                .send()
                .await?;
            Ok(resp.status().is_success())
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

    #[derive(Deserialize, Clone, Debug)]
    pub struct Zone {
        zone_id: String,
        domains: Vec<String>,
    }

    #[cfg(test)]
    impl Zone {
        pub(crate) fn zone_id(&self) -> &str {
            &self.zone_id
        }
        pub(crate) fn domains(&self) -> &Vec<String> {
            &self.domains
        }
    }

    impl Zone {
        pub(crate) async fn request_domain_record(
            &self,
            session: &reqwest::Client,
        ) -> Result<Vec<DNSRecord>, reqwest::Error> {
            let mut records: Vec<DNSRecord> = Default::default();
            //let form: HashMap::<_, _>::from_iter = (("test", "test"), ("test", "test"));

            for domain in &self.domains {
                let query: HashMap<&str, &str> =
                    [("type", "A"), ("name", &domain)].iter().cloned().collect();
                let resp = session
                    .get(
                        format!(
                            "https://api.cloudflare.com/client/v4/zones/{}/dns_records",
                            &self.zone_id
                        )
                        .as_str(),
                    )
                    .query(&query)
                    .send()
                    .await?;
                let resp_json: serde_json::Value = resp.json().await.unwrap();
                let dns_record: DNSRecord =
                    serde_json::from_value(resp_json["result"][0].to_owned()).unwrap();
                records.push(dns_record);
            }
            Ok(records)
        }
    }

    pub struct Configure {
        zones: Vec<Zone>,
        session: reqwest::Client,
    }

    impl Configure {
        pub fn new<T: Into<String>>(domains: Vec<Zone>, api_token: T) -> Configure {
            let api_token = api_token.into();
            let mut header_map = reqwest::header::HeaderMap::new();
            header_map.insert(
                "Authorization",
                format!("Bearer {api_token}").parse().unwrap(),
            );
            header_map.insert("Content-Type", "application/json".parse().unwrap());
            header_map.insert("Connection", "close".parse().unwrap());

            let session = reqwest::Client::builder()
                .default_headers(header_map)
                .timeout(Duration::from_secs(DEFAULT_TIMEOUT))
                .connect_timeout(Duration::from_secs(DEFAULT_TIMEOUT))
                .build()
                .unwrap();

            Configure {
                zones: domains.clone(),
                session,
            }
        }

        async fn fetch_data(&self) -> Result<Vec<DNSRecord>, reqwest::Error> {
            let mut result = Vec::new();
            for zone in &self.zones {
                result.extend(zone.request_domain_record(&self.session).await?);
            }
            Ok(result)
        }

        #[cfg(test)]
        pub(crate) fn zones(&self) -> &Vec<Zone> {
            &self.zones
        }
    }

    #[derive(Deserialize)]
    pub struct CloudFlareConfigure {
        enabled: Option<bool>,
        token: Option<String>,
        domain: Option<Vec<Zone>>,
    }

    impl CloudFlareConfigure {
        /// Default is true
        pub fn get_enabled(&self) -> bool {
            self.enabled.unwrap_or(true)
        }
        pub fn get_token(&self) -> &Option<String> {
            &self.token
        }
        pub fn get_domain(&self) -> &Option<Vec<Zone>> {
            &self.domain
        }
    }

    #[async_trait::async_trait]
    impl NameServer for Configure {
        async fn update_dns_result(&self, new_record: &str) -> Result<bool, reqwest::Error> {
            let mut need_updated = Vec::new();
            for record in self.fetch_data().await? {
                if !record.content.eq(new_record) {
                    let mut mut_record = record;
                    mut_record.content = String::from(new_record);
                    need_updated.push(mut_record);
                }
            }
            let rt = !need_updated.is_empty();
            for record in need_updated {
                record.update_ns_record(&self.session).await?;
            }
            Ok(rt)
        }
    }
}
