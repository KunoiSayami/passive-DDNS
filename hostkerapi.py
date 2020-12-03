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
import logging
import re
import time
from configparser import ConfigParser
from typing import Dict, List, TypeVar

import requests

T = TypeVar('T')


class HostkerApiHelper:
    apiTarget = {
        'getRecord': 'https://i.hostker.com/api/dnsGetRecords',
        'addRecord': 'https://i.hostker.com/api/dnsAddRecord',
        'editRecord': 'https://i.hostker.com/api/dnsEditRecord',
        'delRecord': 'https://i.hostker.com/api/dnsDeleteRecord',
    }

    def __init__(self, config: ConfigParser):
        self.logger = logging.getLogger(__name__)
        self.logger.setLevel(logging.DEBUG)
        self.header_domain = {x[0][1:-1]: re.findall(r'\'([^\']+)\'', x[1]) for x in map(lambda x: x.split(':'),
                                                                                         map(lambda x: x.strip(),
                                                                                             config.get('hostker',
                                                                                                        'header_domain')[
                                                                                             1:-1].split('],')))}
        self.token = {'email': config['hostker']['email'], 'token': config['hostker']['token']}
        self._cache_time = config.getint('hostker', 'ns_cache', fallback=30 * 60)
        self._last_get_ip_request = 0
        self._ip_cache = {}

    def api_request(self, operation: str, data: Dict[str, str]) -> Dict[str, T]:
        assert data is None or isinstance(data, dict), 'data param must dict'
        assert isinstance(operation, str)
        assert operation in self.apiTarget, 'operation `{}\' not support'.format(operation)
        t = self.token.copy()
        if data is not None:
            t.update(data)
        r = requests.post(self.apiTarget[operation], t)
        r.raise_for_status()
        rjson = r.json()
        # r.close()
        if rjson['success'] != 1:
            self.logger.error('Error in api_request()! (errorMessage:`%s\')', rjson['errorMessage'])
            self.logger.debug('operation=`%s\', request_uri = `%s\', data=`%s\', t=`%s\'', operation,
                              self.apiTarget[operation], repr(data), repr(t))
        return rjson

    def get_record_ip_ex(self, domain: str, headers: List[str]) -> List[Dict[str, str]]:
        return [x for x in self.api_request('getRecord', {'domain': domain})['records'] if x['header'] in headers]

    def _get_record_ip(self) -> Dict[str, T]:
        return {domain: self.get_record_ip_ex(domain, self.header_domain[domain]) for domain, _ in
                self.header_domain.items()}

    def get_record_ip(self) -> Dict[str, str]:
        if time.time() - self._last_get_ip_request <= self._cache_time:
            return self._ip_cache
        self._ip_cache = self._get_record_ip()
        self._last_get_ip_request = time.time()
        # return self.get_record_ip()
        return self._ip_cache

    def reset_cache_time(self) -> None:
        self._cache_time = 0
