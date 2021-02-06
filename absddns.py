# -*- coding: utf-8 -*-
# absddns.py
# Copyright (C) 2018-2021 KunoiSayami and contributors
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
import signal
import sys
import time
import traceback
import platform
from abc import ABCMeta, abstractmethod
from configparser import ConfigParser
from subprocess import TimeoutExpired

import requests

import libopenwrt
import libother
import libtplink


class AbstractDDNS(metaclass=ABCMeta):
    def __init__(self, from_user: bool = False):
        config = ConfigParser()
        config.read('data/config.ini')

        self.logger: logging.Logger = logging.getLogger('passive-DDNS')
        self.file_log: logging.FileHandler = logging.FileHandler('data/log.log')
        self.file_log.setLevel(logging.DEBUG)
        self.file_log.setFormatter(
            logging.Formatter('%(asctime)s - %(levelname)s - %(name)s - %(funcName)s - %(lineno)d - %(message)s'))
        self.logger.setLevel(config.getint('log', 'level', fallback=logging.DEBUG if from_user else logging.INFO))
        self.logger.addHandler(self.file_log)

        self.logger.debug('Loading configure file')

        self.interval: int = config.getint('account', 'interval', fallback=600)
        libother.SimpleIPQuery.extend_ip_from_list(config.get('account', 'extern_ip_uris', fallback=[]))

        self.tplink_enabled: bool = config.getboolean('tplink', 'enabled', fallback=False)
        if self.tplink_enabled:
            self.tplink_helper: libtplink.TpLinkHelper = libtplink.TpLinkHelper(config.get('tplink', 'url'),
                                                                                config.get('tplink', 'password'))
        self.openwrt_enabled: bool = config.getboolean('openwrt', 'enabled', fallback=False)
        if self.openwrt_enabled:
            self.openwrt_helper: libopenwrt.OpenWRTHelper = libopenwrt.OpenWRTHelper(
                config.get('openwrt', 'route'),
                config.get('openwrt', 'user'),
                config.get('openwrt', 'password'))

        self._reload_request: bool = False
        if platform.system() != 'Windows':
            signal.signal(10, self._handle_reload)
        else:
            self.logger.info("Ignore reload command in Windows platform")

    @abstractmethod
    def handle_reload(self) -> None:
        raise NotImplemented

    def _handle_reload(self, *_args) -> None:
        self.logger.info('Got reload request, refreshing IP status')
        self.handle_reload()
        self._reload_request = True
        os.kill(os.getpid(), signal.SIGINT)

    def run(self) -> None:
        _exception = False
        while True:
            try:
                self.logger.debug('Getting current ip')
                if self.tplink_enabled:
                    now_ip = self.tplink_helper.get_ip()
                elif self.openwrt_enabled:
                    now_ip = self.openwrt_helper.get_ip()
                else:
                    try:
                        now_ip = libother.SimpleIPQuery.get_ip()
                    except:
                        now_ip = libother.IPIPdotNet.get_ip()
                if self.do_ip_update(now_ip):
                    self.logger.debug('IP changed')
                else:
                    self.logger.debug('IP unchanged')
                if _exception:
                    self.logger.info('Program working normally')
            except AssertionError as e:
                self.logger.exception('Catch AssertionError, Program will now exit.')
                raise e
            except requests.Timeout:
                self.logger.error('Wait more time to reconnect')
                time.sleep(120)
            except requests.ConnectionError as e:
                self.logger.error('Got %s, ignored.', traceback.format_exception_only(type(e), e)[0].strip())
                time.sleep(10)
            except:
                self.logger.critical('Got unexpected error', exc_info=True)
                time.sleep(self.interval / 3)  # Failsafe
            else:
                try:
                    time.sleep(self.interval)
                except KeyboardInterrupt:
                    if not self._reload_request:
                        self.close()
                        raise SystemExit
                    else:
                        self._reload_request = False
                        self.logger.debug('Reset reload request')
                    continue

    @abstractmethod
    def close(self) -> None:
        raise NotImplemented

    @abstractmethod
    def do_ip_update(self, now_ip: str) -> bool:
        return NotImplemented


if __name__ == "__main__":
    if len(sys.argv) == 3:
        if sys.argv[1] == 'stop':
            import subprocess

            os.kill(int(sys.argv[2]), 2)
            p = subprocess.Popen(['/usr/bin/tail', f'--pid={int(sys.argv[2])}', '-f', '/dev/null'])
            try:
                p.wait(2)
            except TimeoutExpired:
                os.kill(int(sys.argv[2]), 2)
            p.wait()
