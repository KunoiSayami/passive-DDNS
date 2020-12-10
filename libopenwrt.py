# -*- coding: utf-8 -*-
# libopenwrt.py
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
import struct
from typing import Optional

import requests


class SessionFile:
    VERSION = 2

    class VersionError(Exception):
        pass

    def __init__(self, f: bytes) -> None:
        self.ver: int = 0
        self.version_defined: bool = True
        self.require_v2: bool = False
        self.session: str = ''
        try:
            ver, version_defined, require_v2, session = struct.unpack('I??64s', f)
            if ver < self.VERSION:
                raise SessionFile.VersionError()
            self.ver = ver
            self.version_defined = version_defined
            self.require_v2 = require_v2
            self.session = session.decode().strip().replace('\x00', '')
        except (SessionFile.VersionError, struct.error):
            pass

    @property
    def v2(self) -> Optional[bool]:
        if self.version_defined:
            return
        return self.require_v2

    @v2.setter
    def v2(self, value: Optional[bool]) -> Optional[bool]:
        if value is None:
            self.version_defined = True
        else:
            self.version_defined = False
            self.require_v2 = value
        return value

    def __repr__(self) -> str:
        return repr((self.ver, self.version_defined, self.require_v2, self.session,))

    def pack(self) -> bytes:
        return struct.pack('<I??64s', self.version_defined, self.VERSION, self.require_v2, self.session.encode())


class OpenWRTHelper:
    class OtherError(Exception):
        pass

    class MaxRetryError(Exception):
        pass

    class UnknownError(Exception):
        pass

    def __init__(self, route_ip: str, user: str, password: str):
        self.route_web: str = f'http://{route_ip}'
        self.user: str = user
        self.password: str = password
        self.requests_session: requests.Session = requests.Session()
        self.logger: logging.Logger = logging.getLogger('passive-DDNS').getChild('OpenWRTHelper')
        self.logger.setLevel(logging.getLogger('passive-DDNS').level)
        self.user_session: SessionFile = self._read_session()

    def _write_session(self) -> None:
        try:
            with open('data/.session', 'wb') as fout:
                fout.write(self.user_session.pack())
            self.logger.debug(repr(self.user_session))
        except PermissionError:
            self.logger.warning('Got permission error while write session file, ignored.')

    def _read_session(self) -> SessionFile:
        try:
            with open('data/.session', 'rb') as fin:
                sf = SessionFile(fin.read())
                self.logger.debug(repr(sf))
                self.requests_session.cookies.update({'sysauth': sf.session})
            return sf
        except FileNotFoundError:
            return SessionFile(b'')

    def check_login(self) -> bool:
        self.logger.debug('Check login')
        return self.requests_session.get(self.route_web + '/cgi-bin/luci/').status_code == 200

    def do_login(self, force: bool = False) -> bool:
        self.logger.debug('Request login')
        if not force and self.check_login():
            return True
        self.requests_session.cookies.clear()
        r = self.requests_session.post(f'{self.route_web}/cgi-bin/luci',
                                       data={'luci_username': self.user, 'luci_password': self.password},
                                       allow_redirects=False)
        r.raise_for_status()
        self.user_session.session = self.requests_session.cookies.get('sysauth')
        self._write_session()
        return self.check_login()

    def get_ipv4(self, relogin: bool = False) -> str:
        return self.get_ip(relogin)

    def get_ipv6(self, relogin: bool = False) -> str:
        return self.get_ip(relogin, v6=True)

    def get_ip(self, relogin: bool = False, *, v6: bool = False) -> str:
        if self.user_session.v2 is None:
            try:
                ip = self.get_ip_v1(relogin, v6=v6)
                self.user_session.v2 = False
            except OpenWRTHelper.UnknownError:
                ip = self.get_ip_v2(relogin, v6=v6)
                self.user_session.v2 = True
            self._write_session()
        elif self.user_session.v2:
            ip = self.get_ip_v2(relogin, v6=v6)
        else:
            ip = self.get_ip_v1(relogin, v6=v6)
        return ip

    def get_ip_v1(self, relogin: bool = False, *, v6: bool = False) -> str:
        self.do_login(relogin)
        r = self.requests_session.post(f'{self.route_web}/ubus/?{int(time.time())}',
                                       json=[{'jsonrpc': '2.0', 'id': 1, 'method': 'call',
                                              'params': [self.user_session.session, 'network.interface', 'dump', {}]}])
        raw_data = r.json()[0]
        self.logger.debug('json object => %s', repr(raw_data))
        if raw_data.get('error') is None:
            for interface in raw_data.get('result')[1].get('interface'):
                if interface.get('interface') == ('wan6' if v6 else 'wan'):
                    return interface.get('ipv6-address' if v6 else 'ipv4-address')[0].get('address', 'ERROR')
        else:
            if raw_data['error']['message'] == 'Access denied':
                if relogin:
                    raise OpenWRTHelper.UnknownError()
                else:
                    return self.get_ip_v1(True)
            else:
                raise OpenWRTHelper.OtherError()

    def get_ip_v2(self, relogin: bool = False, *, v6: bool = False) -> str:
        self.do_login(relogin)
        r = self.requests_session.get(f'{self.route_web}/cgi-bin/luci/?status=1&_={time.time()}')
        return r.json()['wan6' if v6 else 'wan']['ipaddr']


if __name__ == "__main__":
    print(OpenWRTHelper('127.0.0.1', 'root', '').get_ip())
