# -*- coding: utf-8 -*-
# cloudflareapi.py
# Copyright (C) 2020 KunoiSayami and contributors
#
# This module is part of passive-DDNS and is released under
# the AGPL v3 License: https://www.gnu.org/licenses/agpl-3.0.txt
#
# This program is free software: you can redistribute it and/or modify
# it under the terms of the GNU Affero General Public License as published by
# the Free Software Foundation, either version 3 of the License, or
# any later version.
#
# This program is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
# GNU Affero General Public License for more details.
#
# You should have received a copy of the GNU Affero General Public License
# along with this program. If not, see <https://www.gnu.org/licenses/>.
import ast
from configparser import ConfigParser
from dataclasses import dataclass
from typing import Dict, Generator, List, TypeVar

import requests

import absddns
from absddns import AbstractDDNS

FixedType = TypeVar('FixedType', bool, str, int)

@dataclass
class DNSRecord:
	id: str
	zone_id: str
	name: str
	content: str
	proxied: bool
	ttl: int

	@classmethod
	def create(cls, rt: Dict[str, FixedType]) -> 'DNSRecord':
		self = cls(rt['id'], rt['zone_id'], rt['name'], rt['content'], rt['proxied'], rt['ttl'])
		return self

	def update_dns(self, session: requests.Session, change_to_ip: str) -> None:
		self.content = change_to_ip
		r = session.put(f'https://api.cloudflare.com/client/v4/zones/{self.zone_id}/dns_records/{self.id}', data=self.get_params())
		r.raise_for_status()
	
	def get_params(self) -> Dict[str, FixedType]:
		return dict(type='A', name=self.name, content=self.content, proxied=self.proxied, ttl=self.ttl)

class CloudFlareApi:
	def __init__(self, config: ConfigParser):
		self.api_token: str = config.get('cloudflare', 'token')
		self.domains: Dict[str, List[str]] = ast.literal_eval(config.get('cloudflare', 'header_domain'))
		self.session = requests.Session()
		self.session.headers.update({
			'Authorization': f'Bearer {self.api_token}',
			'Content-Type': 'application/json'
			})

	def get_domain_record(self, domain: str, name: str) -> Dict[str, FixedType]:
		r = self.session.get(f'https://api.cloudflare.com/client/v4/zones/{domain}/dns_records', params={'type': 'A', 'name': name})
		r.raise_for_status()
		return r.json()['result'][0]

	def get_records(self) -> Generator[DNSRecord, None, None]:
		for key, item in self.domains.items():
			for x in item:
				yield DNSRecord.create(self.get_domain_record(key, x))

	def update_records(self, change_to_ip: str) -> None:
		for record in self.get_records():
			record.update_dns(self.session, change_to_ip)

	def close(self) -> None:
		self.session.close()

class CloudFlareDDNS(AbstractDDNS):
	def __init__(self):
		super().__init__()
		config = ConfigParser()
		config.read('data/config.ini')
		self.cloudflare_api = CloudFlareApi(config)
	
	def do_ip_update(self, now_ip: str) -> None:
		self.cloudflare_api.update_records(now_ip)
	
	def handle_reload(self) -> None:
		pass

	def close(self) -> None:
		self.cloudflare_api.close()

def test():
	config = ConfigParser()
	config.read('data/config.ini')
	c = CloudFlareApi(config)
	for record in c.get_records():
		print(record)
	c.close()

def main():
	pass

if __name__ == "__main__":
	main() 
