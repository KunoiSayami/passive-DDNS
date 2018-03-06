# -*- coding: utf-8 -*-
# main.py
# Copyright (C) 2018 Too-Naive and contributors
#
# This module is part of passive-DDNS and is released under
# the AGPL v3 License: https://www.gnu.org/licenses/agpl-3.0.txt
import sys
import time
import urllib2
import hostkerapi
from libpy import Log
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

if __name__ == '__main__':
	init()
	main()
