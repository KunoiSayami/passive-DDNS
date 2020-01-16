# -*- coding: utf-8 -*-
# libopenwrt.py
# Copyright (C) 2020 KunoiSayami
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
import requests
import time

class OpenWRTHelper:
	class OtherError(Exception): pass

	def __init__(self, route_ip: str, user: str, password: str):
		self.route_web = f'http://{route_ip}'
		self.user = user
		self.password = password
		self.Session = requests.Session()
		self.session_str = self._read_session_str()

	def _write_session_str(self):
		with open('data/.session', 'w') as fout:
			fout.write(self.session_str)

	def _read_session_str(self) -> str:
		with open('data/.session', 'r') as fin:
			self.session_str = fin.read()
			self.Session.cookies.update({'sysauth': self.session_str})
		return self.session_str

	def check_login(self) -> bool:
		return self.Session.get(self.route_web + '/cgi-bin/luci/').status_code == 200

	def do_login(self) -> bool:
		if self.check_login():
			return True
		r = self.Session.post(f'{self.route_web}/cgi-bin/luci',
				data={'luci_username': self.user, 'luci_password': self.password},
				allow_redirects=False)
		r.raise_for_status()
		self.session_str = self.Session.cookies.get('sysauth')
		self._write_session_str()
		return self.check_login()

	def get_ip(self) -> str:
		if not self.check_login():
			self.do_login()
		r = self.Session.post(f'{self.route_web}/ubus/?{time.time()}',
				json=[{'jsonrpc': '2.0', 'id': 1, 'method': 'call', 'params': [self.session_str, 'network.interface', 'dump', {}]}])
		raw_data = r.json()[0]
		if raw_data.get('error') is None:
			for interface in raw_data.get('result')[1].get('interface'):
				if interface.get('interface') == 'wan':
					return interface.get('ipv4-address')[0].get('address')
		else:
			raise OpenWRTHelper.OtherError()

if __name__ == "__main__":
	print(OpenWRTHelper('127.0.0.1', 'root', '').get_ip())