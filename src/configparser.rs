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
pub(crate) mod parser {
    use crate::cloudflare_api::api::CloudFlareConfigure;
    use crate::configparser::{IPSource, NameServer};
    use crate::custom_target::api::{CustomUpstream, CustomUpstreamConfigure};
    use crate::openwrt::api::OpenWRTConfigure;
    use crate::{cloudflare_api, openwrt};
    use log::info;
    use serde_derive::Deserialize;
    use std::borrow::Borrow;
    use std::path::Path;

    #[derive(Deserialize)]
    pub struct Configure {
        account: AccountConfigure,
        cloudflare: CloudFlareConfigure,
        openwrt: OpenWRTConfigure,
        custom_upstream: Option<CustomUpstreamConfigure>,
    }

    #[derive(Deserialize)]
    pub struct AccountConfigure {
        extern_ip_uris: Option<Vec<String>>,
        duration: Option<i32>,
    }

    impl AccountConfigure {
        fn get_extern_ip_uris(&self) -> &Option<Vec<String>> {
            &self.extern_ip_uris
        }

        fn get_duration(&self) -> u32 {
            self.duration.unwrap_or(600) as u32
        }
    }

    impl Configure {
        pub fn get_account(&self) -> &AccountConfigure {
            &self.account
        }

        pub fn get_custom_upstream(&self) -> &Option<CustomUpstreamConfigure> {
            &self.custom_upstream
        }

        pub fn get_openwrt_configure(&self) -> &OpenWRTConfigure {
            &self.openwrt
        }

        pub fn get_cloudflare_configure(&self) -> &CloudFlareConfigure {
            &self.cloudflare
        }
    }

    pub fn get_configure_value<T>(
        configure_path: T,
    ) -> (Box<dyn NameServer>, Box<dyn IPSource>, u32)
    where
        T: Into<String>,
    {
        let path_str = configure_path.into();
        let path = Path::new(path_str.as_str());
        if !Path::exists(path) {
            panic!("Configure file not exist!");
        }
        let contents = std::fs::read_to_string(path).unwrap();
        let contents_str = contents.as_str();
        let configure: Configure = toml::from_str(contents_str).unwrap();

        let openwrt_config = configure.get_openwrt_configure();
        let ip_source_client: Box<dyn IPSource> = if openwrt_config.get_status() {
            Box::new(openwrt::api::Client::new(
                openwrt_config
                    .get_user()
                    .borrow()
                    .as_ref()
                    .unwrap()
                    .to_string(),
                openwrt_config
                    .get_password()
                    .borrow()
                    .as_ref()
                    .unwrap()
                    .to_string(),
                openwrt_config
                    .get_route()
                    .borrow()
                    .as_ref()
                    .unwrap()
                    .to_string(),
            ))
        } else {
            Box::new(DefaultIPSource::new(
                configure.get_account().get_extern_ip_uris(),
            ))
        };

        let cf_configure = configure.get_cloudflare_configure();
        let ns: Box<dyn NameServer> = if cf_configure.get_enabled() {
            Box::new(cloudflare_api::api::Configure::new(
                cf_configure.get_domain().borrow().as_ref().unwrap(),
                cf_configure.get_token().borrow().as_ref().unwrap(),
            ))
        } else {
            info!("Use custom upstream instead of cloudflare");
            Box::new(CustomUpstream::option_new(&configure).unwrap())
        };

        (ns, ip_source_client, configure.get_account().get_duration())
    }
    // TODO: ADD CUSTOM EXTERN IP URI

    pub struct DefaultIPSource {
        uris: Vec<String>,
    }

    impl DefaultIPSource {
        fn new(extern_uris: &Option<Vec<String>>) -> DefaultIPSource {
            let mut default_uris: Vec<String> = Default::default();
            if extern_uris.is_none() {
                default_uris.push(String::from("https://api-ipv4.ip.sb/ip"));
            }

            let used_uris = match extern_uris {
                Some(uris) => uris,
                None => &default_uris,
            };
            Self {
                uris: used_uris.clone(),
            }
        }

        fn fetch_ip_from_extern_uris(uris: &[String]) -> Result<String, reqwest::Error> {
            assert!(!uris.is_empty(), "Uris should not empty");

            for i in 0..uris.len() {
                let result = reqwest::blocking::get(&uris[i]);
                match result {
                    Ok(resp) => {
                        return Ok(String::from(resp.text().unwrap().trim()));
                    }
                    Err(e) => {
                        if i == uris.len() - 1 {
                            return Err(e);
                        }
                    }
                }
            }
            unreachable!()
        }
    }

    impl IPSource for DefaultIPSource {
        fn get_current_ip(&self) -> Result<String, reqwest::Error> {
            Self::fetch_ip_from_extern_uris(&self.uris)
        }
    }
}

pub trait NameServer {
    fn update_dns_result(&self, new_record: &str) -> Result<bool, reqwest::Error>;
}

impl<F: ?Sized> NameServer for Box<F>
where
    F: NameServer,
{
    fn update_dns_result(&self, new_record: &str) -> Result<bool, reqwest::Error> {
        (**self).update_dns_result(new_record)
    }
}

pub trait IPSource {
    fn get_current_ip(&self) -> Result<String, reqwest::Error>;
}
