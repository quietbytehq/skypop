# Called only by the local Worker integration suite; never uses the public URL.
extends SceneTree
func _initialize():call_deferred("run")
func run():
	var game=load("res://scenes/main.tscn").instantiate();root.add_child(game)
	game.set_physics_process(false);game.set_process(false)
	game.online_endpoint="http://127.0.0.1:8793/api/submit"
	game.online_enabled=true;game._finish_run("Local test",false);game.name_entry="GAMEQA";game.score=2000
	game._submit_score()
	if game.game_state!="leaderboard":quit(1);return
	var deadline=Time.get_ticks_msec()+6000
	while Time.get_ticks_msec()<deadline and not game.find_children("*","HTTPRequest",true,false).is_empty():await process_frame
	game.music_player.stop()
	game.queue_free();await process_frame;quit(0)
