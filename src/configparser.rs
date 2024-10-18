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
pub(crate) mod parser {
    use crate::cloudflare_api::api::CloudFlareConfigure;
    use crate::configparser::{IPSource, NameServer};
    use crate::custom_target::api::{CustomUpstream, CustomUpstreamConfigure};
    use crate::openwrt::api::OpenWRTConfigure;
    use crate::{cloudflare_api, openwrt};
    use anyhow::anyhow;
    use log::{error, info};
    use serde::Deserialize;
    use tap::TapFallible;

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

    pub async fn get_configure_value<P: AsRef<Path> + std::fmt::Debug>(
        configure_path: P,
    ) -> anyhow::Result<(Box<dyn NameServer>, Box<dyn IPSource>, u32)> {
        let contents = tokio::fs::read_to_string(configure_path).await?;
        let configure: Configure =
            toml::from_str(&contents).tap_err(|e| error!("Read configure file error: {e:?}"))?;

        let openwrt_config = configure.get_openwrt_configure();
        let ip_source_client: Box<dyn IPSource> = if openwrt_config.get_status() {
            Box::new(openwrt::api::Client::new(
                openwrt_config.get_user().as_ref().cloned().unwrap(),
                openwrt_config.get_password().as_ref().cloned().unwrap(),
                openwrt_config.get_route().as_ref().cloned().unwrap(),
            ))
        } else {
            Box::new(DefaultIPSource::new(
                configure.get_account().get_extern_ip_uris(),
            ))
        };

        let cf_configure = configure.get_cloudflare_configure();
        let ns: Box<dyn NameServer> = if cf_configure.get_enabled() {
            Box::new(cloudflare_api::api::Configure::new(
                cf_configure.get_domain().clone().unwrap(),
                cf_configure.get_token().as_ref().unwrap(),
            ))
        } else {
            info!("Use custom upstream instead of cloudflare");
            Box::new(CustomUpstream::option_new(&configure).unwrap())
        };

        Ok((ns, ip_source_client, configure.get_account().get_duration()))
    }
    // TODO: ADD CUSTOM EXTERN IP URI

    pub struct DefaultIPSource {
        uris: Vec<String>,
    }

    impl DefaultIPSource {
        fn new(extern_uris: &Option<Vec<String>>) -> DefaultIPSource {
            Self {
                uris: match extern_uris {
                    Some(uris) => uris.clone(),
                    None => vec!["https://api-ipv4.ip.sb/ip".into()],
                },
            }
        }

        async fn fetch_ip_from_extern_uris(uris: &[String]) -> anyhow::Result<String> {
            assert!(!uris.is_empty(), "Uris should not empty");

            for i in 0..uris.len() {
                let result = reqwest::get(&uris[i]).await;
                match result {
                    Ok(resp) => {
                        return Ok(resp
                            .text()
                            .await
                            .map_err(|e| anyhow!("Fetch text error: {e:?}"))?
                            .trim()
                            .into());
                    }
                    Err(e) => {
                        if i == uris.len() - 1 {
                            return Err(e.into());
                        }
                    }
                }
            }
            unreachable!()
        }
    }

    #[async_trait::async_trait]
    impl IPSource for DefaultIPSource {
        async fn get_current_ip(&self) -> anyhow::Result<String> {
            Self::fetch_ip_from_extern_uris(&self.uris).await
        }
    }
}

#[async_trait::async_trait]
pub trait NameServer {
    async fn update_dns_result(&self, new_record: &str) -> Result<bool, reqwest::Error>;
}

/* #[async_trait::async_trait]
impl<F: ?Sized> NameServer for Box<F>
where
    F: NameServer,
{
    async fn update_dns_result(&self, new_record: &str) -> Result<bool, reqwest::Error> {
        (**self).update_dns_result(new_record)
    }
} */

#[async_trait::async_trait]
pub trait IPSource {
    async fn get_current_ip(&self) -> anyhow::Result<String>;
}
