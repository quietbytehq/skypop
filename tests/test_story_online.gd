extends SceneTree
var checks := 0
var failures := 0
func check(value: bool, message: String):
	checks += 1
	if not value:
		failures += 1
		push_error(message)
func _initialize(): call_deferred("run_tests")
func run_tests():
	var game = load("res://scenes/main.tscn").instantiate()
	root.add_child(game)
	game.set_physics_process(false)
	game.set_process(false)
	game._begin_game()
	check(game.game_state == "story" and game.story_page == 0, "Opening appears before play")
	var original: Vector2 = game.balls[0].position
	game._physics_process(1.0)
	check(game.balls[0].position == original, "Story freezes gameplay")
	game._advance_story()
	check(game.story_page == 0, "Transition ignores immediate held input")
	for i in range(3):
		game.story_time = 1.0
		game._advance_story()
	check(game.game_state == "playing" and game.score == 0 and game.stage == 1, "Three cards start fresh adventure")
	game._start_stage(10);game.balls.clear();game._check_stage_clear();game.celebration_time = 2.0
	var final_score: int = game.score
	game._continue_stage()
	check(game.game_state == "story" and game.story_ending, "Final clear opens victory story")
	for i in range(3):
		game.story_time = 1.0;game._advance_story()
	check(game.game_state == "name_entry" and game.run_won and game.score == final_score, "Ending preserves score and opens name entry")
	game._finish_run("No continues left", false)
	check(game.game_state == "name_entry" and not game.run_won, "Defeat does not show victory story")
	game.online_enabled = true
	var event := InputEventKey.new();event.pressed = true;event.keycode = KEY_F2
	game._unhandled_input(event)
	check(not game.online_enabled and not root.get_node("SaveGame").get_online_enabled(), "F2 remembers local-only choice")
	game.name_entry = "LOCAL";game.score = 1500;game._submit_score()
	check(game.game_state == "leaderboard", "Local score continues immediately")
	check(game.find_children("*", "HTTPRequest", true, false).is_empty(), "Local-only choice sends no request")
	check(not game.OnlineScores.qualifying(950) and not game.OnlineScores.qualifying(1001) and not game.OnlineScores.qualifying(1000050), "Client rejects nonqualifying score")
	check(game.OnlineScores.clean_name("Ab<c>!") == "ABC", "Client restricts name characters")
	var body: Dictionary = game.OnlineScores.payload("NETTEST", 1000)
	check(body.size() == 4 and body.keys().all(func(k): return k in ["name","score","mode","version"]), "Only four allowed fields are sent")
	# TCP capture exercises the real game submission path, never the proposed public hostname.
	var server := TCPServer.new()
	check(server.listen(18794,"127.0.0.1") == OK, "Local HTTP capture starts")
	game.online_endpoint = "http://127.0.0.1:18794/api/submit"
	game.online_enabled = true;game._finish_run("Test", false);game.name_entry = "NETTEST";game.score = 1000
	game._submit_score()
	check(game.game_state == "leaderboard", "Submission never waits for network response")
	var deadline := Time.get_ticks_msec()+5000
	var peer: StreamPeerTCP
	var captured := ""
	while Time.get_ticks_msec()<deadline:
		if peer == null and server.is_connection_available():peer=server.take_connection()
		if peer != null:
			peer.poll()
			if peer.get_available_bytes()>0:captured += peer.get_utf8_string(peer.get_available_bytes())
			if captured.contains("\r\n\r\n") and captured.ends_with("}"):break
		await process_frame
	check(captured.begins_with("POST /api/submit"), "Game sends a real HTTP POST locally")
	var parts := captured.split("\r\n\r\n",true,1)
	var parsed = JSON.parse_string(parts[1]) if parts.size()>1 else null
	check(parsed is Dictionary and parsed.size()==4 and parsed.name==body.name and int(parsed.score)==body.score and parsed.mode==body.mode and parsed.version==body.version, "Actual transmitted JSON matches four-field contract")
	if peer != null:
		peer.put_data("HTTP/1.1 200 OK\r\nContent-Length: 11\r\nConnection: close\r\n\r\n{\"ok\":true}".to_utf8_buffer())
	for i in range(20):await process_frame
	check(game.find_children("*", "HTTPRequest", true, false).is_empty(), "Completed request cleans up")
	if peer != null:peer.disconnect_from_host()
	server.stop()
	game._finish_run("Offline", false);game._submit_score()
	check(game.game_state == "leaderboard", "Offline request also continues immediately")
	await create_timer(5.5).timeout
	check(game.find_children("*", "HTTPRequest", true, false).is_empty(), "Failed request cleans up without retries")
	game.queue_free();await process_frame
	print("STORY / ONLINE: ",checks," checks, ",failures," failures")
	quit(0 if failures==0 else 1)
