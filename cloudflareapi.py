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
import logging
import time
from configparser import ConfigParser
from dataclasses import dataclass
from typing import Dict, Generator, List, TypeVar

import requests

FixedType = TypeVar('FixedType', bool, str, int)

logger: logging.Logger = logging.getLogger('CloudFlareApi')
logger.setLevel(logging.INFO)
retry_delay = [60, 20, 10, 0]

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

	def update_dns(self, session: requests.Session, change_to_ip: str, retries: int=3) -> None:
		self.content = change_to_ip
		time.sleep(retry_delay[retries])
		try:
			r = session.put(f'https://api.cloudflare.com/client/v4/zones/{self.zone_id}/dns_records/{self.id}', json=self.get_params(), timeout=30)
			r.raise_for_status()
			r.close()
		except requests.exceptions.Timeout:
			if retries > 0:
				logger.error('Got Timeout error during update dns record (retries: %d)', retries)
				self.update_dns(session, change_to_ip, retries - 1)
			else:
				logger.critical('Can\'t update dns, raise it')
				raise

	def get_params(self) -> Dict[str, FixedType]:
		return dict(type='A', name=self.name, content=self.content, proxied=self.proxied, ttl=self.ttl)


class CloudFlareApi:
	def __init__(self, config: ConfigParser):
		self.domains: Dict[str, List[str]]
		self.api_token: str = config.get('cloudflare', 'token')
		if config.has_option('cloudflare', 'header_domain'):
			self.domains = ast.literal_eval(config.get('cloudflare', 'header_domain'))
		else:
			self.domains = ast.literal_eval(config.get('cloudflare', 'ipv4_domain'))
		self.session = requests.Session()
		self.session.headers.update({
			'Authorization': f'Bearer {self.api_token}',
			'Content-Type': 'application/json',
			'Connection': 'close'
			})

	def get_domain_record(self, domain: str, name: str, retries: int=3) -> Dict[str, FixedType]:
		time.sleep(retry_delay[retries])
		try:
			r = self.session.get(f'https://api.cloudflare.com/client/v4/zones/{domain}/dns_records', params={'type': 'A', 'name': name}, timeout=30)
			r.raise_for_status()
			result = r.json()['result'][0]
			r.close()
			return result
		except requests.exceptions.Timeout:
			if retries > 0:
				logger.error('Got timeout error (retries: %d)', retries)
				return self.get_domain_record(domain, name, retries - 1)
			else:
				logger.critical('Can\'t fetch dns records, raise it')
				raise

	def get_records(self) -> Generator[DNSRecord, None, None]:
		for key, item in self.domains.items():
			for x in item:
				yield DNSRecord.create(self.get_domain_record(key, x))

	def update_records(self, change_to_ip: str) -> bool:
		is_update: bool = False
		for record in self.get_records():
			if record.content != change_to_ip:
				is_update = True
				record.update_dns(self.session, change_to_ip)
		return is_update

	def close(self) -> None:
		self.session.close()


def test():
	config = ConfigParser()
	config.read('data/config.ini')
	c = CloudFlareApi(config)
	for record in c.get_records():
		print(record)
	c.close()

if __name__ == "__main__":
	test()
