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
import sys
import time
from configparser import ConfigParser
import logging
import libtplink
import hostkerapi
import libopenwrt
import libother

logger = logging.getLogger('passive-DDNS')

def main():
	logger.debug('Loading configure file')

	config = ConfigParser()
	config.read('data/config.ini')

	interval = config.getint('account', 'interval', fallback=600)
	logger.setLevel(config.getint('log', 'level', fallback=logging.INFO))
	libother.simple_ip.set_url(config.get('account', 'extern_ip_uri', fallback=libother.simple_ip.url))

	tplink_enabled = config.getboolean('tplink', 'enabled', fallback=False)
	if tplink_enabled:
		tplink_helper = libtplink.TpLinkHelper(config['tplink']['url'], config['tplink']['password'])
	openwrt_enabled = config.getboolean('openwrt', 'enabled', fallback=False)
	if openwrt_enabled:
		openwrt_helper = libopenwrt.OpenWRTHelper(
			config.get('openwrt', 'route'),
			config.get('openwrt', 'user'),
			config.get('openwrt', 'password'))

	logger.info('Initializtion successful')
	domain_checker = []

	while True:
		try:
			logger.debug('Getting current ip')
			if tplink_enabled:
				now_ip = tplink_helper.get_ip()
			elif openwrt_enabled:
				now_ip = openwrt_helper.get_ip()
			else:
				try:
					now_ip = libother.ipip.get_ip()
				except:
					now_ip = libother.simple_ip.get_ip()
			logger.debug('Getting dns record ip')
			data_group = hostkerapi.get_record_ip()
			logger.debug('Checking records')
			for _domain, headers_data in data_group.items():
				for header_data in headers_data:
					if now_ip != header_data['data']:
						domain_checker.append({'id': header_data['id'], 'data': now_ip, 'ttl': header_data['ttl']})
			if domain_checker:
				logger.debug('Find %d record need update, update it.', len(domain_checker))
				for data in domain_checker:
					hostkerapi.apiRequest('editRecord', data)
				logger.info('IP change detected, Changed dns ip from to %s', now_ip)
				domain_checker = []
		except AssertionError as e:
			logger.exception('Catched AssertionError, Program will now exit.')
			raise e
		except:
			logger.exception('Got unexcepted error')
			time.sleep(10) # Failsafe
		else:
			try:
				time.sleep(interval)
			except KeyboardInterrupt:
				raise SystemExit

if __name__ == '__main__':
	logging.basicConfig(level=logging.DEBUG, format='%(asctime)s - %(levelname)s - %(name)s - %(funcName)s - %(lineno)d - %(message)s')
	if len(sys.argv) == 1:
		main()
	elif len(sys.argv) == 2:
		if sys.argv[1] == '--systemd':
			logging.basicConfig(level=logging.INFO, format='%(levelname)s - %(name)s - %(funcName)s - %(lineno)d - %(message)s')
			main()
