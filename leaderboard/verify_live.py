"""Run ONLY after the user confirms first deployment. Never prints the admin URL."""
import argparse, json, pathlib, secrets, sys, urllib.request, urllib.parse
parser=argparse.ArgumentParser()
parser.add_argument('--deployed',action='store_true',help='Confirm the initial deployment completed; only then contact the domain')
args=parser.parse_args()
if not args.deployed:sys.exit('Not contacted. Run with --deployed only after first deployment succeeds.')
site='https://skypop.quietbytehq.uk'
admin=pathlib.Path.home()/'.config/skypop-scores-admin.txt'
key=urllib.parse.parse_qs(urllib.parse.urlparse(admin.read_text().strip()).query)['key'][0]
def call(route,body=None):
    req=urllib.request.Request(site+route,data=None if body is None else json.dumps(body).encode(),headers={'Content-Type':'application/json','User-Agent':'SkyPop-Release-Check/1.1.0'})
    with urllib.request.urlopen(req,timeout=15) as response:return json.load(response)
name='QA'+secrets.token_hex(4).upper()
try:
    initial=call('/api/top')['scores']
    call('/api/submit',{'name':name,'score':1000,'mode':'adventure','version':'1.1.0'})
    entries=call('/admin/entries?key='+urllib.parse.quote(key))['entries']
    ours=[row for row in entries if row['name']==name]
    if len(ours)!=1:raise RuntimeError()
    listed=any(row['name']==name for row in call('/api/top')['scores'])
    for row in ours:call('/admin/delete?key='+urllib.parse.quote(key)+'&id='+str(row['id']),{})
    remaining=call('/api/top')['scores']
    assert not any(row['name']==name for row in remaining)
    if not initial:assert not remaining
    print('Live test submitted and removed through admin. '+('It appeared in the public top 100. ' if listed else 'It was stored but did not reach the existing top 100. ')+('Board is empty.' if not remaining else 'Existing player scores retained.'))
except Exception:
    print('Live verification did not complete. Inspect the private admin page locally for the QA-prefixed test score; no key or URL is printed.',file=sys.stderr)
    sys.exit(1)
