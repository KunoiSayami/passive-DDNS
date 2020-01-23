# -*- coding: utf-8 -*-
# pddns.py
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
import time
import os
from configparser import ConfigParser
import signal
import logging
import libtplink
import hostkerapi
import libopenwrt
import libother

class DDNS:
	def __init__(self):
		config = ConfigParser()
		config.read('data/config.ini')

		self.logger = logging.getLogger('passive-DDNS')
		self.logger.setLevel(config.getint('log', 'level', fallback=logging.INFO))

		self.logger.debug('Loading configure file')

		self.interval = config.getint('account', 'interval', fallback=600)
		libother.simple_ip.set_url(config.get('account', 'extern_ip_uri', fallback=libother.simple_ip.url))

		self.tplink_enabled = config.getboolean('tplink', 'enabled', fallback=False)
		if self.tplink_enabled:
			self.tplink_helper = libtplink.TpLinkHelper(config['tplink']['url'], config['tplink']['password'])
		self.openwrt_enabled = config.getboolean('openwrt', 'enabled', fallback=False)
		if self.openwrt_enabled:
			self.openwrt_helper = libopenwrt.OpenWRTHelper(
				config.get('openwrt', 'route'),
				config.get('openwrt', 'user'),
				config.get('openwrt', 'password'))

		self.api_helper =  hostkerapi.HostkerApiHelper(config)

		self.logger.info('Initializtion successful')
		self.domain_checker = []
		self._reload_request = False
		signal.signal(10, self.handle_reload)

	def handle_reload(self, *_args):
		self.logger.info('Got reload request, refreshing IP status')
		self.api_helper.reset_cache_time()
		self._reload_request = True
		os.kill(os.getpid(), signal.SIGINT)

	def run(self):
		while True:
			try:
				self.logger.debug('Getting current ip')
				if self.tplink_enabled:
					now_ip = self.tplink_helper.get_ip()
				elif self.openwrt_enabled:
					now_ip = self.openwrt_helper.get_ip()
				else:
					try:
						now_ip = libother.ipip.get_ip()
					except:
						now_ip = libother.simple_ip.get_ip()
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
						self.api_helper.apiRequest('editRecord', data)
					self.logger.info('IP change detected, Changed dns ip to %s', now_ip)
					self.domain_checker = []
			except AssertionError as e:
				self.logger.exception('Catched AssertionError, Program will now exit.')
				raise e
			except:
				self.logger.exception('Got unexcepted error')
				time.sleep(10) # Failsafe
			else:
				try:
					time.sleep(self.interval)
				except KeyboardInterrupt:
					if not self._reload_request:
						raise SystemExit
					else:
						self._reload_request = False
						self.logger.debug('Reset reload request')
					continue

if __name__ == '__main__':
	if os.getppid() == 1:
		logging.basicConfig(level=logging.INFO, format='[%(levelname)s]\t%(funcName)s - %(lineno)d - %(message)s')
	else:
		logging.basicConfig(level=logging.DEBUG, format='%(asctime)s - %(levelname)s - %(name)s - %(funcName)s - %(lineno)d - %(message)s')
		logging.getLogger('passive-DDNS').info('Start program from notrmal mode, show debug message by default.')
	DDNS().run()
