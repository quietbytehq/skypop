import {test} from 'node:test';
import assert from 'node:assert/strict';
import {mkdtempSync,writeFileSync,readFileSync,rmSync} from 'node:fs';
import {tmpdir} from 'node:os';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {spawn,spawnSync} from 'node:child_process';
import {randomBytes} from 'node:crypto';
const root=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'..');
const temp=mkdtempSync(path.join(tmpdir(),'skypop-worker-'));
const config=path.join(temp,'wrangler.toml'),state=path.join(temp,'state');
const key=randomBytes(32).toString('hex'),salt=randomBytes(32).toString('hex');
writeFileSync(config,readFileSync(path.join(root,'wrangler.toml'),'utf8').replace('main = "src/worker.js"',`main = ${JSON.stringify(path.join(root,'src/worker.js'))}`).replace(/^routes = .*$/m,''));
writeFileSync(path.join(temp,'.dev.vars'),JSON.stringify({ADMIN_KEY:key,IP_SALT:salt}),{mode:0o600});
// Wrangler dotenv format, no real credentials are used.
writeFileSync(path.join(temp,'.dev.vars'),`ADMIN_KEY=${key}\nIP_SALT=${salt}\n`,{mode:0o600});
const wrangler=path.join(root,'node_modules/.bin/wrangler');
function command(args){const r=spawnSync(wrangler,args,{cwd:temp,env:{...process.env,WRANGLER_SEND_METRICS:'false'},encoding:'utf8'});assert.equal(r.status,0,'local Wrangler command succeeds');return r.stdout;}
const base='http://127.0.0.1:8793';
let worker;
async function post(body,ip='192.0.2.1'){return fetch(base+'/api/submit',{method:'POST',headers:{'content-type':'application/json','cf-connecting-ip':ip},body:JSON.stringify(body)});}
const valid={name:'ACE',score:1000,mode:'adventure',version:'1.1.0'};
test('local Wrangler/D1 scoreboard integration', {timeout:90000},async t=>{
 try {
 command(['d1','execute','skypop-scores','--local','--config',config,'--persist-to',state,'--file',path.join(root,'schema.sql')]);
 worker=spawn(wrangler,['dev','--local','--config',config,'--persist-to',state,'--port','8793','--ip','127.0.0.1'],{cwd:temp,env:{...process.env,WRANGLER_SEND_METRICS:'false'},stdio:'ignore',detached:true});
 let ready=false;for(let i=0;i<100;i++){try{if((await fetch(base+'/api/top')).ok){ready=true;break;}}catch{}await new Promise(r=>setTimeout(r,200));}assert.ok(ready,'wrangler dev starts');
 await t.test('valid score, best per name and uppercase',async()=>{assert.equal((await post({...valid,name:'Ace'})).status,200);assert.equal((await post({...valid,score:1500})).status,200);const data=await (await fetch(base+'/api/top')).json();assert.equal(data.scores.length,1);assert.equal(data.scores[0].name,'ACE');assert.equal(data.scores[0].score,1500);});
 let ip=10;
 for(const [label,body] of [['below minimum',{...valid,score:950}],['impossible score',{...valid,score:1000050}],['wrong increment',{...valid,score:1001}],['fractional score',{...valid,score:1000.5}],['string score',{...valid,score:'1000'}],['blocked name',{...valid,name:'SH1T'}],['unknown mode',{...valid,mode:'__proto__'}],['array mode',{...valid,mode:['adventure']}],['null body',null]]){
  await t.test(label,async()=>assert.equal((await post(body,'192.0.2.'+(ip++))).status,400));
 }
 await t.test('rate limit is six including concurrent submissions',async()=>{const replies=await Promise.all(Array.from({length:10},()=>post(valid,'192.0.2.99')));assert.equal(replies.filter(r=>r.status===200).length,6);assert.equal(replies.filter(r=>r.status===429).length,4);});
 await t.test('admin hidden without correct key',async()=>{for(const route of ['/admin','/admin?key=wrong','/admin/entries','/admin/delete?id=1'])assert.equal((await fetch(base+route)).status,404);});
 await t.test('actual Godot game posts to local Worker',async()=>{
  const result=spawnSync(process.env.GODOT||'godot',['--headless','--path',path.dirname(root),'--audio-driver','Dummy','--script','tests/test_game_to_worker.gd'],{env:{...process.env,XDG_DATA_HOME:path.join(temp,'game-data'),XDG_CONFIG_HOME:path.join(temp,'game-config')},encoding:'utf8',timeout:30000});
  assert.equal(result.status,0,'game submission process succeeds');
  const data=await (await fetch(base+'/api/top')).json();assert.ok(data.scores.some(row=>row.name==='GAMEQA'&&row.score===2000));
 });
 await t.test('admin page, list and delete',async()=>{assert.equal((await fetch(base+'/admin?key='+key)).status,200);const entries=(await (await fetch(base+'/admin/entries?key='+key)).json()).entries;assert.ok(entries.length>0);for(const row of entries){assert.equal((await fetch(base+'/admin/delete?key='+key+'&id='+row.id,{method:'POST'})).status,200);}assert.equal((await (await fetch(base+'/api/top')).json()).scores.length,0);assert.equal((await post(valid,'192.0.2.99')).status,429,'deleting scores cannot reset rate limit');});
 await t.test('themed page uses safe DOM and no external resources',async()=>{const r=await fetch(base);const html=await r.text();assert.ok(html.includes('SKY')||html.includes('<span>S</span>'));assert.ok(html.includes('textContent'));assert.ok(!html.includes('innerHTML'));assert.ok(r.headers.get('content-security-policy').includes("frame-ancestors 'none'"));});
 await t.test('only full salted hashes stored',()=>{const out=command(['d1','execute','skypop-scores','--local','--config',config,'--persist-to',state,'--command','SELECT ip_hash FROM submissions','--json']);const rows=JSON.parse(out)[0].results;assert.ok(rows.length>0);assert.ok(rows.every(r=>/^[a-f0-9]{64}$/.test(r.ip_hash)));assert.ok(!out.includes('192.0.2.'));});
 } finally {if(worker){try{process.kill(-worker.pid,'SIGTERM');}catch{}}rmSync(temp,{recursive:true,force:true});}
});

test('unexpected failures expose no details',async()=>{const {default:worker}=await import('../src/worker.js');const response=await worker.fetch(new Request('http://localhost/api/top'),{MIN_SCORE:'1000',DB:{prepare(){throw Error('private database details');}}});assert.equal(response.status,500);assert.deepEqual(await response.json(),{ok:false,error:'server error'});});
