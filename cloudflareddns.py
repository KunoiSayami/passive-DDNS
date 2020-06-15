# -*- coding: utf-8 -*-
# cloudflareddns.py
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

from absddns import AbstractDDNS
from cloudflareapi import CloudFlareApi


class CloudFlareDDNS(AbstractDDNS):
	def __init__(self):
		super().__init__()
		config = ConfigParser()
		config.read('data/config.ini')
		self.cloudflare_api = CloudFlareApi(config)
	
	def do_ip_update(self, now_ip: str) -> None:
		self.cloudflare_api.update_records(now_ip)
	
	def handle_reload(self) -> None:
		pass

	def close(self) -> None:
		self.cloudflare_api.close()


if __name__ == '__main__':
	if os.getppid() == 1:
		logging.basicConfig(level=logging.INFO, format='[%(levelname)s]\t%(funcName)s - %(lineno)d - %(message)s')
	else:
		logging.basicConfig(level=logging.DEBUG, format='%(asctime)s - %(levelname)s - %(name)s - %(funcName)s - %(lineno)d - %(message)s')
		logging.getLogger('passive-DDNS').info('Start program from normal mode, show debug message by default.')
	CloudFlareDDNS().run()
