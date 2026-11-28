extends Node
## Let the desktop launch sandboxed browsers instead of inheriting game restrictions.
signal opened
signal failed
var platform=OS.get_name()
var portal_execute: Callable=OS.execute
var shell_open: Callable=OS.shell_open
var worker: Thread
var pending_url=""

func open_url(url: String) -> Error:
 if worker!=null:return OK
 if platform!="Linux":return _shell(url)
 pending_url=url
 worker=Thread.new()
 var result=worker.start(_portal.bind(url))
 if result!=OK:
  worker=null
  return _shell(url)
 return OK

func _portal(url: String) -> int:
 var output=[]
 var executable="gdbus"
 var runtime=OS.get_environment("SNAP_DESKTOP_RUNTIME")
 if not runtime.is_empty() and FileAccess.file_exists(runtime.path_join("usr/bin/gdbus")):
  executable=runtime.path_join("usr/bin/gdbus")
 return portal_execute.call(executable,PackedStringArray([
  "call","--session","--timeout","5","--dest","org.freedesktop.portal.Desktop",
  "--object-path","/org/freedesktop/portal/desktop",
  "--method","org.freedesktop.portal.OpenURI.OpenURI","",url,"{}"
 ]),output,true)

func _shell(url: String) -> Error:
 var result: Error=shell_open.call(url)
 if result==OK:opened.emit()
 return result

func _process(_delta: float):
 if worker==null or worker.is_alive():return
 var result=worker.wait_to_finish()
 worker=null
 if result==0:opened.emit()
 elif _shell(pending_url)!=OK:failed.emit()
 pending_url=""

func _exit_tree():
 if worker!=null:
  worker.wait_to_finish()
  worker=null
