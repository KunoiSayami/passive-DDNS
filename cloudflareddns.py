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
from configparser import ConfigParser

from absddns import AbstractDDNS
from cloudflareapi import CloudFlareApi


class CloudFlareDDNS(AbstractDDNS):
    def __init__(self, from_user: bool = False):
        super().__init__(from_user)
        config = ConfigParser()
        config.read('data/config.ini')
        self.cloudflare_api = CloudFlareApi(config)

    def do_ip_update(self, now_ip: str) -> bool:
        if self.cloudflare_api.update_records(now_ip):
            self.logger.info('IP change detected, Changed dns ip to %s', now_ip)
            return True
        return False

    def handle_reload(self) -> None:
        pass

    def close(self) -> None:
        self.cloudflare_api.close()
