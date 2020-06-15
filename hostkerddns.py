# -*- coding: utf-8 -*-
# hosterddns.py
# Copyright (C) 2018-2020 KunoiSayami and contributors
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
import logging
import os
from configparser import ConfigParser
from typing import List

import hostkerapi
from absddns import AbstractDDNS


class HostkerDDNS(AbstractDDNS):
	def __init__(self):
		super().__init__()
		config = ConfigParser()
		config.read('data/config.ini')

		self.api_helper: hostkerapi.HostkerApiHelper =  hostkerapi.HostkerApiHelper(config)

		self.logger.info('Initializtion successful')
		self.domain_checker: List[str] = []

	def handle_reload(self) -> None:
		self.api_helper.reset_cache_time()

	def do_ip_update(self, now_ip: str) -> None:
		self.logger.debug('Getting dns record ip')
		data_group = self.api_helper.get_record_ip()
		self.logger.debug('Checking records')
		for _domain, headers_data in data_group.items():
			for header_data in headers_data:
				if now_ip != header_data['data']:
					self.domain_checker.append({'id': header_data['id'], 'data': now_ip, 'ttl': header_data['ttl']})
		if self.domain_checker:
			self.logger.debug('Find %d record need update, update it.', len(self.domain_checker))
			for data in self.domain_checker:
				self.api_helper.api_request('editRecord', data)
			self.logger.info('IP change detected, Changed dns ip to %s', now_ip)
			self.domain_checker = []

if __name__ == '__main__':
	if os.getppid() == 1:
		logging.basicConfig(level=logging.INFO, format='[%(levelname)s]\t%(funcName)s - %(lineno)d - %(message)s')
	else:
		logging.basicConfig(level=logging.DEBUG, format='%(asctime)s - %(levelname)s - %(name)s - %(funcName)s - %(lineno)d - %(message)s')
		logging.getLogger('passive-DDNS').info('Start program from normal mode, show debug message by default.')
	HostkerDDNS().run()
