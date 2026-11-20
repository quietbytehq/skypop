"""User-run deployment. Secrets and logs stay outside the repository."""
import json, os, pathlib, re, secrets, subprocess, sys
root = pathlib.Path(__file__).resolve().parent
private = pathlib.Path.home() / '.config' / 'skypop-scores'
private.mkdir(parents=True, exist_ok=True, mode=0o700)
os.chmod(private, 0o700)
secret_file = private / 'secrets.json'
admin_file = pathlib.Path.home() / '.config' / 'skypop-scores-admin.txt'
log_file = private / 'deploy.log'
def write_private(path, value):
    fd = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_TRUNC, 0o600)
    os.fchmod(fd, 0o600)
    with os.fdopen(fd, 'w') as f: f.write(value)
def run(*args):
    result = subprocess.run(['npx', '--no-install', 'wrangler', *args], cwd=root, text=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    with log_file.open('a') as f: f.write(result.stderr)
    if result.returncode:
        raise RuntimeError('Cloudflare command failed; inspect the private deployment log locally.')
    return result.stdout
try:
    write_private(log_file, '')
    if not secret_file.exists():
        if admin_file.exists(): raise RuntimeError('Admin link exists but secrets file is missing. Restore the secrets file before deploying; keys were not rotated.')
        write_private(secret_file, json.dumps({'ADMIN_KEY':secrets.token_hex(32), 'IP_SALT':secrets.token_hex(32)}))
    values = json.loads(secret_file.read_text())
    if any(not re.fullmatch(r'[0-9a-f]{64}', values.get(k,'')) for k in ['ADMIN_KEY','IP_SALT']): raise RuntimeError('Private secrets file is invalid.')
    os.chmod(secret_file,0o600)
    databases=json.loads(run('d1','list','--json'))
    database=next((d for d in databases if d['name']=='skypop-scores'), None)
    if database is None:
        print('Creating SkyPop database…')
        run('d1','create','skypop-scores')
        database=next(d for d in json.loads(run('d1','list','--json')) if d['name']=='skypop-scores')
    config=root/'wrangler.toml'
    config.write_text(re.sub(r'database_id = "[^"]+"', 'database_id = "'+database['uuid']+'"', config.read_text()))
    print('Applying database schema…')
    run('d1','execute','skypop-scores','--remote','--file=schema.sql','--yes')
    print('Deploying scoreboard and secrets together…')
    run('deploy','--secrets-file',str(secret_file))
    write_private(admin_file, 'https://skypop.quietbytehq.uk/admin?key='+values['ADMIN_KEY']+'\n')
    print('Deployed SkyPop scoreboard. The private admin link is saved in ~/.config/skypop-scores-admin.txt.')
    print('Do not paste that link or the private deployment log into chat or GitHub.')
except Exception:
    # A subprocess error or parsed value must never expose a key in terminal output.
    print('Deployment stopped. Check login, account access and ~/.config/skypop-scores/deploy.log locally. No secrets were printed.', file=sys.stderr)
    sys.exit(1)
