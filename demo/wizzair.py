import execjs
import json
import re
import time
from curl_cffi import requests
import manv8
import pp_primp
import uuid
from demo.call_edge_sandbox import call_javascript

with open('copli_dt.js', 'r', encoding='utf8') as f:
    jsss=execjs.compile(f.read())


country = 'us'
session_id = int(time.time() * 1000)
proxyStr = f'http://brd-customer-hl_19cb0fe8-zone-ba-country-{country}-session-{session_id}:31whyoefh065@brd.superproxy.io:22225'
proxyStr = f'http://127.0.0.1:7890'

session = requests.Session(impersonate='chrome146', proxy=proxyStr)

headers = {
    'accept': 'text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7',
    'accept-language': 'en-US;q=0.9,en;q=0.8',
    'priority': 'u=0, i',
    'sec-ch-ua': '"Not;A=Brand";v="8", "Chromium";v="150", "Google Chrome";v="150"',
    'sec-ch-ua-mobile': '?0',
    'sec-ch-ua-platform': '"Windows"',
    'sec-fetch-dest': 'document',
    'sec-fetch-mode': 'navigate',
    'sec-fetch-site': 'none',
    'sec-fetch-user': '?1',
    'upgrade-insecure-requests': '1',
    'user-agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/150.0.0.0 Safari/537.36',
}

response = session.get('https://www.wizzair.com/149e9513-01fa-4fb0-aad4-566afd725d1b/2d206a39-8ed7-437e-a3be-862e0f06eea3/fp?x-kpsdk-v=j-1.2.543', headers=headers)
ak_bm_vw_1 = response.headers['x-kpsdk-ct']

cookies = {
    'ak_bm_vw_1.1': ak_bm_vw_1,
    'ak_bm_vw_1.1-ssn': ak_bm_vw_1,
}

# 加载基础配置， 配置"network": {"backend": "manual"}才能进行拦截请求，获取请求信息


js_path = re.findall('src="(.*?)"></script>', response.text)[0]
jsurl = 'https://www.wizzair.com' + js_path.replace('&amp;', '&')

js_resp = session.get(jsurl, headers=headers)


start_time = time.time()



iframe_Str = '<!DOCTYPE html><html><head></head><body><script>window.KPSDK={};KPSDK.now=typeof performance!==\'undefined\'&&performance.now?performance.now.bind(performance):Date.now.bind(Date);KPSDK.start=KPSDK.now();</script><script src="' + js_path + '"></script></body></html>'

print('开始执行...')
# 加载url， 以及fp的html和ips


middle_time = time.time()
# ctx.eval(f'''
# ifr = document.createElement('iframe')
# document.body.appendChild(ifr)
# ifr.srcdoc = `{iframe_Str}`
# ''')
# js会生成多个请求， 收集所有的请求， 然后取最终的/tl请求， 后续进行优化
# while True:




# print(network)
# print(len(network))
tl_req = call_javascript(
    js_resp.text,
    source_url=jsurl,
)
end_time = time.time()
print(f'执行ips.js消耗时间 {middle_time - start_time} s')
print(f'获取请求信息消耗时间 {end_time - middle_time} s')
print(f'总消耗时间 {end_time - start_time} s')

# 取出/tl的header以及body
headers = dict(tl_req.requests[0].headers)
data = tl_req.requests[0].body
print(f'[tl body length] {len(data)}')
url = tl_req.requests[0].url
# print(headers)

tl_headers = {
    'accept': '*/*',
    'accept-language': 'en-US;q=0.9,en;q=0.8',
    'content-type': 'application/octet-stream',
    'origin': 'https://www.wizzair.com',
    'priority': 'u=1, i',
    'referer': 'https://www.wizzair.com/149e9513-01fa-4fb0-aad4-566afd725d1b/2d206a39-8ed7-437e-a3be-862e0f06eea3/fp?x-kpsdk-v=j-1.2.543',
    'sec-ch-ua': '"Not;A=Brand";v="8", "Chromium";v="150", "Google Chrome";v="150"',
    'sec-ch-ua-mobile': '?0',
    'sec-ch-ua-platform': '"Windows"',
    'sec-fetch-dest': 'empty',
    'sec-fetch-mode': 'cors',
    'sec-fetch-site': 'same-origin',
    'user-agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/150.0.0.0 Safari/537.36',
    "x-kpsdk-ct": headers['x-kpsdk-ct'],
    "x-kpsdk-dt": jsss.call('get_dt'),
    # "x-kpsdk-dt": headers['x-kpsdk-dt'],
    "x-kpsdk-im": headers['x-kpsdk-im'],
    'x-kpsdk-v': 'j-1.2.543',
}
tl_resp = session.post(url, data=data, headers=tl_headers, cookies=cookies)
print(tl_resp.text)
ct = tl_resp.headers['x-kpsdk-ct']
st = int(tl_resp.headers['x-kpsdk-st'])
# exit(0)
for i in range(5):
        url = "http://1.12.223.150:8878/x-kpsdk-cd"
        # url = "http://127.0.0.1:7474/x-kpsdk-cd"

        payload = {
            "x-kpsdk-ct": ct,
            "x-kpsdk-st": st,
            "token": "59a826db7394c21f1b732dac180af5ae-cd",
            "host": "www.wizzair.com"
        }

        headers = {
            'Content-Type': "application/json"
        }

        cd = requests.post(url, data=json.dumps(payload), headers=headers).json()['x-kpsdk-cd']

        # print(cd)
        headers = {
            'accept': 'application/json, text/plain, */*',
            'accept-language': 'ko-KR,ko;q=0.9',
            'content-type': 'application/json',
            'dnt': '1',
            'origin': 'https://www.wizzair.com',
            'priority': 'u=1, i',
            'referer': 'https://www.wizzair.com/en-gb/booking/select-flight/GYD/FCO/2025-05-09/null/1/0/0/null',
            'sec-ch-ua': '"Not;A=Brand";v="8", "Chromium";v="150", "Google Chrome";v="150"',
            'sec-ch-ua-mobile': '?0',
            'sec-ch-ua-platform': '"Windows"',
            'sec-fetch-dest': 'empty',
            'sec-fetch-mode': 'cors',
            'sec-fetch-site': 'same-site',
            'user-agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/150.0.0.0 Safari/537.36',
            'x-kpsdk-cd': cd,
            'x-kpsdk-ct': ct,
            'x-kpsdk-v': 'j-1.2.543',
            # 'x-requestverificationtoken': '824126dd048c494a999aedc07c4ec223',
        }

        json_data = {
            'isFlightChange': False,
            'flightList': [
                {
                    'departureStation': 'TIA',
                    'arrivalStation': 'CRL',
                    'departureDate': '2026-07-20T00:00:00',
                },
                {
                    'departureStation': 'CRL',
                    'arrivalStation': 'TIA',
                    'departureDate': '2026-07-24T00:00:00',
                },
            ],
            'adultCount': 1,
            'childCount': 0,
            'infantCount': 0,
            'wdc': True,
        }
        response = session.post('https://be.wizzair.com/29.9.0/Api/search/search',
                                 headers=headers,
                                 json=json_data)
        print(response.status_code, response.text)
        if response.status_code == 200 and len(response.text) != 0:
            print('success')
        # ct = response.headers['x-kpsdk-ct']


