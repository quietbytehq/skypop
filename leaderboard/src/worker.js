// Inspired by quietbytehq/nudge/leaderboard. Scores are self-reported, not verified.
const MODES = Object.freeze({adventure: 'Adventure · Ten worlds'});
const BLOCKED = ['fuck','shit','cunt','nigg','fag','bitch','dick','cock','pussy','rape','nazi','hitler','porn','sex','whore','slut','arse','wank'];
const headers = {'cache-control':'no-store','x-content-type-options':'nosniff','referrer-policy':'no-referrer'};
const json = (body,status=200) => new Response(JSON.stringify(body),{status,headers:{...headers,'content-type':'application/json; charset=utf-8'}});
const missing = () => new Response('Not found',{status:404,headers});
function cleanName(raw) {
 if (typeof raw !== 'string') return null;
 const name=raw.toUpperCase().replace(/[^A-Z0-9 ._-]/g,'').replace(/\s+/g,' ').trim().slice(0,12);
 const plain=name.toLowerCase().replace(/[01345]/g,c=>({'0':'o','1':'i','3':'e','4':'a','5':'s'}[c])).replace(/[^a-z]/g,'');
 const literal=name.toLowerCase().replace(/[^a-z]/g,'');
 return name && !BLOCKED.some(w=>plain.includes(w)||literal.includes(w)) ? name : null;
}
async function digest(ip,salt) {
 return Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',new TextEncoder().encode(salt+'|'+ip))),b=>b.toString(16).padStart(2,'0')).join('');
}
async function rate(request,env) {
 if (!env.IP_SALT) throw new Error('unconfigured');
 const hash=await digest(request.headers.get('cf-connecting-ip')||'local',env.IP_SALT);
 const now=Math.floor(Date.now()/1000);
 // SQLite executes the count + insert atomically, including concurrent requests.
 const result=await env.DB.prepare('INSERT INTO submissions(id,ip_hash,created_at) SELECT ?1,?2,?3 WHERE (SELECT COUNT(*) FROM submissions WHERE ip_hash=?2 AND created_at>?4)<6').bind(crypto.randomUUID(),hash,now,now-3600).run();
 await env.DB.prepare('DELETE FROM submissions WHERE created_at<=?1').bind(now-3600).run();
 return result.meta.changes===1;
}
async function top(env) {
 const {results}=await env.DB.prepare('SELECT name,MAX(score) AS score,mode FROM scores GROUP BY name,mode ORDER BY score DESC,name ASC LIMIT 100').all();
 return results.map((r,i)=>({rank:i+1,...r}));
}
async function submit(request,env) {
 if (!(await rate(request,env))) return json({ok:false,error:'too many scores, try later'},429);
 if (Number(request.headers.get('content-length')||0)>2048) return json({ok:false,error:'bad request'},400);
 const text=await request.text();
 if (text.length>2048) return json({ok:false,error:'bad request'},400);
 let body;try { body=JSON.parse(text); } catch {return json({ok:false,error:'bad request'},400);}
 if (!body || typeof body!=='object' || Array.isArray(body)) return json({ok:false,error:'bad request'},400);
 const name=cleanName(body.name);
 if (!name) return json({ok:false,error:'name not allowed'},400);
 if (typeof body.mode!=='string'||!Object.hasOwn(MODES,body.mode)) return json({ok:false,error:'unknown mode'},400);
 const min=Number(env.MIN_SCORE),max=Number(env.MAX_SCORE);
 if (!Number.isSafeInteger(min)||!Number.isSafeInteger(max)||min>max) throw new Error('unconfigured');
 if (!Number.isSafeInteger(body.score)||body.score<min||body.score>max||body.score%50!==0) return json({ok:false,error:'score not accepted'},400);
 if (typeof body.version!=='string'||body.version.length>32||!/^[-.a-zA-Z0-9]+$/.test(body.version)) return json({ok:false,error:'bad request'},400);
 await env.DB.prepare('INSERT INTO scores(name,score,mode,version,created_at) VALUES(?1,?2,?3,?4,?5)').bind(name,body.score,body.mode,body.version,Math.floor(Date.now()/1000)).run();
 return json({ok:true});
}
async function admin(request,url,env) {
 if (!env.ADMIN_KEY || url.searchParams.get('key')!==env.ADMIN_KEY) return missing();
 if (url.pathname==='/admin/entries' && request.method==='GET') {
  const {results}=await env.DB.prepare('SELECT id,name,score,mode,version,created_at FROM scores ORDER BY id DESC LIMIT 300').all();
  return json({ok:true,entries:results});
 }
 if (url.pathname==='/admin/delete' && request.method==='POST') {
  const id=Number(url.searchParams.get('id'));
  if (!Number.isSafeInteger(id)||id<1) return json({ok:false,error:'bad request'},400);
  await env.DB.prepare('DELETE FROM scores WHERE id=?1').bind(id).run();
  return json({ok:true});
 }
 if(url.pathname==='/admin' && request.method==='GET')return page(true);
 return missing();
}
export default {
 async fetch(request,env) {
  const url=new URL(request.url);
  try {
   if(url.pathname.startsWith('/admin')) return await admin(request,url,env);
   if(url.pathname==='/api/top'&&request.method==='GET')return json({ok:true,min:Number(env.MIN_SCORE),modes:MODES,scores:await top(env)});
   if(url.pathname==='/api/submit'&&request.method==='POST')return await submit(request,env);
   if(url.pathname==='/'&&request.method==='GET')return page(false);
   return missing();
  } catch {return json({ok:false,error:'server error'},500);}
 }
};
function page(adminMode) {
 const nonce=crypto.randomUUID();
 return new Response(`<!doctype html><html lang="en"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>SkyPop · ${adminMode?'Moderation':'Global Top 100'}</title>
<style nonce="${nonce}">
*{box-sizing:border-box}body{margin:0;color:#e7f4ff;background:radial-gradient(ellipse at top,#244778,#08152c 65%);font:17px system-ui,sans-serif;min-height:100vh}.wrap{max-width:900px;margin:auto;padding:42px 20px}header{text-align:center}h1{font-size:clamp(58px,15vw,115px);letter-spacing:-5px;margin:0;font-weight:1000;line-height:1.1;text-shadow:0 5px #081021,0 8px #4a2a6c}h1 span:nth-child(3n+1){color:#ff7ab2}h1 span:nth-child(3n+2){color:#ffdb78}h1 span:nth-child(3n){color:#75efde}h2{color:#87ebff;letter-spacing:3px;font-size:22px}.crystal{color:#ffe18d;font-size:40px}.note{color:#aec9e3;line-height:1.6;text-align:center}ol{list-style:none;padding:0;display:grid;gap:10px}li{display:grid;grid-template-columns:55px 1fr auto;gap:15px;align-items:center;padding:18px;border:1px solid #427087;border-radius:15px;background:linear-gradient(110deg,#21334e,#101e34);box-shadow:0 4px #050d1b}.rank{color:#81dce9;font-weight:800}.name{font-weight:800;overflow-wrap:anywhere}.score{color:#ffe4a2;font-weight:900;font-variant-numeric:tabular-nums}li:first-child{border:2px solid #edc564}a,button{color:#85e8f1}button{background:#19394a;border:1px solid #89ceda;padding:12px;border-radius:8px;cursor:pointer;font:inherit}a:focus-visible,button:focus-visible{outline:3px solid #ffe18d}table{width:100%;border-collapse:collapse}td,th{text-align:left;padding:10px;border-bottom:1px solid #426080}.scroll{overflow:auto}footer{margin-top:30px;font-size:14px}@media(max-width:500px){li{padding:12px;gap:8px;grid-template-columns:35px 1fr auto}.wrap{padding:28px 12px}}
</style><div class="wrap"><header><div class="crystal">✦</div><h1><span>S</span><span>K</span><span>Y</span><span>P</span><span>O</span><span>P</span></h1><h2>${adminMode?'SCORE MODERATION':'GLOBAL TOP 100'}</h2><p class="note">THE SHATTERED SKY · ADVENTURE</p></header><p id="status" class="note">Loading…</p><main id="board"></main><footer class="note">Scores are self-reported, not verified. Fake scores remain possible.<br>Only each name’s best Adventure score is ranked. Names are not accounts.<br>Submissions send name, score, mode and game version. A salted address hash is used for an hour of abuse prevention; expired hashes are cleaned on the next submission; this service does not store the raw IP.<br><a href="https://github.com/quietbytehq/skypop/releases">Play SkyPop</a> · <a href="https://quietbytehq.uk">QuietByte</a></footer></div>
<script nonce="${nonce}">
const admin=${adminMode};const board=document.getElementById('board'),status=document.getElementById('status');
const key=admin?new URL(location.href).searchParams.get('key'):'';
function el(tag,text,cls){const n=document.createElement(tag);n.textContent=text;if(cls)n.className=cls;return n;}
async function load(){try{const res=await fetch(admin?'/admin/entries?key='+encodeURIComponent(key):'/api/top');if(!res.ok)throw Error();const data=await res.json();board.replaceChildren();
if(admin){const box=el('div','','scroll'),table=el('table',''),head=el('tr','');for(const s of ['Name','Score','Version','UTC','Action'])head.append(el('th',s));table.append(head);for(const row of data.entries){const tr=el('tr','');for(const value of [row.name,row.score,row.version,new Date(row.created_at*1000).toISOString()])tr.append(el('td',String(value)));const cell=el('td',''),link=el('a','Delete');link.href='#delete-'+row.id;link.addEventListener('click',async e=>{e.preventDefault();if(!confirm('Delete this score?'))return;const r=await fetch('/admin/delete?key='+encodeURIComponent(key)+'&id='+row.id,{method:'POST'});if(r.ok)load();else status.textContent='Could not delete score.';});cell.append(link);tr.append(cell);table.append(tr);}box.append(table);board.append(box);status.textContent='Latest '+data.entries.length+' submissions. Delete removes one submission; an older best may remain.';}
else{const list=el('ol','');for(const row of data.scores){const li=el('li','');li.append(el('span','#'+row.rank,'rank'),el('span',row.name,'name'),el('span',row.score.toLocaleString('en-GB'),'score'));list.append(li);}board.append(list);status.textContent=data.scores.length?'Score '+data.min.toLocaleString('en-GB')+' or more to join the board.':'No scores yet. Score '+data.min.toLocaleString('en-GB')+' or more and be the first!';}}
catch{status.textContent='The scoreboard could not be loaded. Please try again.';}}
load();if(!admin)setInterval(load,60000);
</script></html>`,{headers:{...headers,'content-type':'text/html; charset=utf-8','content-security-policy':`default-src 'none'; script-src 'nonce-${nonce}'; style-src 'nonce-${nonce}'; connect-src 'self'; base-uri 'none'; frame-ancestors 'none'; form-action 'self'`}});
}
