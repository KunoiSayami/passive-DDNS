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
    use crate::{cloudflare_api, openwrt};
    use serde_derive::Deserialize;
    use std::path::Path;

    #[derive(Deserialize)]
    pub struct Configure {
        pub(crate) account: AccountConfigure,
        pub(crate) cloudflare: CloudFlareConfigure,
        pub(crate) openwrt: OpenWRTConfigure,
    }

    #[derive(Deserialize)]
    pub struct AccountConfigure {
        pub(crate) extern_ip_uris: Option<Vec<String>>,
        duration: Option<i32>,
    }

    #[derive(Deserialize)]
    pub struct CloudFlareConfigure {
        pub(crate) token: Option<String>,
        pub(crate) domain: Option<String>,
    }

    #[derive(Deserialize)]
    pub struct OpenWRTConfigure {
        enabled: bool,
        route: Option<String>,
        user: Option<String>,
        password: Option<String>,
    }

    pub fn get_configure_value<T>(
        configure_path: T,
    ) -> (
        Option<Vec<String>>,
        cloudflare_api::api::Configure,
        Option<openwrt::api::Client>,
        u32,
    )
    where
        T: Into<String>,
    {
        let path_str = configure_path.into();
        let path = Path::new(path_str.as_str());
        if !Path::exists(&path) {
            panic!("Configure file not exist!");
        }
        let contents = std::fs::read_to_string(path).unwrap();
        let contents_str = contents.as_str();
        let configure: Configure = toml::from_str(contents_str).unwrap();
        let openwrt_client = if configure.openwrt.enabled {
            Some(openwrt::api::Client::new(
                configure.openwrt.user.unwrap(),
                configure.openwrt.password.unwrap(),
                configure.openwrt.route.unwrap(),
            ))
        } else {
            None
        };
        (
            configure.account.extern_ip_uris,
            cloudflare_api::api::Configure::new(
                configure.cloudflare.domain.unwrap(),
                configure.cloudflare.token.unwrap(),
            ),
            openwrt_client,
            configure.account.duration.unwrap_or(600) as u32,
        )
    }
    // TODO: ADD CUSTOM EXTERN IP URI
}
