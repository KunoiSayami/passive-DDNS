/*
 ** Copyright (C) 2021 KunoiSayami
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
#[cfg(test)]
mod test {
    use crate::configparser::parser::Configure;

    #[test]
    fn test_configure() {
        let content = r#"[account]
# extern_ip_uri = ""
# duration = 600

[cloudflare]
# enabled = true
token = "114514"
# For example, you want to set a.example.com and b.example.com and c.example.moe
# you should set like this
# example.com zone id is `ca3d180a0c66ac16da45fad9f7674292'
# example.moe zone id is `2d9437302c842804ab97f94e657c98af'
# domain = {'ca3d180a0c66ac16da45fad9f7674292': ['a.example.com', 'b.example.com'], '2d9437302c842804ab97f94e657c98af': ['c.example.moe']}

[[cloudflare.domain]]
zone_id = "ca3d180a0c66ac16da45fad9f7674292"
domains = ["a.example.com", "b.example.com"]

[[cloudflare.domain]]
zone_id = "2d9437302c842804ab97f94e657c98af"
domains = ["c.example.moe"]

[openwrt]
enabled = false
route = ""
user = ""
password = ""

[custom_upstream]
enabled = false
upstream_url = ""
token = ""
        "#;
        let configure: Configure = toml::from_str(content).unwrap();

        let cf = configure.get_cloudflare_configure();

        let cf2 = crate::cloudflare_api::api::Configure::new(
            cf.get_domain().clone().unwrap(),
            cf.get_token().clone().unwrap(),
        );

        for zone in cf2.zones() {
            if zone.zone_id().eq("ca3d180a0c66ac16da45fad9f7674292") {
                assert_eq!(zone.domains(), &vec!["a.example.com".to_string(), "b.example.com".to_string()])
            } else if zone.zone_id().eq("2d9437302c842804ab97f94e657c98af") {
                assert_eq!(zone.domains(), &vec!["c.example.moe".to_string()])
            } else {
                unreachable!()
            }
        }

    }
}