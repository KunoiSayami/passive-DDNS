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
import requests
import os, signal
from subprocess import Popen
from configparser import ConfigParser
import logging
import bs4
import libtplink
import hostkerapi
import libopenwrt

external_ip_uri = 'https://ipip.net/'

headers = {
	'User-Agent': 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_9_3) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/35.0.1916.47 Safari/537.36'
}

logger = logging.getLogger('passive-DDNS')

def get_current_IP():
	while True:
		try:
			return _get_current_IP()
		except:
			logger.exception('Exception while get current ip:')
			time.sleep(5)

def _get_current_IP():
	r = requests.get(external_ip_uri, headers=headers)
	r.raise_for_status()
	try:
		soup = bs4.BeautifulSoup(r.text, 'lxml')
	except bs4.FeatureNotFound:
		soup = bs4.BeautifulSoup(r.text, 'html.parser')
	ip = soup.find(class_='yourInfo').select('li a')[0].text
	return ip

def main():
	logger.debug('Loading configure file')
	config = ConfigParser()
	config.read('data/config.ini')
	interval = config.getint('account', 'interval', fallback=600)
	tplink_enabled = config.getboolean('tplink', 'enabled', fallback=False)
	if tplink_enabled:
		tplink_helper = libtplink.TpLinkHelper(config['tplink']['url'], config['tplink']['password'])
	openwrt_enabled = config.getboolean('openwrt', 'enabled', fallback=False)
	if openwrt_enabled:
		openwrt_helper = libopenwrt.OpenWRTHelper(
			config.get('openwrt', 'route'),
			config.get('openwrt', 'user'),
			config.get('openwrt', 'password'))
	logger.setLevel(config.getint('log', 'level', fallback=logging.INFO))
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
				now_ip = get_current_IP()
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
			time.sleep(interval)

def helpmsg():
	print('Please using `[--daemon, -d] <file name>\' to run this program.\n\tusing `-kill` to kill daemon process (if running)')

if __name__ == '__main__':
	logging.basicConfig(level=logging.DEBUG, format = '%(asctime)s - %(levelname)s - %(name)s - %(funcName)s - %(lineno)d - %(message)s')
	if len(sys.argv) == 1:
		main()
	elif len(sys.argv) == 2:
		if sys.argv[1] in ('-d', '--daemon'):
			Popen([sys.executable, sys.argv[0], '--daemon-start'])
		elif sys.argv[1] == '--daemon-start':
			with open('pid', 'w') as fout:
				fout.write(str(os.getpid()))
			main()
		elif sys.argv[1] == '-kill':
			with open('pid') as fin:
				os.kill(int(fin.read()), signal.SIGINT)
		else:
			helpmsg()
	else:
		helpmsg()