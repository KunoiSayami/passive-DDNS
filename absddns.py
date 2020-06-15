# -*- coding: utf-8 -*-
# absddns.py
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
import signal
import time
from abc import ABCMeta, abstractmethod
from configparser import ConfigParser

import libopenwrt
import libother
import libtplink


class AbstractDDNS(metaclass=ABCMeta):
	def __init__(self):
		config = ConfigParser()
		config.read('data/config.ini')

		self.logger: logging.Logger = logging.getLogger('passive-DDNS')
		self.logger.setLevel(config.getint('log', 'level', fallback=logging.INFO))

		self.logger.debug('Loading configure file')

		self.interval: int = config.getint('account', 'interval', fallback=600)
		libother.SimpleIPQuery.set_url(config.get('account', 'extern_ip_uri', fallback=libother.SimpleIPQuery.url))

		self.tplink_enabled: bool = config.getboolean('tplink', 'enabled', fallback=False)
		if self.tplink_enabled:
			self.tplink_helper: libtplink.TpLinkHelper = libtplink.TpLinkHelper(config['tplink']['url'], config['tplink']['password'])
		self.openwrt_enabled: bool = config.getboolean('openwrt', 'enabled', fallback=False)
		if self.openwrt_enabled:
			self.openwrt_helper: libopenwrt.OpenWRTHelper = libopenwrt.OpenWRTHelper(
				config.get('openwrt', 'route'),
				config.get('openwrt', 'user'),
				config.get('openwrt', 'password'))

		self._reload_request: bool = False
		signal.signal(10, self._handle_reload)

	@abstractmethod
	def handle_reload(self) -> None:
		return NotImplemented

	def _handle_reload(self, *_args) -> None:
		self.logger.info('Got reload request, refreshing IP status')
		self.handle_reload()
		self._reload_request = True
		os.kill(os.getpid(), signal.SIGINT)

	def run(self) -> None:
		while True:
			try:
				self.logger.debug('Getting current ip')
				if self.tplink_enabled:
					now_ip = self.tplink_helper.get_ip()
				elif self.openwrt_enabled:
					now_ip = self.openwrt_helper.get_ip()
				else:
					try:
						now_ip = libother.SimpleIPQuery.get_ip()
					except:
						now_ip = libother.IPIPdotNet.get_ip()
				self.do_ip_update(now_ip)
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

	@abstractmethod
	def do_ip_update(self, _now_ip: str) -> None:
		return NotImplemented
