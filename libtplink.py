# -*- ending: utf-8 -*-
from __future__ import print_function, unicode_literals
import requests
import json
# copied from http://www.voidcn.com/article/p-ckdtymdi-pz.html
short = "RDpbLfCPsJZ7fiv"
Lng = 'yLwVl0zKqws7LgKPRQ84Mdt708T1qQ3Ha7xv3H7NyU84p21BriUWBU43odz3iP4rBL3cD02KZciXTysVXiV8ngg6vL48rPJyAUw0HurW20xqxv9aYb4M9wK1Ae0wlro510qXeU07kV57fQMc8L6aLgMLwygtc0F10a0Dg70TOoouyFhdysuRMO51yY5ZlOZZLEal1h0t9YQW0Ko7oBwmCAHoic4HYbUyVeU3sfQ1xtXcPcf1aT303wAQhv66qzW'
def encrypt_passwd(origin_password):
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

def get_ip_from_tplink(url, passwd):
	r = requests.post(url, json={'method': 'do', 'login': {'password': encrypt_passwd(passwd)}})
	r.raise_for_status()
	status = json.loads(r.text)
	stok = status['stok']
	r = requests.post(f'{url}stok={stok}/ds', json={'method': 'get', 'network': {'name': 'wan_status'}})
	r.raise_for_status()
	status = json.loads(r.text)
	#if status['network']['wan_status']['proto'] != 'pppoe':
	return status['network']['wan_status']['ipaddr']

if __name__ == "__main__":
	print(encrypt_passwd('test'))
	print(get_ip_from_tplink('http://127.0.0.1/', 'test'))