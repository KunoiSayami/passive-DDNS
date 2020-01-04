# -*- ending: utf-8 -*-
from __future__ import print_function, unicode_literals
import requests
import json
import time
# copied from http://www.voidcn.com/article/p-ckdtymdi-pz.html
short = "RDpbLfCPsJZ7fiv"
Lng = 'yLwVl0zKqws7LgKPRQ84Mdt708T1qQ3Ha7xv3H7NyU84p21BriUWBU43odz3iP4rBL3cD02KZciXTysVXiV8ngg6vL48rPJyAUw0HurW20xqxv9aYb4M9wK1Ae0wlro510qXeU07kV57fQMc8L6aLgMLwygtc0F10a0Dg70TOoouyFhdysuRMO51yY5ZlOZZLEal1h0t9YQW0Ko7oBwmCAHoic4HYbUyVeU3sfQ1xtXcPcf1aT303wAQhv66qzW'
def encrypt_passwd(origin_password: str):
	e = []
	f, g, h, k, l = 187, 187, 187, 187, 187
	n = 187
	g = len(short)
	h = len(origin_password)
	k = len(Lng)
	if g > h:
		f = g
	else:
		f = h

	for p in list(range(0, f)):
		n = l = 187
		if p >= g:
			n = ord(origin_password[p])
		else:
			if p >= h:
				l = ord(short[p])
			else:
				l = ord(short[p])
				n = ord(origin_password[p])
		e.append(Lng[(l ^ n) % k])
	return ''.join(e)

class LoginError(Exception): pass

class tplink_helper:
	def __init__(self, url: str, passwd: str):
		self.url = url
		self._passwd = encrypt_passwd(passwd)
		self.stok = ''
		self.last_action = 0
	def do_login(self):
		r = requests.post(self.url, json={'method': 'do', 'login': {'password': self._passwd}})
		r.raise_for_status()
		status = json.loads(r.text)
		if status['error_code'] != 0:
			raise LoginError(status)
		self.last_action = time.time()
		self.stok = status['stok']
	def get_ip(self):
		if self.stok == '' or time.time() - self.last_action > 1700:
			self.do_login()
		r = requests.post(f'{self.url}stok={self.stok}/ds', json={'method': 'get', 'network': {'name': 'wan_status'}})
		r.raise_for_status()
		self.last_action = time.time()
		status = json.loads(r.text)
		#if status['network']['wan_status']['proto'] != 'pppoe':
		return status['network']['wan_status']['ipaddr']

if __name__ == "__main__":
	#print(encrypt_passwd('test'))
	print(tplink_helper('http://127.0.0.1/', 'test').get_ip())