extends SceneTree
var checks=0
var failures=0
func check(value: bool,label: String):
	checks+=1
	if not value:failures+=1;push_error(label)
func _initialize():call_deferred("run")
func key(code: Key):
	var e=InputEventKey.new();e.pressed=true;e.keycode=code;root.push_input(e,true)
func click(point: Vector2):
	var e=InputEventMouseButton.new();e.position=point;e.pressed=true;e.button_index=MOUSE_BUTTON_LEFT;root.push_input(e,true)
func pad():
	var e=InputEventJoypadButton.new();e.pressed=true;e.button_index=JOY_BUTTON_Y;root.push_input(e,true)
func run():
	var game=load("res://scenes/main.tscn").instantiate();root.add_child(game);game.set_physics_process(false)
	await process_frame
	var calls=[];game.browser_opener=func(url):calls.append(url);return OK
	for screen in ["title","leaderboard"]:
		game.game_state=screen
		key(KEY_G);check(calls.size()==1 and game.game_state==screen,"Keyboard opens global board without starting a run")
		pad();check(calls.size()==2 and game.game_state==screen,"Gamepad opens global board without starting a run")
		click(game.GLOBAL_TITLE_BUTTON.get_center() if screen=="title" else game.GLOBAL_BOARD_BUTTON.get_center())
		check(calls.size()==3 and game.game_state==screen,"Mouse global button leaves current screen intact")
		check(calls.all(func(url):return url=="https://skypop.quietbytehq.uk"),"Only SkyPop's public scoreboard is opened")
		calls.clear()
	game.game_state="title";game.settings_open=true;key(KEY_G)
	check(calls.is_empty(),"Settings keyboard input cannot open browser")
	game.settings_open=false;game.browser_opener=func(_url):return ERR_CANT_FORK
	check(game.open_global_scores()==ERR_CANT_FORK,"Browser failure returns an error")
	check(not game.find_children("*","AcceptDialog",true,false).is_empty(),"Failed browser launch gives a visible web address")
	game.queue_free();await process_frame
	print("GLOBAL BROWSER: ",checks," checks, ",failures," failures")
	quit(0 if failures==0 else 1)
