extends SceneTree
func _initialize():call_deferred("run")
func frames():
 for i in range(8):await process_frame;await RenderingServer.frame_post_draw
func run():
 var game=load("res://scenes/main.tscn").instantiate();root.add_child(game);game.set_physics_process(false)
 var viewport=SubViewport.new();viewport.size=Vector2i(1280,720);viewport.render_target_update_mode=SubViewport.UPDATE_ALWAYS;root.add_child(viewport);game.reparent(viewport)
 game.game_state="title";await frames();viewport.get_texture().get_image().save_png("res://build/1.1.1/validation/title-global.png")
 var opened=[];game.browser.opened.connect(func():opened.append(true))
 var event=InputEventMouseButton.new();event.position=game.GLOBAL_TITLE_BUTTON.get_center();event.pressed=true;event.button_index=MOUSE_BUTTON_LEFT;viewport.push_input(event,true)
 for i in range(600):
  if not opened.is_empty():break
  await process_frame
 print("REAL SKYPOP BROWSER: ","PASS" if not opened.is_empty() else "FAIL")
 game.game_state="leaderboard";game.queue_redraw();await frames();viewport.get_texture().get_image().save_png("res://build/1.1.1/validation/scores-global.png")
 game.queue_free();await process_frame;quit(0 if not opened.is_empty() else 1)
