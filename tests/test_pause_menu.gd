extends SceneTree
var viewport: SubViewport
var checks := 0
var failures := 0
func check(value: bool, label: String):
	checks += 1
	if not value:failures += 1;push_error(label)
func _initialize():call_deferred("run")
func key(code: Key):
	var event := InputEventKey.new();event.pressed=true;event.keycode=code
	viewport.push_input(event, true)
func click(at: Vector2):
	var event := InputEventMouseButton.new();event.position=at;event.pressed=true;event.button_index=MOUSE_BUTTON_LEFT
	viewport.push_input(event, true)
func pad(button: int):
	var event := InputEventJoypadButton.new();event.pressed=true;event.button_index=button
	viewport.push_input(event, true)
func run():
	var game=load("res://scenes/main.tscn").instantiate();root.add_child(game)
	viewport=SubViewport.new();viewport.size=Vector2i(1280,720);viewport.render_target_update_mode=SubViewport.UPDATE_ALWAYS;root.add_child(viewport);game.reparent(viewport)
	game.set_physics_process(false)
	await process_frame
	key(KEY_ESCAPE);check(game.paused and game.game_state=="title","Escape opens pause menu from title")
	key(KEY_ESCAPE);check(not game.paused,"Escape resumes title")
	game._start_stage(1);key(KEY_ESCAPE)
	check(game.paused and game.pause_choice==0,"Escape pauses with Resume selected")
	var position: Vector2=game.balls[0].position
	game._physics_process(1.0);check(game.balls[0].position==position,"Pause freezes simulation")
	key(KEY_DOWN);check(game.pause_choice==1,"Keyboard selects Quit")
	key(KEY_UP);key(KEY_ENTER);check(not game.paused and game.game_state=="playing","Enter resumes selected Resume")
	key(KEY_ESCAPE);click(Vector2(640,345));check(not game.paused,"Mouse Resume works")
	pad(JOY_BUTTON_START);check(game.paused,"Controller Start pauses")
	pad(JOY_BUTTON_DPAD_DOWN);check(game.pause_choice==1,"Controller selects Quit")
	pad(JOY_BUTTON_B);check(not game.paused,"Controller B resumes")
	game.game_state="celebrating";game.celebration_time=2.0;key(KEY_ESCAPE)
	var stage: int=game.stage
	key(KEY_ENTER);check(game.stage==stage and not game.paused,"Resume cannot advance celebration accidentally")
	game.game_state="name_entry";key(KEY_ESCAPE);check(game.paused,"Pause also available during name entry")
	# Last action exercises the actual quit path and exits this process.
	game.queue_redraw()
	if DisplayServer.get_name() != "headless":
		for i in range(5):await process_frame;await RenderingServer.frame_post_draw
		viewport.get_texture().get_image().save_png("/tmp/skypop-pause-menu.png")
	var requested := [false]
	game.desktop_quit_requested.connect(func():requested[0]=true)
	click(Vector2(640,425))
	check(requested[0],"Mouse Quit invokes desktop exit")
	print("PAUSE MENU: ",checks," checks, ",failures," failures")
	quit(0 if failures==0 else 1)
