# -*- coding: utf-8 -*-
# hostkerapi.py
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
from configparser import ConfigParser
import logging
import re
import requests

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

def apiRequest(operaction: str = 'getRecord', data: dict = None) -> dict:
	assert data is None or isinstance(data, dict), 'data param must dict'
	assert isinstance(operaction, str)
	assert operaction in szapiTarget, 'operation `{}\' not support'.format(operaction)
	t = {'email': config['account']['email'], 'token': config['account']['token']}
	if data is not None:
		t.update(data)
	r = requests.post(szapiTarget[operaction], t)
	r.raise_for_status()
	rjson = r.json()
	#r.close()
	if rjson['success'] != 1:
		logger.error('Error in apiRequest()! (errorMessage:`%s\')', rjson['errorMessage'])
		logger.debug('operaction=`%s\', request_uri = `%s\', data=`%s\', t=`%s\'', operaction, szapiTarget[operaction], repr(data), repr(t))
	return rjson

def get_record_ip_ex(domain: str, headers: list) -> list:
	return [x for x in apiRequest(data={'domain': domain})['records'] if x['header'] in headers]

def get_record_ip() -> dict:
	header_domain = {x[0][1:-1]: re.findall(r'\'([^\']+)\'', x[1]) for x in map(lambda x: x.split(':'),
			map(lambda x: x.strip(), config.get('account', 'header_domain')[1:-1].split('],')))}
	return {domain: get_record_ip_ex(domain, header_domain[domain]) for domain, _ in header_domain.items()}
