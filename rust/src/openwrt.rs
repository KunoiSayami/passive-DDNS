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
mod openwrt {
    use std::collections::HashMap;
    use std::path::Path;
    use std::hash::Hash;

    struct Configure {
        user: String,
        password: String,
        basic_address: String
    }

    impl ConfigureBuilder {
        fn build(user: String, password: String, basic_address: String) -> Configure {
            Configure{user, password, basic_address}
        }
    }

    pub struct Client {
        configure: Configure,
        client: reqwest::Client,
        jar: HashMap<String, String>,
    }

    impl Client {
        fn check_login(basic_address: String) -> bool {
            log::debug!("Check login");
            let resp =
                match reqwest::blocking::get(format!("{}/cgi-bin/luci", basic_address)) {
                    Ok(req) => req,
                    Err(e) => {
                        panic!("Error with status code: {}", e);
                    }
                }
                    .status();
            resp.as_u16() == 200
        }

        pub fn do_login(&self) -> bool {
            let mut post_data: HashMap<&str, &String> = HashMap::new();
            post_data.insert("luci_username", &self.configure.user);
            post_data.insert("luci_password", &self.configure.password);
            let req = self.client.post(format!("{}/cgi-bin/luci", self.configure.basic_address))
                .json(&post_data).header("cookies", );

        }

        pub fn get_current_ip(&self) -> String {
            return String::new();
        }

        pub fn new(user: String, password: String, basic_address: String) -> OpenwrtClient {
            let client: reqwest::Client = reqwest::ClientBuilder::new()
                .cookie_store(true)
                .build()
                .unwarp();
            let session_path = Path::new(".data/.session");
            let session_string = if Path::exists(session_path) {
                let contents = std::fs::read_to_string(session_path);
                match contents {
                    Ok(content) => content,
                    Err(e) => String::new()
                }
            } else {
                String::new()
            };
            let configure = Configure{user, password, basic_address};
            OpenwrtClient{configure, client, jar: HashMap::new()}
        }
    }

}

#[cfg(test)]
fn main() {
    //println!("{}", openwrt::get_current_ip("127.0.0.1", "root", ""));
}