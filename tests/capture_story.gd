extends SceneTree
var viewport: SubViewport
func _initialize():call_deferred("run")
func capture(game, filename: String):
	game.queue_redraw()
	for i in range(5):await process_frame;await RenderingServer.frame_post_draw
	viewport.get_texture().get_image().save_png("/tmp/"+filename+".png")
func run():
	var game=load("res://scenes/main.tscn").instantiate();root.add_child(game)
	game.set_physics_process(false);game.set_process(false)
	viewport=SubViewport.new();viewport.size=Vector2i(1280,720);viewport.render_target_update_mode=SubViewport.UPDATE_ALWAYS;root.add_child(viewport);game.reparent(viewport)
	game._begin_game();game.story_time=1.0
	await capture(game,"skypop-opening")
	game._start_story(true);game.story_page=2;game.story_time=3.2;game.score=30250
	await capture(game,"skypop-ending")
	game._finish_story();game.name_entry="SKY HERO";game.online_enabled=true
	await capture(game,"skypop-online-choice")
	game.online_enabled=false
	await capture(game,"skypop-local-choice")
	viewport.queue_free();await process_frame;quit()
