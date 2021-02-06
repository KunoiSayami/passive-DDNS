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
    use std::collections::HashMap;
    use std::path::Path;
    use std::io::Write;
    use serde::{Serialize, Deserialize};
    use reqwest::header::HeaderMap;

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
        value: String
    }

    impl Cookie {
        fn new<T> (key: T, value: T) -> Cookie
            where T: Into<String> {
            Cookie{key: key.into(), value: value.into()}
        }

        fn paste(&self) -> String {
            format!("{}={}", &self.key, &self.value)
        }

        fn load_from_entry(entry: reqwest::cookie::Cookie) -> Cookie {
            Cookie::new(entry.name(), entry.value())
        }
    }


    #[derive(Serialize, Deserialize)]
    struct Cookies {
        cookies: Vec<Cookie>
    }

    impl Cookies {
        fn new() -> Cookies {
            Cookies{cookies: Default::default()}
        }

        fn from_response(response: &reqwest::blocking::Response) -> Cookies {
            let mut cookies: Vec<Cookie> = Default::default();
            for cookie in response.cookies() {
                cookies.push(Cookie::load_from_entry(cookie))
            }
            Cookies{cookies}
        }

        fn to_string(&self) -> String {
            let mut cookies: Vec<String> = Default::default();
            for cookie in &self.cookies {
                cookies.push(cookie.paste())
            }
            cookies.join("; ")
        }

        fn save_cookies(resp: reqwest::blocking::Response) -> std::io::Result<()> {
            let mut session_file = std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open("data/.session")
                .unwrap();
            let content = Cookies::from_response(&resp);
            session_file.write_all(serde_json::to_string(&content)?.as_bytes())
        }

        fn load_cookies() -> Cookies {
            //let mut cookies= Cookies::new();
            let session_path = Path::new(".data/.session");
            if Path::exists(session_path) {
                match std::fs::read_to_string(session_path) {
                    Ok(content) => serde_json::from_str(content.as_str())
                        .unwrap_or_else(|_| Cookies::new()),
                    Err(_e) => Cookies::new()
                }
            } else {
                Cookies::new()
            }
        }

        fn len(&self) -> usize {
            self.cookies.len()
        }
    }

    struct Configure {
        user: String,
        password: String,
        basic_address: String
    }

    impl Configure {
        fn new<T>(user: T, password: T, basic_address: T) -> Configure
            where T: Into<String> {
            Configure{
                user: user.into(),
                password: password.into(),
                basic_address: basic_address.into()
            }
        }
    }

    pub struct Client {
        configure: Configure,
        client: reqwest::blocking::Client
    }

    impl Client {
        fn check_login(&self, cookies: &Cookies) -> bool {
            log::debug!("Check login");
            let mut header_map = HeaderMap::new();
            if cookies.len() > 0 {
                header_map.append("cookies", cookies.to_string().parse().unwrap());
            }

            let request_builder = self.client.get(format!("{}/cgi-bin/luci/",
                                                          &self.configure.basic_address).as_str())
                .headers(header_map);

            let resp =
                match request_builder.send() {
                    Ok(req) => req,
                    Err(e) => {
                        panic!("Error with status code: {}", e);
                    }
                }
                    .status();
            resp.as_u16() == 200
        }

        fn do_login(&self, cookies: &Cookies) -> Result<bool, reqwest::blocking::Response> {
            //let cookies = Cookies::load_cookies();
            if self.check_login(&cookies) {
                return Ok(true)
            }
            let mut post_data: HashMap<&str, &String> = HashMap::new();
            post_data.insert("luci_username", &self.configure.user);
            post_data.insert("luci_password", &self.configure.password);
            let resp = self.client.post(format!("{}/cgi-bin/luci", self.configure.basic_address)
                .as_str())
                .form(&post_data)
                //.header("cookies", cookies.to_string())
                .send()
                .unwrap();
            let status_code = resp.status().as_u16();
            log::debug!("Status code: {}", status_code);
            if status_code == 200 {
                Cookies::save_cookies(resp).unwrap();
                Ok(false)
            } else {
                log::error!("Error code: {}", status_code);
                Err(resp)
            }
        }

        pub fn get_current_ip(&self) -> String {
            let cookies = Cookies::load_cookies();
            let need_load_cookie = self.do_login(&cookies).unwrap();

            let mut header_map = HeaderMap::new();
            if need_load_cookie {
                header_map.append("cookies", cookies.to_string().parse().unwrap());
            }

            let resp = self.client.get(format!("{}/cgi-bin/luci/?status=1&_={}",
                                               &self.configure.basic_address,
                                               get_current_timestamp()).as_str())
                .headers(header_map)
                .send()
                .unwrap();

            let content: serde_json::Value = resp.json().unwrap();

            String::from(content["wan"]["ipaddr"].as_str().unwrap())
        }

        pub fn new<T>(user: T, password: T, basic_address: T) -> Client
            where T: Into<String> {
            let client = reqwest::blocking::ClientBuilder::new()
                .cookie_store(true)
                .redirect(reqwest::redirect::Policy::limited(5))
                .build()
                .unwrap();
            let configure = Configure::new(user, password, basic_address);
            Client{configure, client}
        }
    }
}
