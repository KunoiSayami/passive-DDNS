# -*- coding: utf-8 -*-
# hostkerapi.py
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
from configparser import ConfigParser
import requests
import json
import sys
import logging

logger = logging.getLogger(__name__)
logger.setLevel(logging.DEBUG)

szapiTarget = {
	'getRecord':'https://i.hostker.com/api/dnsGetRecords',
	'addRecord':'https://i.hostker.com/api/dnsAddRecord',
	'editRecord':'https://i.hostker.com/api/dnsEditRecord',
	'delRecord':'https://i.hostker.com/api/dnsDeleteRecord',
}

config = ConfigParser()
config.read('data/config.ini')

def apiRequest(operaction='getRecord', data=None):
	assert data is None or isinstance(data, dict), 'data param must dict'
	assert isinstance(operaction, str)
	assert operaction in szapiTarget, 'operation `{}\' not support'.format(operaction)
	t = {'email':config['account']['email'], 'token':config['account']['token']}
	if data is not None:
		t.update(data)
	r = requests.post(szapiTarget[operaction], t)
	r.raise_for_status()
	rjson = r.json()
	#r.close()
	if rjson['success'] != 1:
		logger.error('Error in apiRequest()! (errorMessage:`%s\')',rjson['errorMessage'])
		if sys.version_info[0] == 2:
			logger.debug('operaction=`%s\', request_uri = `%s\', data=`%s\', t=`%s\'', operaction, szapiTarget[operaction], repr(data), repr(t))
		else:
			logger.debug('operaction=`%s\', request_uri = `%s\', data=`%s\', t=`%s\'', operaction, szapiTarget[operaction], repr(data), repr(t))
	return rjson

def get_record_ip_ex(domain, headers):
	r = apiRequest(data={'domain': domain})
	#print(r['records'])
	record_ip = []
	for x in r['records']:
		if x['header'] in headers:
			record_ip.append(x)
	return record_ip

def get_record_ip():
	if config.has_option('account', 'domain'):
		return {config['account']['domain']: get_record_ip_ex(config['account']['domain'], [config['account']['header']])}
	else:
		header_domain = eval(config['account']['header_domain'])
		return {domain: get_record_ip_ex(domain, header_domain[domain]) for domain, _ in header_domain.items()}
