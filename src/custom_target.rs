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
pub(crate) mod api {
    use crate::configparser::NameServer;

    use serde::{Deserialize, Serialize};

    #[derive(Deserialize)]
    pub struct CustomUpstreamConfigure {
        #[allow(dead_code)]
        enabled: Option<bool>,
        upstream_url: String,
    }

    impl CustomUpstreamConfigure {
        pub fn get_upstream(&self) -> &String {
            &self.upstream_url
        }
    }

    pub struct CustomUpstream {
        upstream_url: String,
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct PostBody {
        data: String,
    }

    impl PostBody {
        pub fn new(s: &str) -> Self {
            Self {
                data: s.to_string(),
            }
        }
    }

    impl From<&str> for PostBody {
        fn from(s: &str) -> Self {
            Self::new(s)
        }
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct PostResponse {
        status: i32,
    }

    impl PostResponse {
        fn get_status(&self) -> i32 {
            self.status
        }
    }

    impl CustomUpstream {
        pub fn option_new(config: &crate::configparser::parser::Configure) -> Option<Self> {
            let upstream = config.get_custom_upstream();
            upstream.as_ref().map(|source| Self {
                upstream_url: source.get_upstream().clone(),
            })
        }
    }

    impl NameServer for CustomUpstream {
        fn update_dns_result(&self, new_record: &str) -> Result<bool, reqwest::Error> {
            let response: PostResponse = reqwest::blocking::ClientBuilder::new()
                .build()?
                .post(&self.upstream_url)
                .json(&PostBody::from(new_record))
                .send()?
                .json()?;
            Ok(response.get_status() == 200)
        }
    }
}
