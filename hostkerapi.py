# -*- coding: utf-8 -*-
# hostkerapi.py
# Copyright (C) 2018 Too-Naive and contributors
#
# This module is part of passive-DDNS and is released under
# the AGPL v3 License: https://www.gnu.org/licenses/agpl-3.0.txt
from libpy.Config import Config
from libpy import Log
import requests
import json
szapiTarget = {'getRecord':'https://i.hostker.com/api/dnsGetRecords',
	'addRecord':'https://i.hostker.com/api/dnsAddRecord',
	'editRecord':'https://i.hostker.com/api/dnsEditRecord',
	'delRecord':'https://i.hostker.com/api/dnsDeleteRecord',
}

def apiRequest(operaction='getRecord', data=None):
	assert data is None or isinstance(data, dict), 'data param must dict'
	assert isinstance(operaction,str)
	assert operaction in szapiTarget, 'operation `{}\' not support'.format(operaction)
	t = {'email':Config.account.email, 'token':Config.account.token}
	if data is not None:
		t.update(data)
	r = requests.post(szapiTarget[operaction], t)
	r.raise_for_status()
	rjson = json.loads(str(r.content))
	r.close()
	if rjson['success'] != 1:
		Log.error('Error in apiRequest()! (errorMessage:`{}\')',rjson['errorMessage'])
		Log.debug(1, 'operaction=`{}\', request_uri = `{}\', data=`{}\', t=`{}\'', operaction, szapiTarget[operaction], repr(data), repr(t))
	return rjson
	
def get_record_ip():
	r = apiRequest(data={'domain': Config.account.domain})
	#print(r['records'])
	for x in r['records']:
		if x['header'] == Config.account.header:
			return x, x['data']
