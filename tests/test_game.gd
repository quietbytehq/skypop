extends SceneTree
var checks := 0
var failures := 0
func check(value: bool, message: String):
	if not value:
		push_error(message)
		failures += 1
		return
	checks += 1
func _initialize():
	call_deferred("run_tests")
func run_tests():
	var game = load("res://scenes/main.tscn").instantiate()
	root.add_child(game)
	check(game.reaction_frames.size() == 4, "Four reaction portrait frames loaded")
	check(game._reaction_frame(true, 0.82) == 2, "Victory portrait blinks")
	check(game._reaction_frame(true, 0.3) == 1 and game._reaction_frame(true, 0.0) == 0, "Laugh alternates mouth poses")
	check(game._reaction_frame(false, 2.6) == 3, "Defeat portrait stays sad")
	var has_audio: bool = game.music_player.stream != null
	check(game.game_state == "title" and game.title_art != null, "Arcade artwork loads at startup")
	if has_audio:
		check(game.music_player.stream.resource_path.ends_with("title_music.mp3"), "Dedicated title music selected at startup")
	if has_audio:
		check(game.music_player.stream.get_length() > 44.0 and game.music_player.stream.loop, "Full title track loops")
	game.title_time = 0.3
	check(game._title_prompt_visible(), "Title prompt visible")
	game.title_time = 1.2
	check(not game._title_prompt_visible(), "Title prompt blinks")
	var start_key := InputEventKey.new()
	start_key.pressed = true
	start_key.keycode = KEY_B
	game._unhandled_input(start_key)
	check(game.game_state == "playing", "Arbitrary keyboard key starts game")
	if has_audio:
		check(game.music_player.stream.resource_path.ends_with("/music.mp3"), "Gameplay track replaces title music on start")
	check(game.start_fire_blocked, "Starting input cannot immediately fire")
	game.set_physics_process(false)
	game._update_player(1.0 / 120.0)
	check(not game.start_fire_blocked, "Starting fire lock releases after input release")
	game.game_state = "title"
	Input.action_press("fire")
	game._begin_game()
	game._update_player(1.0 / 120.0)
	check(game.projectiles.is_empty(), "Held start key does not shoot on entering stage")
	Input.action_release("fire")
	game._update_player(1.0 / 120.0)
	Input.action_press("fire")
	game._update_player(1.0 / 120.0)
	Input.action_release("fire")
	check(game.projectiles.size() == 1, "Fire works after releasing start input")
	game._start_stage(1)
	game.game_state = "playing"
	game.set_physics_process(false)
	check(Engine.physics_ticks_per_second == 120, "120 Hz physics")
	if has_audio:
		check(game.music_player.stream.get_length() >= 29.9, "Full soundtrack loaded")
	if has_audio:
		if game.music_player.stream is AudioStreamWAV:
			check(game.music_player.stream.loop_end > 1000000, "Music loops full decoded length")
		else:
			check(game.music_player.stream.loop, "Compressed music loops")
	var original_music: bool = game.music_enabled
	game._toggle_music()
	if has_audio:
		check(game.music_player.stream_paused == not game.music_enabled, "Music toggle pauses playback")
	check(root.get_node("SaveGame").get_audio_settings()["music_enabled"] == game.music_enabled, "Music preference persisted")
	game._toggle_music()
	check(game.music_enabled == original_music, "Music preference restored")
	for sound in game.sounds:
		check(game.sounds[sound] is AudioStreamMP3, "Supplied MP3 selected: " + str(sound))
	if has_audio:
		check(game.music_player.stream is AudioStreamMP3, "Supplied music MP3 selected")
	if has_audio:
		check(game.sounds.has("voice_ouch") and game.sounds.has("voice_fall"), "Spoken reaction assets loaded")
	check(game.collapse_frames.size() == 10, "Ten dedicated collapse strips")
	check(game.stage_backgrounds.size() == 10, "Ten backgrounds")
	check(game.player_walk_frames.size() == 10, "Ten costumes")
	game.hearts = 2
	game._collect_pickup(0)
	check(game.hearts == 3, "Health pickup")
	game._collect_pickup(1)
	check(game.power_time == 10, "Weapon boost")
	game._collect_pickup(2)
	game._take_hit()
	check(game.hearts == 3 and game.shield_time == 0, "Shield absorbs hit")
	game._start_stage(1)
	check(game.power_time == 0 and game.pickups.is_empty(), "Stage resets powers")
	game.game_state = "playing"
	game.player_position.y -= 100
	game._fire_projectile()
	check(is_equal_approx(game.projectiles[0].base_y, game._launcher_origin().y), "Airborne muzzle alignment")
	game.projectiles.clear()
	game.balls.assign([{"position": Vector2(200, 300), "velocity": Vector2(100, 0), "radius": 30.0, "color": 0}])
	game._update_balls(1.0 / 60.0)
	check(game.balls[0].velocity.y > 0, "Gravity")
	game.balls[0].position.y = 580
	game.balls[0].velocity.y = 200
	game._update_balls(1.0 / 60.0)
	check(game.balls[0].velocity.y < 0, "Floor bounce")
	game._pop_ball(0)
	check(game.balls.size() == 2, "Split")
	check(game.balls[0].velocity.x < 0 and game.balls[1].velocity.x > 0, "Opposing split directions")
	game.projectiles.assign([{"x": 200.0, "base_y": 500.0, "tip_y": 200.0}])
	game.balls.assign([{"position": Vector2(200, 350), "velocity": Vector2.ZERO, "radius": 16.0, "color": 0}])
	game._update_projectiles(1.0 / 60.0)
	check(game.balls.is_empty() and game.projectiles.is_empty(), "Rope body collision")
	game.projectiles.assign([{"x": 200.0, "base_y": 500.0, "tip_y": 113.0}])
	game._update_projectiles(1.0 / 60.0)
	check(game.projectiles.is_empty(), "Rope expires at ceiling")
	game.balls.clear()
	game.wire_type = 1
	game._fire_projectile()
	game._update_projectiles(0.6)
	check(game.projectiles.size() == 1, "Anchor remains at ceiling")
	game._update_projectiles(1.5)
	check(game.projectiles.is_empty(), "Anchor expires")
	game.wire_type = 2
	game._fire_projectile()
	var x = game.projectiles[0].x
	game.balls.assign([{"position": Vector2(x, 350), "velocity": Vector2.ZERO, "radius": 16.0, "color": 0}])
	game._update_projectiles(0.3)
	check(game.projectiles.size() == 1 and game.balls.is_empty(), "Pierce survives first hit")
	game.balls.assign([{"position": Vector2(x, 400), "velocity": Vector2.ZERO, "radius": 16.0, "color": 0}])
	game._update_projectiles(0.13)
	check(game.projectiles.is_empty() and game.balls.is_empty(), "Pierce consumes second hit")
	game.wire_type = 0
	game.balls.assign([{"position": Vector2(200, 350), "velocity": Vector2(120, 0), "radius": 20.0, "color": 0}])
	game._pop_ball(0)
	check(game.balls.size() == 2 and game.balls[0].radius < 16.32, "Extra tiny ball split")
	game._pop_ball(0)
	check(game.balls.size() == 1, "Tiny balls disappear")
	game.balls.clear()
	game._collect_pickup(3)
	check(game.roof_ready, "Roof pickup arms next shot")
	game._fire_projectile()
	check(not game.roof_ready and game.projectiles[0].roof, "Roof charge consumed")
	game._update_projectiles(1.0)
	check(game.projectiles.size() == 1 and game.projectiles[0].roof_left == 5, "Roof countdown starts on attachment")
	check(game._moving_rope_count() == 0, "Attached roof trap permits another shot")
	check(game.projectiles[0].base_y == game.PLAYER_FOOT_Y, "Roof trap extends fully to floor")
	var roof_x: float = game.projectiles[0].x
	for hit in range(3):
		game.balls.assign([{"position": Vector2(roof_x, 600), "velocity": Vector2.ZERO, "radius": 12.0, "color": 0}])
		game._update_projectiles(0.2)
		check(game.balls.is_empty() and game.projectiles.size() == 1, "Roof survives repeated hits")
	game._update_projectiles(4.3)
	check(game.projectiles.size() == 1, "Roof remains for 4.9 seconds")
	game._update_projectiles(0.11)
	check(game.projectiles.is_empty(), "Roof expires at five seconds even after hits")
	game._start_stage(1)
	game.score = 500
	game.stage_score = 100
	game.hearts = 1
	game._take_hit()
	check(game.game_state == "fainting" and game.hearts == 0, "Death begins fainting before retry")
	check(game.continues == 3, "Continue not spent before faint")
	game._physics_process(1.9)
	check(game.continues == 2 and game.hearts == 3 and game.score == 400, "Continue rollback")
	game.paused = true
	game._use_special()
	check(game.special_ready, "Special blocked while paused")
	game.paused = false
	for stage in range(1, 11):
		game._start_stage(stage)
		for step in range(600):
			game.invulnerable_time = 100
			game._update_balls(1.0 / 60.0)
			for ball in game.balls:
				check(game.FIELD.grow(1).has_point(ball.position), "Ball remains inside field")
		game._use_special()
		check(game.game_state == "celebrating", "Stage celebration")
		if has_audio:
			check(game.music_player.stream.resource_path.ends_with("happy_music.mp3") and not game.music_player.stream.loop, "Victory music plays once")
		if has_audio:
			check(game.sounds.has("voice_alright"), "Victory voice loaded")
		var cleared_score: int = game.score
		game._check_stage_clear()
		check(game.score == cleared_score, "Bonus awarded once")
		game._continue_stage()
		check(game.game_state == "celebrating", "Celebration input grace period")
		game.celebration_time = 2
		game._continue_stage()
		check(game.stage == mini(stage + 1, 10), "Stage progression")
	check(game.game_state == "name_entry" and game.run_won, "Victory name entry")
	game._start_stage(1)
	game.continues = 0
	game.hearts = 1
	game._take_hit()
	check(game.game_state == "fainting", "Final death also animates")
	game._physics_process(1.9)
	check(game.game_state == "name_entry" and not game.run_won, "Game over follows faint animation")
	if has_audio:
		check(game.music_player.stream.resource_path.ends_with("sad_music.mp3") and not game.music_player.stream.loop, "Defeat music plays once")
	game._start_stage(1)
	game.projectiles.assign([{"x": 200.0, "base_y": 610.0, "tip_y": 112.0, "roof": true, "roof_left": 5.0}])
	game.fire_cooldown = 0
	Input.action_press("fire")
	game._update_player(1.0 / 120.0)
	Input.action_release("fire")
	check(game.projectiles.size() == 2 and game._moving_rope_count() == 1, "Fire while roof trap is active")
	game._start_stage(1)
	game._collect_pickup(4)
	game._fire_projectile()
	check(game.projectiles.size() == 3, "Triple harpoon volley")
	check(game.projectiles[0].x < game.projectiles[1].x and game.projectiles[1].x < game.projectiles[2].x, "Triple spread")
	game.projectiles.clear()
	game._collect_pickup(5)
	game._fire_projectile()
	check(game.fire_cooldown == 0.14, "Rapid reel cooldown")
	game.projectiles.clear()
	game._collect_pickup(6)
	game._fire_projectile()
	check(game.projectiles[0].hits == 4 and game.projectiles[0].weapon == 6, "Heavy spear pierces four targets")
	game._physics_process(12.1)
	check(game.weapon_kind == 0, "Temporary weapon expires")
	game._collect_pickup(4)
	game._start_stage(2)
	check(game.weapon_kind == 0 and game.weapon_time == 0, "Weapon resets on stage change")
	game._start_stage(1)
	game.player_position.y -= 160
	game.player_vertical_speed = -100
	game.hearts = 1
	game._take_hit()
	check(not game.faint_grounded, "Airborne death starts falling")
	game._physics_process(0.1)
	check(game.faint_time == 0 and not game.faint_grounded, "Collapse waits for landing")
	for tick in range(120):
		if game.faint_grounded:
			break
		game._physics_process(1.0 / 120.0)
	check(game.faint_grounded and game.faint_origin.y == game.PLAYER_GROUND_CENTER_Y, "Gravity lands character")
	for number in range(1, 11):
		game._start_stage(number)
		game.faint_origin = game.player_position
		for facing in [-1.0, 1.0]:
			game.player_facing = facing
			for t in [0.0, 0.2, 0.5, 0.8, 1.2]:
				game.faint_time = t
				var frame: Dictionary = game.collapse_frames[number][game._collapse_frame_index()]
				var rect: Rect2 = game._collapse_frame_rect()
				check(absf(rect.position.y + float(frame["bottom"]) * rect.size.y) < 0.1, "Drawn collapse frame has opaque pixels anchored to floor")
	game._start_stage(1)
	game.balls.assign([{"position": Vector2(300,300), "velocity": Vector2(100,0), "radius": 20.0, "color": 0}, {"position": Vector2(335,300), "velocity": Vector2(-100,0), "radius": 20.0, "color": 1}])
	game._resolve_ball_contacts()
	check(game.balls[0].velocity.x < 0 and game.balls[1].velocity.x > 0, "Touching balls change direction")
	game.hazards.assign([{"kind": "steel", "position": Vector2(-34,557), "velocity": Vector2(200,0), "radius": 28.0, "warning": 0.0}])
	game.invulnerable_time = 100
	game._update_hazards(1)
	check(game.hazards[0].position == Vector2(166,557) and game.hazards[0].velocity == Vector2(200,0), "Steel moves straight")
	game.projectiles.assign([{"x":166.0,"base_y":610.0,"tip_y":112.0,"wire":0}])
	game._update_projectiles(0.01)
	check(game.hazards.size() == 1, "Steel is immune to ropes")
	game._update_hazards(7)
	check(game.hazards.is_empty(), "Steel exits screen")
	game.hazards.assign([{"kind":"bomb", "position":Vector2(800,300), "velocity":Vector2.ZERO, "radius":25.0, "fuse":0.01}])
	game._update_hazards(0.02)
	check(game.hazards.size() == 8 and game.hazards[0].kind == "fire", "Bomb bursts into eight fireballs")
	game._update_hazards(3)
	check(game.hazards.is_empty(), "Fireballs expire")
	game.hazards.assign([{"kind":"fire", "position":game.player_position, "velocity":Vector2.ZERO, "radius":11.0, "life":2.0}])
	game.invulnerable_time = 0
	game._update_hazards(0.01)
	check(game.hearts == 2, "Fireball damages player")
	game._start_stage(3)
	game._update_hazards(4.01)
	check(game.hazards.size() == 1 and game.hazards[0].kind == "steel", "Steel introduced stage three")
	game._start_stage(5)
	game._update_hazards(0.01)
	check(game.hazards.is_empty(), "Stage starts without immediate hazard")
	var last_speed := 0.0
	for number in range(1, 11):
		game._start_stage(number)
		var speed: float = absf(game.balls[0].velocity.x)
		check(speed > last_speed, "Each stage has faster initial balls")
		check(is_equal_approx(speed, 160.0 * (0.55 + (number - 1) * 0.1)), "Stage speed follows shared pace table")
		last_speed = speed
		game._spawn_steel()
		check(is_equal_approx(absf(game.hazards.back().velocity.x), 220.0 * game._stage_speed()), "Steel uses stage pace")
	print("PASS" if failures == 0 else "FAIL", ": ", checks, " assertions; failures=", failures)
	game.queue_free()
	await process_frame
	quit(0 if failures == 0 else 1)
