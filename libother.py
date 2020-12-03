# -*- coding: utf-8 -*-
# libother.py
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
import logging
import time
from abc import ABCMeta, abstractmethod

import bs4
import requests

logger = logging.getLogger(__name__)


class IPQuery(metaclass=ABCMeta):

    @classmethod
    @abstractmethod
    def get_ip(cls) -> str:
        return NotImplemented


class IPIPdotNet(IPQuery):
    @classmethod
    def get_ip(cls) -> str:
        while True:
            try:
                return cls.get_current_IP()
            except:
                logger.exception('Exception while get current ip:')
                time.sleep(5)

    @staticmethod
    def get_current_IP() -> str:
        r = requests.get('https://ipip.net/', headers={
            'User-Agent': 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_9_3) AppleWebKit/537.36 (KHTML, like Gecko) '
                          'Chrome/35.0.1916.47 Safari/537.36 '
        })
        r.raise_for_status()
        try:
            soup = bs4.BeautifulSoup(r.text, 'lxml')
        except bs4.FeatureNotFound:
            soup = bs4.BeautifulSoup(r.text, 'html.parser')
        ip = soup.find(class_='yourInfo').select('li a')[0].text
        return ip


class SimpleIPQuery(IPQuery):
    url = 'https://api-ipv4.ip.sb/ip'

    @classmethod
    def get_ip(cls) -> str:
        r = requests.get(cls.url)
        r.raise_for_status()
        return r.text

    @classmethod
    def set_url(cls, url: str) -> str:
        cls.url = url
        return url
