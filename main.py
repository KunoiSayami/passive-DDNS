# -*- coding: utf-8 -*-
# main.py
# Copyright (C) 2018 Too-Naive and contributors
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
import urllib2
import hostkerapi
import os, signal
from libpy import Log
from subprocess import Popen
from libpy.Config import Config

external_ip_uri = 'https://www.appveyor.com/tools/my-ip.aspx'

def init():
	reload(sys)
	sys.setdefaultencoding('utf8')

def get_current_IP():
	r = urllib2.urlopen(external_ip_uri)
	ip = r.read()
	# Maybe need some process
	r.close()
	return ip

def main():
	raw, ipaddr = hostkerapi.get_record_ip()
	Log.info('Initializtion successful')
	#print(raw,ipaddr)
	while True:
		now_ip = get_current_IP()
		if now_ip != ipaddr:
			hostkerapi.apiRequest('editRecord',{'id': raw['id'], 'data': now_ip, 'ttl': raw['ttl']})
			Log.info('IP change detected, Changed dns ip from {} to {}', ipaddr, now_ip)
			raw, ipaddr = hostkerapi.get_record_ip()
		time.sleep(600)

def helpmsg():
	print('Please using `[--daemon, -d] <file name>\' to run this program.\n\tusing `-kill` to kill daemon process (if running)')

if __name__ == '__main__':
	#if len(sys.argv) == 3:
	#	if sys.argv[1] == '-d' or sys.argv[1] == '--daemon':
	#		Popen(['python', sys.argv[0], '--daemon-start', sys.argv[2]])
	#	elif sys.argv[1] == '--daemon-start':
	#		with open(sys.argv[2], 'w') as fout:
	#			fout.write(str(os.getpid()))
	#	else:
	#		helpmsg()
	if len(sys.argv) == 2:
		if sys.argv[1] == '-d' or sys.argv[1] == '--daemon':
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