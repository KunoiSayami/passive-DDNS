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
pub(crate) mod api {
    use crate::configparser::NameServer;

    use serde::{Deserialize, Serialize};

    #[derive(Deserialize)]
    pub struct CustomUpstreamConfigure {
        #[allow(dead_code)]
        enabled: Option<bool>,
        upstream_url: String,
        token: Option<String>,
    }

    impl CustomUpstreamConfigure {
        pub fn get_upstream(&self) -> &String {
            &self.upstream_url
        }

        fn get_token(&self) -> &Option<String> {
            &self.token
        }
    }

    pub struct CustomUpstream {
        upstream_url: String,
        token: String,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct PostBody {
        data: String,
        token: String,
    }

    impl PostBody {
        pub fn new(s: &str, token: &str) -> Self {
            Self {
                data: s.to_string(),
                token: token.to_string(),
            }
        }
    }

    impl From<&str> for PostBody {
        fn from(s: &str) -> Self {
            Self::new(s, "")
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
                token: source.get_token().clone().unwrap_or_default(),
            })
        }

        pub fn to_post_body(&self, s: &str) -> PostBody {
            PostBody::new(s, self.token.as_str())
        }
    }

    #[async_trait::async_trait]
    impl NameServer for CustomUpstream {
        async fn update_dns_result(&self, new_record: &str) -> Result<bool, reqwest::Error> {
            let response: PostResponse = reqwest::ClientBuilder::new()
                .build()?
                .post(&self.upstream_url)
                .json(&self.to_post_body(new_record))
                .send()
                .await?
                .json()
                .await?;
            Ok(response.get_status() == 200)
        }
    }
}
