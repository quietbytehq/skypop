extends SceneTree
const Browser=preload("res://scripts/browser.gd")
const URL="https://skypop.quietbytehq.uk"
var checks=0
var failures=0
func check(ok: bool,label: String):
 checks+=1
 if not ok:failures+=1;push_error(label)
func _initialize():call_deferred("run")
func wait_for_browser(browser):
 for i in range(300):
  if browser.worker==null:return
  await create_timer(0.01).timeout
 check(false,"Browser request finishes")
func run():
 var browser=Browser.new();root.add_child(browser)
 browser.platform="Linux"
 var requested=[];var shell=[];var events=[]
 browser.opened.connect(func():events.append("opened"))
 browser.failed.connect(func():events.append("failed"))
 browser.portal_execute=func(command,args,_output,_stderr):
  requested.append([command,args]);OS.delay_msec(80);return 0
 browser.shell_open=func(url):shell.append(url);return OK
 var started=Time.get_ticks_msec()
 check(browser.open_url(URL)==OK,"Browser request accepted")
 check(Time.get_ticks_msec()-started<50,"Browser launch does not block the game")
 browser.open_url(URL)
 await wait_for_browser(browser)
 check(requested.size()==1,"Repeated clicks do not launch duplicate pending requests")
 var args=requested[0][1]
 check(requested[0][0]=="gdbus" and "org.freedesktop.portal.OpenURI.OpenURI" in args and URL in args,"Desktop opens the exact global board URL")
 check("--timeout" in args and "5" in args,"Unavailable desktop service has bounded wait")
 check(shell.is_empty() and events==["opened"],"Successful portal does not also launch browser directly")
 browser.portal_execute=func(_command,_args,_output,_stderr):return 1
 events.clear();browser.open_url(URL);await wait_for_browser(browser)
 check(shell==[URL] and events==["opened"],"Missing portal falls back to system browser")
 events.clear();browser.shell_open=func(_url):return ERR_CANT_FORK
 browser.open_url(URL);await wait_for_browser(browser)
 check(events==["failed"],"Asynchronous failures notify the interface")
 events.clear();requested.clear();browser.platform="Windows"
 check(browser.open_url(URL)==ERR_CANT_FORK and browser.worker==null and requested.is_empty(),"Windows uses native browser handler and returns failures")
 browser.shell_open=func(url):shell.append(url);return OK
 check(browser.open_url(URL)==OK and events==["opened"],"Windows native browser success is reported")
 browser.queue_free();await process_frame
 print("BROWSER: ",checks," checks, ",failures," failures")
 quit(0 if failures==0 else 1)
