# -*- coding: utf-8 -*-
# main.py
# Copyright (C) 2018-2019 KunoiSayami and contributors
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
if sys.version_info[0] == 2:
	import urllib2
else:
	import requests
import hostkerapi
import os, signal
from subprocess import Popen
from configparser import ConfigParser
import logging

external_ip_uri = 'https://www.appveyor.com/tools/my-ip.aspx'

logger = logging.getLogger('passive-DDNS')
logger.setLevel(logging.DEBUG)

def init():
	if sys.version[0] == 2:
		reload(sys)
		sys.setdefaultencoding('utf8')

def get_current_IP():
	while True:
		try:
			return _get_current_IP()
		except:
			logger.exception('Exception while get current ip:')
			time.sleep(5)


def _get_current_IP():
	if sys.version[0] == 2:
		r = urllib2.urlopen(external_ip_uri)
		ip = r.read()
		r.close()
	else:
		r = requests.get(external_ip_uri)
		r.raise_for_status()
		ip = r.text
	# Maybe need some process
	#r.close()
	return ip

def main():
	config = ConfigParser()
	config.read('data/config.ini')
	interval = 600 if not config.has_option('account', 'interval') else int(config['account']['interval'])
	if config.has_section('ipconfig') and config.has_option('ipconfig', 'extern_ip_uri'):
		global external_ip_uri
		external_ip_uri = config['ipconfig']['extern_ip_uri']
	logger.info('Initializtion successful')
	#print(raw,ipaddr)
	domain_checker = []
	while True:
		try:
			now_ip = get_current_IP()
			data_group = hostkerapi.get_record_ip()
			for domain, headers_data in data_group.items():
				for header_data in headers_data:
					if now_ip != header_data['data']:
						domain_checker.append({'id': header_data['id'], 'data': now_ip, 'ttl': header_data['ttl']})
			if len(domain_checker):
				for data in domain_checker:
					hostkerapi.apiRequest('editRecord', data)
				logger.info('IP change detected, Changed dns ip from to %s', now_ip)
				domain_checker = []
		except AssertionError:
			logger.exception('Catched AssertionError, Program will now exit.')
		except:
			logger.exception('Got unexcepted error')
			time.sleep(10) # Failsafe
		else:
			time.sleep(interval)

def helpmsg():
	print('Please using `[--daemon, -d] <file name>\' to run this program.\n\tusing `-kill` to kill daemon process (if running)')

if __name__ == '__main__':
	if len(sys.argv) == 1:
		init()
		main()
	elif len(sys.argv) == 2:
		if sys.argv[1] in ('-d', '--daemon'):
			Popen(['python', sys.argv[0], '--daemon-start'])
		elif sys.argv[1] == '--daemon-start':
			with open('pid', 'w') as fout:
				fout.write(str(os.getpid()))
			init()
			main()
		elif sys.argv[1] == '-kill':
			with open('pid') as fin:
				os.kill(int(fin.read()), signal.SIGINT)
		else:
			helpmsg()
	else:
		helpmsg()