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
pub(crate) mod api {
    use crate::configparser::IPSource;
    use log::{error, warn};
    use reqwest::header::HeaderMap;
    use reqwest::StatusCode;
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;
    use std::path::Path;
    use tap::TapFallible;
    use tokio::io::AsyncWriteExt as _;
    const DEFAULT_SESSION_FILE: &str = ".session";

    pub fn get_current_timestamp() -> u128 {
        let start = std::time::SystemTime::now();
        let since_the_epoch = start
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Time went backwards");
        since_the_epoch.as_millis()
    }

    #[derive(Serialize, Deserialize)]
    struct Cookie {
        key: String,
        value: String,
    }

    impl Cookie {
        fn new<T>(key: T, value: T) -> Cookie
        where
            T: Into<String>,
        {
            Cookie {
                key: key.into(),
                value: value.into(),
            }
        }

        fn to_entry_string(&self) -> String {
            format!("{}={}", &self.key, &self.value)
        }

        fn load_from_entry(entry: reqwest::cookie::Cookie) -> Cookie {
            log::debug!("{}={}", entry.name(), entry.value());
            Cookie::new(entry.name(), entry.value())
        }
    }

    #[derive(Serialize, Deserialize, Default)]
    struct Cookies {
        cookies: Vec<Cookie>,
    }

    impl Cookies {
        fn from_response(response: &reqwest::Response) -> Cookies {
            let mut cookies: Vec<Cookie> = Default::default();
            log::debug!("Cookie length: {:?}", response.headers());
            for cookie in response.cookies() {
                log::debug!("{cookie:?}");
                cookies.push(Cookie::load_from_entry(cookie))
            }
            Cookies { cookies }
        }

        fn to_header_string(&self) -> String {
            let mut cookies: Vec<String> = Default::default();
            for cookie in &self.cookies {
                cookies.push(cookie.to_entry_string())
            }
            cookies.join("; ")
        }

        async fn save_cookies(resp: reqwest::Response) -> std::io::Result<()> {
            let mut session_file = tokio::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(DEFAULT_SESSION_FILE)
                .await?;

            let content = Cookies::from_response(&resp);
            session_file
                .write_all(serde_json::to_string(&content)?.as_bytes())
                .await
        }

        async fn load_cookies() -> anyhow::Result<Cookies> {
            //let mut cookies= Cookies::new();
            log::debug!("Loading cookies");
            if <&str as AsRef<Path>>::as_ref(&DEFAULT_SESSION_FILE).exists() {
                let txt = tokio::fs::read_to_string(DEFAULT_SESSION_FILE)
                    .await
                    .tap_err(|e| warn!("Read cookie file error: {e:?}"))?;
                Ok(serde_json::from_str(&txt)
                    .tap_err(|e| warn!("Deserialize cookie file error: {e:?}"))?)
            } else {
                Err(anyhow::anyhow!(
                    "Cookie file not found, fallback to default"
                ))
            }
        }

        fn len(&self) -> usize {
            self.cookies.len()
        }
    }

    struct Configure {
        user: String,
        password: String,
        basic_address: String,
    }

    impl Configure {
        fn new<T>(user: T, password: T, basic_address: T) -> Configure
        where
            T: Into<String>,
        {
            Configure {
                user: user.into(),
                password: password.into(),
                basic_address: basic_address.into(),
            }
        }
    }

    pub struct Client {
        configure: Configure,
        client: reqwest::Client,
    }

    impl Client {
        async fn check_login(&self, cookies: &Cookies) -> anyhow::Result<bool> {
            log::debug!("Check login");
            let mut header_map = HeaderMap::new();
            if cookies.len() > 0 {
                header_map.append(
                    "cookie",
                    cookies
                        .to_header_string()
                        .parse()
                        .tap_err(|e| error!("Invalid header value: {e:?}"))?,
                );
            }

            let request_builder = self
                .client
                .get(format!("{}/cgi-bin/luci/", &self.configure.basic_address).as_str())
                .headers(header_map);

            Ok(request_builder.send().await?.status() == 200)
        }

        async fn do_login(&self, cookies: &Cookies) -> anyhow::Result<bool> {
            //let cookies = Cookies::load_cookies();
            if self.check_login(cookies).await? {
                return Ok(true);
            }
            log::debug!("Trying re-login");
            let mut post_data: HashMap<&str, &String> = HashMap::new();
            post_data.insert("luci_username", &self.configure.user);
            post_data.insert("luci_password", &self.configure.password);
            let resp = self
                .client
                .post(format!("{}/cgi-bin/luci", self.configure.basic_address).as_str())
                .form(&post_data)
                //.header("cookie", cookies.to_entry_string())
                .send()
                .await?;
            let status_code = resp.status();
            log::debug!("Status code: {status_code}");
            if status_code == StatusCode::OK || status_code == StatusCode::FOUND {
                Cookies::save_cookies(resp)
                    .await
                    .tap_err(|e| error!("Save cookie error: {e:?}"))?;
                Ok(false)
            } else {
                error!("Error code: {status_code}");
                Err(anyhow::anyhow!("Not login because status code"))
            }
        }

        pub fn new<T>(user: T, password: T, basic_address: T) -> Client
        where
            T: Into<String>,
        {
            let client = reqwest::ClientBuilder::new()
                .cookie_store(true)
                .redirect(reqwest::redirect::Policy::none())
                .build()
                .unwrap();
            let configure = Configure::new(user, password, basic_address);
            Client { configure, client }
        }
    }

    #[async_trait::async_trait]
    impl IPSource for Client {
        async fn get_current_ip(&self) -> anyhow::Result<String> {
            let cookies = Cookies::load_cookies().await.unwrap_or_default();
            let need_load_cookie = self.do_login(&cookies).await?;

            let mut header_map = HeaderMap::new();
            if need_load_cookie {
                header_map.append(
                    "cookie",
                    cookies
                        .to_header_string()
                        .parse()
                        .tap_err(|e| error!("Invalid header value: {e:?}"))?,
                );
            }

            let resp = self
                .client
                .get(
                    format!(
                        "{}/cgi-bin/luci/?status=1&_={}",
                        &self.configure.basic_address,
                        get_current_timestamp()
                    )
                    .as_str(),
                )
                .headers(header_map)
                .send()
                .await?;

            let content: serde_json::Value = resp
                .json()
                .await
                .tap_err(|e| error!("Parse json error: {e:?}"))?;

            Ok(String::from(
                content["wan"]["ipaddr"].as_str().unwrap_or_else(|| {
                    error!("Can't found address {content:?}");
                    "N/A"
                }),
            ))
        }
    }

    #[derive(Deserialize)]
    pub struct OpenWRTConfigure {
        enabled: bool,
        route: Option<String>,
        user: Option<String>,
        password: Option<String>,
    }

    impl OpenWRTConfigure {
        pub fn get_status(&self) -> bool {
            self.enabled
        }

        pub fn get_route(&self) -> &Option<String> {
            &self.route
        }

        pub fn get_user(&self) -> &Option<String> {
            &self.user
        }

        pub fn get_password(&self) -> &Option<String> {
            &self.password
        }
    }
}
