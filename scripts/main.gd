extends Node2D

const PLAYER_FOOT_Y := 610.0
const HURT_DURATION := 1.15
const COLLAPSE_CELL_SIZE := 92.0
const SEATED_HEIGHT := 84.0
const PLAYER_DRAW_SIZE := Vector2(76.0, 112.0)
const PLAYER_HALF_HEIGHT := PLAYER_DRAW_SIZE.y * 0.5
const PLAYER_GROUND_CENTER_Y := PLAYER_FOOT_Y - PLAYER_HALF_HEIGHT
const FIELD := Rect2(40.0, 112.0, 1200.0, PLAYER_FOOT_Y - 112.0)
const PLAYER_RADIUS := 17.0
const HEARTS_PER_STAGE := 3
const STARTING_CONTINUES := 3
const MAX_STAGE := 10
const PLAYER_SPEED := 400.0
const PLAYER_ACCELERATION := 1800.0
const PLAYER_FRICTION := 2400.0
const JUMP_SPEED := 590.0
const PLAYER_GRAVITY := 1550.0
const HARPOON_SPEED := 1300.0
const BALL_GRAVITY := 650.0
const REMAPPABLE_ACTIONS := ["move_left", "move_right", "move_up", "move_down", "fire", "special"]

var game_state := "playing"
var stage := 1
var celebration_time := 0.0
var stage_bonus := 0
var wire_type := 0
var pickups: Array[Dictionary] = []
var roof_ready := false
var reaction_frames: Array[Texture2D] = []
var outcome_time := 0.0
var run_won := false
var title_art: Texture2D
var title_time := 0.0
var start_fire_blocked := false
var body_font: Font
var score_font: Font
var logo_art: Texture2D
var logo_font: Font
var weapon_time := 0.0
var weapon_kind := 0
var pickup_notice := ""
var notice_time := 0.0
var power_time := 0.0
var shield_time := 0.0
var pops_since_drop := 0
const WIRE_NAMES := ["SWIFT", "ANCHOR", "PIERCE"]
const WIRE_COLORS := [Color("7deaff"), Color("ffd071"), Color("e4a1ff")]
var hurt_time := 0.0
var faint_time := 0.0
var faint_origin := Vector2.ZERO
var faint_velocity := 0.0
var faint_grounded := true
var collapse_frames: Dictionary = {}
var hearts := HEARTS_PER_STAGE
var continues := STARTING_CONTINUES
var score := 0
var stage_score := 0
var leaderboard_entries: Array = []
var special_ready := true
var player_position := Vector2(640.0, PLAYER_GROUND_CENTER_Y)
var player_movement := Vector2.ZERO
var player_velocity_x := 0.0
var player_walk_phase := 0.0
var player_vertical_speed := 0.0
var player_is_airborne := false
var player_is_sitting := false
var player_landing_time := 0.0
var player_shot_kick := 0.0
var player_facing := 1.0
var idle_glance_time := 0.0
var idle_glance_delay := 2.5
var idle_glancing := false
var previous_gamepad_jump := false
var aim_direction := Vector2.UP
var fire_cooldown := 0.0
var player_fire_pose_time := 0.0
var invulnerable_time := 0.0
var stage_banner_time := 1.7
var lightning_time := 0.0
var name_entry := ""
var finish_message := ""
var settings_open := false
var paused := false
var pop_effects: Array[Dictionary] = []
var settings_cursor := 0
var remapping_action := ""
var music_enabled := true
var active_sfx: Dictionary = {}
var previous_player_position := Vector2(640, PLAYER_GROUND_CENTER_Y)
var music_volume := 0.35
var effects_volume := 0.8
var balls: Array[Dictionary] = []
var hazards: Array[Dictionary] = []
var steel_timer := 4.0
var bomb_timer := 8.0
var projectiles: Array[Dictionary] = []
var weather_particles: Array[Dictionary] = []
var background_stars: Array[Vector2] = []
var rng := RandomNumberGenerator.new()
var sounds: Dictionary = {}
var stage_backgrounds: Dictionary = {}
var player_costumes: Array[Texture2D] = []
var player_walk_frames: Dictionary = {}
var player_sitting_poses: Dictionary = {}
var audio_player: AudioStreamPlayer
var music_player: AudioStreamPlayer


func _ready() -> void:
	rng.randomize()
	logo_font = load("res://assets/fonts/LilitaOne-Regular.ttf")
	body_font = load("res://assets/fonts/Lato-Semibold.ttf")
	score_font = load("res://assets/fonts/Lato-Bold.ttf")
	logo_art = load("res://assets/ui/skypop_logo.png")
	audio_player = AudioStreamPlayer.new()
	add_child(audio_player)
	music_player = AudioStreamPlayer.new()
	add_child(music_player)
	var audio_settings: Dictionary = SaveGame.get_audio_settings()
	music_enabled = bool(audio_settings.get("music_enabled", true))
	music_volume = float(audio_settings["music"])
	effects_volume = float(audio_settings["effects"])
	_load_input_mappings()
	_load_sounds()
	_load_player_costumes()
	_normalize_character_frames()
	_prepare_collapse_frames()
	_load_reaction_portraits()
	for stage_number in range(1, 11):
		var background_path := "res://assets/backgrounds/stage_%02d_" % stage_number
		var background_files := {
			1: "egypt.png",
			2: "taj_mahal.png",
			3: "antarctica.png",
			4: "tropical_island.png",
			5: "tokyo_night.png",
			6: "new_york.png",
			7: "great_wall.png",
			8: "rome.png",
			9: "highlands.png",
			10: "moon_earth.png"
		}
		var background := load(background_path + background_files[stage_number]) as Texture2D
		if background != null:
			stage_backgrounds[stage_number] = background
	_initialize_background()
	_start_stage(1)
	game_state = "title"
	title_art = load("res://assets/backgrounds/title_arcade.png")
	_load_music()


func _load_player_costumes() -> void:
	var costume_sheet := load("res://assets/characters/skypop_costume_sheet.png") as Texture2D
	if costume_sheet == null:
		return
	var cell_width := float(costume_sheet.get_width()) / 5.0
	var cell_height := float(costume_sheet.get_height()) / 2.0
	for stage_index in range(MAX_STAGE):
		var costume := AtlasTexture.new()
		costume.atlas = costume_sheet
		var column := stage_index % 5
		var row := int(stage_index / 5)
		costume.region = Rect2(column * cell_width, row * cell_height, cell_width, cell_height)
		player_costumes.append(costume)
	for stage_number in range(1, MAX_STAGE + 1):
		var walk_path := "res://assets/characters/walk_stage_%02d.png" % stage_number
		var walk_sheet := load(walk_path) as Texture2D
		if walk_sheet != null:
			var frame_width := float(walk_sheet.get_width()) / 4.0
			var frames: Array[Texture2D] = []
			for frame_index in range(4):
				var frame := AtlasTexture.new()
				frame.atlas = walk_sheet
				frame.region = Rect2(frame_index * frame_width, 0.0, frame_width, float(walk_sheet.get_height()))
				frames.append(frame)
			player_walk_frames[stage_number] = frames
		var sit_path := "res://assets/characters/sit_stage_%02d.png" % stage_number
		var sit_texture := load(sit_path) as Texture2D
		if sit_texture != null:
			player_sitting_poses[stage_number] = sit_texture


func _trim_frame(texture: Texture2D) -> Texture2D:
	var image := texture.get_image()
	if image == null:
		return texture
	var used := image.get_used_rect()
	if not used.has_area():
		return texture
	# Ignore faint generated shadows below the soles when anchoring the feet.
	var bottom := used.end.y - 1
	while bottom > used.position.y:
		var solid := false
		for x in range(used.position.x, used.end.x):
			if image.get_pixel(x, bottom).a > 0.8:
				solid = true
				break
		if solid:
			break
		bottom -= 1
	used.size.y = bottom - used.position.y + 1
	var cropped := AtlasTexture.new()
	cropped.atlas = texture
	cropped.region = Rect2(used)
	return cropped


func _normalize_character_frames() -> void:
	for i in range(player_costumes.size()):
		player_costumes[i] = _trim_frame(player_costumes[i])
	for key in player_walk_frames:
		var frames: Array = player_walk_frames[key]
		for i in range(frames.size()):
			frames[i] = _trim_frame(frames[i])
	for key in player_sitting_poses:
		player_sitting_poses[key] = _trim_frame(player_sitting_poses[key])


func _audio_asset(sound_name: String) -> AudioStream:
	for extension in ["mp3", "ogg", "wav"]:
		var path := "res://assets/audio/%s.%s" % [sound_name, extension]
		if ResourceLoader.exists(path):
			return load(path) as AudioStream
	return null


func _load_sounds() -> void:
	for sound_name in ["fire", "pop", "hit", "special", "stage", "pickup", "heal", "jump", "shield", "voice_ouch", "voice_fall", "voice_alright"]:
		var stream := _audio_asset(sound_name)
		if stream != null:
			sounds[sound_name] = stream
	audio_player.max_polyphony = 8
	audio_player.volume_db = _volume_db(effects_volume)


func _load_music() -> void:
	var track := "music"
	if game_state == "title":
		track = "title_music"
	elif game_state == "celebrating" or (game_state == "name_entry" and run_won):
		track = "happy_music"
	elif game_state == "name_entry":
		track = "sad_music"
	var music_stream := _audio_asset(track)
	if music_stream == null and game_state == "title":
		music_stream = _audio_asset("music")
	if music_stream == null:
		music_player.stop()
		music_player.stream = null
		return
	if music_player.playing and music_player.stream == music_stream:
		return
	var should_loop := track == "music" or track == "title_music"
	if music_stream is AudioStreamWAV:
		music_stream.loop_mode = AudioStreamWAV.LOOP_FORWARD if should_loop else AudioStreamWAV.LOOP_DISABLED
		music_stream.loop_begin = 0
		music_stream.loop_end = roundi(music_stream.get_length() * music_stream.mix_rate)
	elif music_stream is AudioStreamMP3 or music_stream is AudioStreamOggVorbis:
		music_stream.loop = should_loop
	music_player.stream = music_stream
	music_player.volume_db = _volume_db(music_volume) - (5.0 if music_player.stream is AudioStreamMP3 else 0.0)
	music_player.play()
	music_player.stream_paused = not music_enabled


func _character_size(texture: Texture2D, height: float = PLAYER_DRAW_SIZE.y) -> Vector2:
	return Vector2(height * texture.get_width() / float(texture.get_height()), height)


func _volume_db(volume: float) -> float:
	return linear_to_db(maxf(volume, 0.0001))


func _load_input_mappings() -> void:
	var mappings: Dictionary = SaveGame.get_input_mappings()
	for action_name in REMAPPABLE_ACTIONS:
		if not mappings.has(action_name) or not InputMap.has_action(action_name):
			continue
		var keycode := int(mappings[action_name])
		if keycode <= 0:
			continue
		var binding := InputEventKey.new()
		binding.physical_keycode = keycode
		InputMap.action_erase_events(action_name)
		InputMap.action_add_event(action_name, binding)


func _play_sound(sound_name: String) -> void:
	if not sounds.has(sound_name):
		return
	var voices: Array = active_sfx.get(sound_name, [])
	voices = voices.filter(func(item): return is_instance_valid(item) and not item.is_queued_for_deletion())
	var limit := 1 if sound_name.begins_with("voice_") else 3
	if voices.size() >= limit:
		var oldest: AudioStreamPlayer = voices.pop_front()
		oldest.stop()
		oldest.queue_free()
	var voice := AudioStreamPlayer.new()
	voice.stream = sounds[sound_name]
	var trim_db := 0.0
	if voice.stream is AudioStreamMP3:
		trim_db = float({"voice_ouch": 14.0, "voice_fall": 11.0, "heal": 8.0, "special": -3.0, "stage": -2.0}.get(sound_name, 0.0))
	voice.volume_db = _volume_db(effects_volume) + trim_db
	add_child(voice)
	voice.finished.connect(voice.queue_free)
	voices.append(voice)
	active_sfx[sound_name] = voices
	voice.play()


func _initialize_background() -> void:
	for index in range(90):
		background_stars.append(Vector2(rng.randf_range(0.0, 1280.0), rng.randf_range(0.0, 720.0)))
	for index in range(70):
		weather_particles.append({
			"position": Vector2(rng.randf_range(0.0, 1280.0), rng.randf_range(0.0, 720.0)),
			"speed": rng.randf_range(80.0, 260.0),
			"drift": rng.randf_range(-28.0, 28.0)
		})


func _stage_speed() -> float:
	# Speed grows evenly; squared gravity preserves bounce height at each pace.
	return 0.55 + float(clampi(stage, 1, MAX_STAGE) - 1) * 0.10


func _start_stage(stage_number: int) -> void:
	stage = clampi(stage_number, 1, MAX_STAGE)
	hearts = HEARTS_PER_STAGE
	hurt_time = 0.0
	faint_time = 0.0
	special_ready = true
	stage_score = 0
	player_position = Vector2(640.0, PLAYER_GROUND_CENTER_Y)
	previous_player_position = player_position
	player_movement = Vector2.ZERO
	player_velocity_x = 0.0
	player_walk_phase = 0.0
	player_vertical_speed = 0.0
	player_is_airborne = false
	player_is_sitting = false
	player_landing_time = 0.0
	player_shot_kick = 0.0
	player_fire_pose_time = 0.0
	player_facing = 1.0
	projectiles.clear()
	pop_effects.clear()
	pickups.clear()
	roof_ready = false
	weapon_time = 0.0
	weapon_kind = 0
	notice_time = 0.0
	power_time = 0.0
	shield_time = 0.0
	pops_since_drop = 0
	invulnerable_time = 1.25
	balls.clear()
	hazards.clear()
	steel_timer = 4.0
	bomb_timer = 8.0
	var ball_count := 4 + int((stage - 1) * 0.8)
	for index in range(ball_count):
		var radius := rng.randf_range(28.0 + stage * 1.2, 32.0 + stage * 1.2)
		var position := Vector2(
			FIELD.position.x + FIELD.size.x * float(index + 1) / float(ball_count + 1),
			rng.randf_range(FIELD.position.y + radius, FIELD.position.y + FIELD.size.y * 0.48)
		)
		var direction := Vector2(rng.randf_range(-1.0, 1.0), rng.randf_range(-0.7, 0.7)).normalized()
		if direction.length_squared() < 0.1:
			direction = Vector2.RIGHT
		var speed := 160.0 * _stage_speed()
		balls.append({"position": position, "velocity": Vector2((-1.0 if direction.x < 0 else 1.0) * speed, 0.0), "radius": radius, "color": rng.randi_range(0, 4)})
	stage_banner_time = 1.7
	lightning_time = 0.0
	game_state = "playing"
	run_won = false
	_load_music()
	queue_redraw()


func _process(delta: float) -> void:
	if game_state == "title":
		title_time += delta
	elif game_state == "name_entry" and not paused:
		outcome_time += delta
	queue_redraw()


func _render_fraction() -> float:
	return 1.0 if paused or settings_open or game_state != "playing" else Engine.get_physics_interpolation_fraction()


func _physics_process(delta: float) -> void:
	if settings_open or paused:
		queue_redraw()
		return
	if game_state == "fainting":
		if not faint_grounded:
			faint_velocity += PLAYER_GRAVITY * delta
			faint_origin.y += faint_velocity * delta
			if faint_origin.y >= PLAYER_GROUND_CENTER_Y:
				faint_origin.y = PLAYER_GROUND_CENTER_Y
				faint_velocity = 0.0
				faint_grounded = true
				_play_sound("hit")
		else:
			faint_time += delta
		if faint_time >= 1.8:
			_finish_faint()
		queue_redraw()
		return
	if game_state == "celebrating":
		celebration_time += delta
		queue_redraw()
		return
	if game_state != "playing":
		queue_redraw()
		return
	hurt_time = maxf(0.0, hurt_time - delta)
	previous_player_position = player_position
	fire_cooldown = maxf(fire_cooldown - delta, 0.0)
	invulnerable_time = maxf(invulnerable_time - delta, 0.0)
	stage_banner_time = maxf(stage_banner_time - delta, 0.0)
	lightning_time = maxf(lightning_time - delta, 0.0)
	for effect in pop_effects:
		effect["life"] -= delta
	pop_effects = pop_effects.filter(func(effect): return effect["life"] > 0.0)
	_update_weather(delta)
	weapon_time = maxf(0.0, weapon_time - delta)
	notice_time = maxf(0.0, notice_time - delta)
	if weapon_time <= 0:
		weapon_kind = 0
	power_time = maxf(0.0, power_time - delta)
	shield_time = maxf(0.0, shield_time - delta)
	_update_pickups(delta)
	_update_player(delta)
	_update_projectiles(delta)
	_update_balls(delta)
	if game_state == "playing" and not balls.is_empty():
		_update_hazards(delta)
	if Input.is_action_just_pressed("special") or _gamepad_special_just_pressed():
		_use_special()
	_check_stage_clear()
	queue_redraw()


func _update_weather(delta: float) -> void:
	var weather := _weather_for_stage()
	for particle in weather_particles:
		var position: Vector2 = particle["position"]
		position.y += float(particle["speed"]) * delta
		position.x += float(particle["drift"]) * delta
		if position.y > 730.0:
			position.y = -10.0
			position.x = rng.randf_range(0.0, 1280.0)
		if position.x < -10.0:
			position.x = 1290.0
		elif position.x > 1290.0:
			position.x = -10.0
		particle["position"] = position
	if weather == "storm" and lightning_time <= 0.0 and rng.randf() < delta * 0.12:
		lightning_time = 0.16


func _update_player(delta: float) -> void:
	var horizontal := Input.get_axis("move_left", "move_right")
	var gamepad := _first_gamepad()
	var joy_movement := Vector2.ZERO
	if gamepad >= 0:
		joy_movement = Vector2(Input.get_joy_axis(gamepad, JOY_AXIS_LEFT_X), Input.get_joy_axis(gamepad, JOY_AXIS_LEFT_Y))
	var crouch_pressed := Input.is_action_pressed("move_down") or joy_movement.y > 0.65
	if absf(joy_movement.x) > 0.18:
		horizontal = joy_movement.x
	if crouch_pressed:
		horizontal = 0.0
	var target_velocity := horizontal * PLAYER_SPEED
	if crouch_pressed:
		player_velocity_x = 0.0
	else:
		var acceleration := PLAYER_ACCELERATION if absf(target_velocity) > 0.01 else PLAYER_FRICTION
		player_velocity_x = move_toward(player_velocity_x, target_velocity, acceleration * delta)
	if absf(player_velocity_x) < 1.0:
		player_velocity_x = 0.0
	player_movement = Vector2(player_velocity_x / PLAYER_SPEED, 0.0)
	if absf(player_velocity_x) > 14.0:
		player_facing = signf(player_velocity_x)
		if not player_is_airborne and not player_is_sitting:
			var walk_rate := 8.0 + 2.0 * clampf(absf(player_velocity_x) / PLAYER_SPEED, 0.0, 1.0)
			player_walk_phase = fposmod(player_walk_phase + delta * walk_rate, TAU)
	player_position.x += player_velocity_x * delta
	player_position.x = clampf(player_position.x, FIELD.position.x + PLAYER_RADIUS, FIELD.end.x - PLAYER_RADIUS)
	if is_equal_approx(player_position.x, FIELD.position.x + PLAYER_RADIUS) or is_equal_approx(player_position.x, FIELD.end.x - PLAYER_RADIUS):
		player_velocity_x = 0.0
		player_movement.x = 0.0
	var grounded := not player_is_airborne
	var jump_from_pad := joy_movement.y < -0.65
	var jump_from_pad_just_pressed := jump_from_pad and not previous_gamepad_jump
	previous_gamepad_jump = jump_from_pad
	var jump_pressed := Input.is_action_just_pressed("move_up") or jump_from_pad_just_pressed
	if jump_pressed and grounded and not crouch_pressed:
		player_is_airborne = true
		player_is_sitting = false
		player_vertical_speed = -JUMP_SPEED
		_play_sound("jump")
		grounded = false
	player_is_sitting = crouch_pressed and grounded
	if player_is_airborne:
		player_vertical_speed += PLAYER_GRAVITY * delta
		player_position.y += player_vertical_speed * delta
		if player_position.y >= PLAYER_GROUND_CENTER_Y:
			player_position.y = PLAYER_GROUND_CENTER_Y
			player_vertical_speed = 0.0
			player_is_airborne = false
			player_landing_time = 0.16
	else:
		player_position.y = PLAYER_GROUND_CENTER_Y
	player_landing_time = maxf(0.0, player_landing_time - delta)
	player_shot_kick = maxf(0.0, player_shot_kick - delta * 8.0)
	player_fire_pose_time = maxf(0.0, player_fire_pose_time - delta)
	var idle_now := absf(player_velocity_x) <= 14.0 and not player_is_airborne and not player_is_sitting
	if idle_now:
		if idle_glancing:
			idle_glance_time -= delta
			if idle_glance_time <= 0.0:
				idle_glancing = false
				idle_glance_delay = rng.randf_range(2.0, 4.0)
		else:
			idle_glance_delay -= delta
			if idle_glance_delay <= 0.0:
				idle_glancing = true
				idle_glance_time = 0.42
	else:
		idle_glancing = false
		idle_glance_time = 0.0
		idle_glance_delay = rng.randf_range(2.0, 4.0)
	var fire_pressed := Input.is_action_pressed("fire") or Input.is_mouse_button_pressed(MOUSE_BUTTON_LEFT) or _gamepad_fire_pressed()
	if start_fire_blocked and not fire_pressed:
		start_fire_blocked = false
	if fire_pressed and not start_fire_blocked and fire_cooldown <= 0.0 and _moving_rope_count() < (4 if weapon_kind == 5 else (3 if weapon_kind == 4 else (2 if power_time > 0.0 else 1))):
		_fire_projectile()


func _launcher_origin() -> Vector2:
	return player_position + Vector2(-20.0, -51.0 if not player_is_sitting else -14.0)


func _moving_rope_count() -> int:
	var count := 0
	for rope in projectiles:
		var attached := float(rope["tip_y"]) <= FIELD.position.y and (bool(rope.get("roof", false)) or int(rope.get("wire", 0)) == 1)
		if not attached:
			count += 1
	return count


func _fire_projectile() -> void:
	var muzzle := _launcher_origin()
	var offsets := [-22.0, 0.0, 22.0] if weapon_kind == 4 else [0.0]
	for offset in offsets:
		projectiles.append({"x": clampf(muzzle.x + offset, FIELD.position.x + 8, FIELD.end.x - 8), "base_y": muzzle.y, "tip_y": muzzle.y, "wire": wire_type, "hold": 1.4, "hits": (4 if weapon_kind == 6 else (2 if wire_type == 2 else 1)) + (1 if power_time > 0.0 else 0), "hit_delay": 0.0, "roof": roof_ready and offset == 0.0, "roof_left": 5.0, "weapon": weapon_kind})
	roof_ready = false
	fire_cooldown = 0.14 if weapon_kind == 5 else (0.7 if weapon_kind == 4 else [0.16, 0.45, 0.65][wire_type])
	player_shot_kick = 1.0
	player_fire_pose_time = 0.36
	_play_sound("fire")


func _update_projectiles(delta: float) -> void:
	for index in range(projectiles.size() - 1, -1, -1):
		var projectile: Dictionary = projectiles[index]
		var roof := bool(projectile.get("roof", false))
		if roof and float(projectile["tip_y"]) <= FIELD.position.y:
			projectile["roof_left"] = float(projectile.get("roof_left", 5.0)) - delta
			if float(projectile["roof_left"]) <= 0.0:
				projectiles.remove_at(index)
				continue
		projectile["previous_tip"] = projectile["tip_y"]
		var x := float(projectile["x"])
		var base_y := float(projectile["base_y"])
		var kind := int(projectile.get("wire", 0))
		var speed: float = [850.0, 560.0, 680.0][kind]
		var weapon := int(projectile.get("weapon", 0))
		if weapon == 5:
			speed = 1100
		elif weapon == 6:
			speed = 480
		projectile["hit_delay"] = maxf(0.0, float(projectile.get("hit_delay", 0.0)) - delta)
		var tip_y := maxf(FIELD.position.y, float(projectile["tip_y"]) - speed * delta)
		projectile["tip_y"] = tip_y
		if roof and tip_y <= FIELD.position.y:
			base_y = PLAYER_FOOT_Y
			projectile["base_y"] = base_y
		var hit_index := -1
		for ball_index in range(balls.size()):
			var ball: Dictionary = balls[ball_index]
			var ball_position: Vector2 = ball["position"]
			var ball_radius: float = ball["radius"]
			var nearest := Vector2(x, clampf(ball_position.y, tip_y, base_y))
			if float(projectile["hit_delay"]) <= 0.0 and ball_position.distance_squared_to(nearest) <= pow(ball_radius + (8.0 if weapon == 6 else 3.5), 2):
				hit_index = ball_index
				break
		if hit_index >= 0:
			_pop_ball(hit_index)
			if roof:
				projectile["hit_delay"] = 0.08
				continue
			projectile["hits"] = int(projectile.get("hits", 1)) - 1
			if int(projectile["hits"]) <= 0:
				projectiles.remove_at(index)
			else:
				projectile["hit_delay"] = 0.12
		elif tip_y <= FIELD.position.y:
			if roof:
				continue
			if kind == 1:
				projectile["hold"] = float(projectile.get("hold", 1.4)) - delta
				if float(projectile["hold"]) <= 0.0:
					projectiles.remove_at(index)
			else:
				projectiles.remove_at(index)
		else:
			projectiles[index] = projectile


func _update_balls(delta: float) -> void:
	for index in range(balls.size()):
		var ball: Dictionary = balls[index]
		ball["previous_position"] = ball["position"]
		var velocity: Vector2 = ball["velocity"]
		velocity.y += BALL_GRAVITY * pow(_stage_speed(), 2) * delta
		var position: Vector2 = ball["position"] + velocity * delta
		var radius: float = ball["radius"]
		if position.x < FIELD.position.x + radius:
			velocity.x = absf(velocity.x)
		elif position.x > FIELD.end.x - radius:
			velocity.x = -absf(velocity.x)
		position.x = clampf(position.x, FIELD.position.x + radius, FIELD.end.x - radius)
		if position.y > FIELD.end.y - radius:
			position.y = FIELD.end.y - radius
			velocity.y = -sqrt(2.0 * BALL_GRAVITY * (85.0 + radius * 5.0)) * _stage_speed()
		elif position.y < FIELD.position.y + radius:
			position.y = FIELD.position.y + radius
			velocity.y = absf(velocity.y)
		ball["position"] = position
		ball["velocity"] = velocity
		balls[index] = ball
		var player_hit_radius := PLAYER_RADIUS * (0.58 if player_is_sitting else (0.82 if player_is_airborne else 1.0))
		if invulnerable_time <= 0.0 and position.distance_to(player_position) < radius + player_hit_radius:
			_take_hit()
			return
	_resolve_ball_contacts()


func _resolve_ball_contacts() -> void:
	for i in range(balls.size()):
		for j in range(i + 1, balls.size()):
			var a := balls[i]
			var b := balls[j]
			var offset: Vector2 = b["position"] - a["position"]
			var total: float = a["radius"] + b["radius"]
			var distance := offset.length()
			if distance >= total:
				continue
			var normal := offset / distance if distance > 0.001 else Vector2.RIGHT
			var overlap := total - distance
			a["position"] -= normal * overlap * 0.5
			b["position"] += normal * overlap * 0.5
			var approach: float = (Vector2(b["velocity"]) - Vector2(a["velocity"])).dot(normal)
			if approach < 0.0:
				a["velocity"] += normal * approach
				b["velocity"] -= normal * approach
			for ball in [a, b]:
				var radius: float = ball["radius"]
				var position: Vector2 = ball["position"]
				position.x = clampf(position.x, FIELD.position.x + radius, FIELD.end.x - radius)
				position.y = clampf(position.y, FIELD.position.y + radius, FIELD.end.y - radius)
				ball["position"] = position


func _spawn_steel() -> void:
	var direction := 1.0 if rng.randf() < 0.5 else -1.0
	hazards.append({"kind": "steel", "position": Vector2(-34 if direction > 0 else 1314, PLAYER_FOOT_Y - 53), "velocity": Vector2(direction * 220.0 * _stage_speed(), 0), "radius": 28.0, "warning": 1.25})


func _spawn_bomb() -> void:
	hazards.append({"kind": "bomb", "position": Vector2(rng.randf_range(180, 1100), 155), "velocity": Vector2(rng.randf_range(-95,95), 30) * _stage_speed(), "radius": 25.0, "fuse": 3.5})


func _explode_bomb(index: int) -> void:
	var center: Vector2 = hazards[index]["position"]
	hazards.remove_at(index)
	pop_effects.append({"position": center, "radius": 38.0, "life": 0.3})
	_play_sound("special")
	for i in range(8):
		var angle := TAU * i / 8.0
		hazards.append({"kind": "fire", "position": center, "previous_position": center, "velocity": Vector2(cos(angle), sin(angle)) * 200.0 * _stage_speed(), "radius": 11.0, "life": 2.8})


func _update_hazards(delta: float) -> void:
	if stage >= 3:
		steel_timer -= delta
		if steel_timer <= 0:
			_spawn_steel()
			steel_timer = maxf(6.0, 12.0 - stage * 0.45)
	if stage >= 5:
		bomb_timer -= delta
		if bomb_timer <= 0:
			_spawn_bomb()
			bomb_timer = maxf(7.0, 14.0 - stage * 0.6)
	for i in range(hazards.size() - 1, -1, -1):
		var hazard := hazards[i]
		var kind: String = hazard["kind"]
		if kind == "steel" and float(hazard["warning"]) > 0:
			hazard["warning"] = maxf(0, float(hazard["warning"]) - delta)
			continue
		hazard["previous_position"] = hazard["position"]
		var velocity: Vector2 = hazard["velocity"]
		if kind == "bomb":
			velocity.y += 460 * pow(_stage_speed(), 2) * delta
		var position: Vector2 = hazard["position"] + velocity * delta
		if kind == "bomb":
			if position.y > PLAYER_FOOT_Y - 25:
				position.y = PLAYER_FOOT_Y - 25
				velocity.y = -210 * _stage_speed()
			if position.x < 65 or position.x > 1215:
				velocity.x *= -1
				position.x = clampf(position.x, 65, 1215)
			hazard["fuse"] -= delta
		if kind == "fire":
			hazard["life"] -= delta
		hazard["position"] = position
		hazard["velocity"] = velocity
		if kind == "bomb" and float(hazard["fuse"]) <= 0:
			_explode_bomb(i)
			continue
		if (kind == "fire" and (float(hazard["life"]) <= 0 or position.y > PLAYER_FOOT_Y + 20 or position.y < 80)) or position.x < -70 or position.x > 1350:
			hazards.remove_at(i)
			continue
		var player_radius := PLAYER_RADIUS * (0.58 if player_is_sitting else 1.0)
		if invulnerable_time <= 0 and position.distance_to(player_position) < float(hazard["radius"]) + player_radius:
			_take_hit()
			return


func _draw_hazards() -> void:
	for hazard in hazards:
		var position: Vector2 = Vector2(hazard.get("previous_position", hazard["position"])).lerp(hazard["position"], _render_fraction())
		var kind: String = hazard["kind"]
		if kind == "steel" and float(hazard["warning"]) > 0:
			var x := 60.0 if hazard["velocity"].x > 0 else 1040.0
			_hud_text("STEEL INCOMING!", Vector2(x, position.y - 45), 15, Color("ffd076"))
			draw_line(Vector2(40, position.y), Vector2(1240, position.y), Color(1,0.7,0.3,0.22), 2)
			continue
		if kind == "steel":
			draw_circle(position + Vector2(3,5), 29, Color(0,0,0,0.4))
			draw_circle(position, 28, Color("29323e"))
			for layer in range(12):
				var t := layer / 11.0
				draw_circle(position + Vector2(-5,-6) * t, 27 - t * 16, Color("566473").lerp(Color("e5f2f6"),t))
			draw_arc(position, 23, 0.1, 2.8, 28, Color("aabcc4"), 2, true)
			for angle in [0.5,2.3,4.1]:
				var rivet := position + Vector2(cos(angle),sin(angle))*21
				draw_circle(rivet,3,Color("283440"))
				draw_circle(rivet-Vector2(0.5,1),1.4,Color("d9e3e8"))
		elif kind == "bomb":
			draw_circle(position + Vector2(2,4),26,Color(0,0,0,0.4))
			draw_circle(position,25,Color("10131a"))
			draw_circle(position+Vector2(-5,-6),18,Color("30343e"))
			draw_arc(position,19,-2.8,-1.6,15,Color("798394"),3,true)
			draw_rect(Rect2(position+Vector2(-6,-29),Vector2(12,7)),Color("737b84"))
			draw_line(position+Vector2(0,-28),position+Vector2(9,-38),Color("cfb276"),3,true)
			var spark := position+Vector2(9,-38)
			draw_circle(spark,5,Color("ff7b32"))
			draw_circle(spark,2.5,Color("fff5b3"))
			_hud_text("%.1f" % float(hazard["fuse"]),position+Vector2(-13,6),13,Color("ffbd81"))
		else:
			var tail: Vector2 = -Vector2(hazard["velocity"]).normalized()*23
			draw_line(position+tail,position,Color(1,0.23,0.02,0.5),15,true)
			draw_circle(position,12,Color("f55020"))
			draw_circle(position-Vector2(2,2),8,Color("ffbb42"))
			draw_circle(position-Vector2(3,3),4,Color("fff2a0"))


func _pop_ball(index: int) -> void:
	var ball: Dictionary = balls[index]
	var radius: float = ball["radius"]
	var position: Vector2 = ball["position"]
	var velocity: Vector2 = ball["velocity"]
	var color_index: int = ball["color"]
	pops_since_drop += 1
	if pops_since_drop >= 3:
		pops_since_drop = 0
		pickups.append({"position": position, "kind": rng.randi_range(0, 6), "life": 12.0})
	pop_effects.append({"position": position, "radius": radius, "life": 0.3})
	balls.remove_at(index)
	stage_score += 100
	score += 100
	_play_sound("pop")
	if radius > 16.32:
		for sign_value in [-1.0, 1.0]:
			var split_velocity := Vector2(sign_value * 190.0, -280.0) * _stage_speed()
			balls.append({
				"position": position + Vector2(sign_value * radius * 0.35, 0.0),
				"velocity": split_velocity,
				"radius": radius * 0.68,
				"color": (color_index + 1) % 5
			})


func _collect_pickup(kind: int) -> void:
	pickup_notice = ["HEALTH", "WEAPON BOOST", "SHIELD", "ROOF TRAP", "TRIPLE HARPOON", "RAPID REEL", "HEAVY SPEAR"][kind]
	notice_time = 2.5
	if kind >= 4:
		weapon_kind = kind
		weapon_time = 12.0
		_play_sound("pickup")
		return
	match kind:
		0:
			if hearts < HEARTS_PER_STAGE:
				hearts += 1
			else:
				score += 150
				stage_score += 150
			_play_sound("heal")
		1:
			power_time = 10.0
			_play_sound("pickup")
		3:
			roof_ready = true
			_play_sound("pickup")
		2:
			shield_time = 12.0
			_play_sound("shield")


func _update_pickups(delta: float) -> void:
	for i in range(pickups.size() - 1, -1, -1):
		var item := pickups[i]
		item["life"] -= delta
		var position: Vector2 = item["position"]
		position.y = minf(PLAYER_FOOT_Y - 18.0, position.y + 145.0 * delta)
		item["position"] = position
		var feet := player_position + Vector2(0, PLAYER_HALF_HEIGHT)
		if absf(position.x - feet.x) < 32 and position.y >= feet.y - PLAYER_DRAW_SIZE.y and position.y <= feet.y + 8:
			_collect_pickup(int(item["kind"]))
			pickups.remove_at(i)
		elif float(item["life"]) <= 0:
			pickups.remove_at(i)


func _draw_pickups() -> void:
	for item in pickups:
		_draw_pickup_icon(item["position"], int(item["kind"]))


func _draw_pickup_icon(position: Vector2, kind: int) -> void:
	var colors := [Color("ff6984"), Color("ffd36c"), Color("70ddff"), Color("b4ef6a"), Color("be91ff"), Color("ffad51"), Color("a9d9ed")]
	var tint: Color = colors[kind]
	draw_circle(position + Vector2(2, 4), 23, Color(0, 0, 0, 0.4))
	draw_circle(position, 23, Color("674521"))
	draw_circle(position, 21, Color("e9c980"))
	draw_circle(position, 18.5, Color("182d43"))
	draw_arc(position, 21, -2.8, -0.4, 20, Color("fff2bd"), 2, true)
	draw_set_transform(position)
	match kind:
		0:
			_draw_heart(Vector2(0, 0), true)
			draw_rect(Rect2(8, 3, 3, 10), Color("fff4d8"))
			draw_rect(Rect2(4.5, 6.5, 10, 3), Color("fff4d8"))
		1, 5:
			draw_circle(Vector2.ZERO, 12, Color("79512c"))
			draw_arc(Vector2.ZERO, 11, 0, TAU, 24, Color("e2bd78"), 3, true)
			draw_colored_polygon(PackedVector2Array([Vector2(3,-16),Vector2(-9,2),Vector2(-1,2),Vector2(-4,16),Vector2(10,-3),Vector2(2,-3)]), tint)
			draw_line(Vector2(3,-12),Vector2(-5,0), Color("fff5ce"),2,true)
			if kind == 5:
				draw_circle(Vector2.ZERO, 12, Color("e1b77b"))
				draw_circle(Vector2.ZERO, 9, Color("71899a"))
				for angle in [0.0, PI * 0.5, PI, PI * 1.5]:
					draw_circle(Vector2(cos(angle), sin(angle)) * 5.5, 2.5, Color("21354a"))
				draw_circle(Vector2.ZERO, 2, Color("ffe4a3"))
				draw_arc(Vector2.ZERO, 12, -2.8, -0.5, 12, Color("fff0ca"), 2, true)
				for y in [-8,0,8]:
					draw_line(Vector2(-17,y),Vector2(-12,y),Color("ffae65"),2,true)
		2:
			var shield := PackedVector2Array([Vector2(-13,-11),Vector2(0,-15),Vector2(13,-11),Vector2(11,5),Vector2(0,16),Vector2(-11,5)])
			draw_colored_polygon(shield, Color("c4efff"))
			draw_colored_polygon(PackedVector2Array([Vector2(-9,-8),Vector2(0,-11),Vector2(9,-8),Vector2(7,4),Vector2(0,11),Vector2(-7,4)]),Color("287dba"))
			draw_line(Vector2(0,-10),Vector2(0,10),Color("7ce6ff"),3,true)
		3:
			draw_line(Vector2(-14,-13),Vector2(14,-13),Color("d7e8e3"),5,true)
			draw_line(Vector2(0,-12),Vector2(0,14),Color("cfb175"),5,true)
			for y in range(-10,14,5):
				draw_line(Vector2(-2,y),Vector2(2,y+3),Color("6b512f"),1.5,true)
			draw_rect(Rect2(-8,11,16,4),tint)
		4, 6:
			var offsets := [-10,0,10] if kind == 4 else [0]
			for x in offsets:
				draw_line(Vector2(x,13),Vector2(x,-8),Color("7d5831"),5,true)
				draw_line(Vector2(x-1,12),Vector2(x-1,-8),Color("e2c48e"),2,true)
				var width := 5 if kind == 4 else 10
				draw_colored_polygon(PackedVector2Array([Vector2(x,-16),Vector2(x-width,-4),Vector2(x,-7),Vector2(x+width,-4)]),tint)
				draw_line(Vector2(x,-14),Vector2(x-width+2,-5),Color.WHITE,1.5,true)
	draw_set_transform(Vector2.ZERO)


func _take_hit() -> void:
	if game_state != "playing":
		return
	if shield_time > 0.0:
		shield_time = 0.0
		invulnerable_time = 1.0
		_play_sound("shield")
		return
	hearts -= 1
	hurt_time = HURT_DURATION
	player_fire_pose_time = 0.0
	invulnerable_time = 1.25
	_play_sound("hit")
	if hearts > 0:
		_play_sound("voice_ouch")
		return
	_play_sound("voice_fall")
	game_state = "fainting"
	faint_time = 0.0
	faint_origin = player_position
	faint_grounded = player_position.y >= PLAYER_GROUND_CENTER_Y - 0.5
	faint_velocity = player_vertical_speed if not faint_grounded else 0.0
	player_velocity_x = 0.0
	projectiles.clear()


func _finish_faint() -> void:
	if game_state != "fainting":
		return
	if continues > 0:
		continues -= 1
		score = maxi(score - stage_score, 0)
		_start_stage(stage)
		return
	_finish_run("No continues left")


func _use_special() -> void:
	if not special_ready or game_state != "playing" or paused or settings_open:
		return
	special_ready = false
	balls.clear()
	projectiles.clear()
	stage_score += 250
	score += 250
	_play_sound("special")
	_check_stage_clear()


func _check_stage_clear() -> void:
	if game_state != "playing" or not balls.is_empty():
		return
	hazards.clear()
	stage_bonus = 500 + stage * 100
	score += stage_bonus
	projectiles.clear()
	game_state = "celebrating"
	celebration_time = 0.0
	_load_music()
	if not sounds.has("voice_alright") or _audio_asset("happy_music") == null:
		_play_sound("stage")
	_play_sound("voice_alright")


func _continue_stage() -> void:
	if game_state != "celebrating" or celebration_time < 1.5:
		return
	if stage == MAX_STAGE:
		_finish_run("All 10 stages cleared!", true)
	else:
		_start_stage(stage + 1)
		SaveGame.save_progress(stage, {"score": score, "continues": continues})


func _finish_run(message: String, won: bool = false) -> void:
	run_won = won
	outcome_time = 0.0
	finish_message = message
	name_entry = ""
	game_state = "name_entry"
	projectiles.clear()
	if not won:
		_load_music()


func _submit_score() -> void:
	var player_name := name_entry.strip_edges().to_upper()
	if player_name.is_empty():
		player_name = "PLAYER"
	SaveGame.add_high_score(player_name.left(12), score)
	leaderboard_entries = SaveGame.get_high_scores()
	game_state = "leaderboard"


func _unhandled_input(event: InputEvent) -> void:
	if event is InputEventKey and event.pressed and not event.echo:
		var key_event := event as InputEventKey
		if game_state == "celebrating" and key_event.keycode == KEY_ENTER and not paused:
			_continue_stage()
			return
		if key_event.keycode == KEY_M and remapping_action.is_empty() and game_state != "name_entry":
			_toggle_music()
			return
		if game_state == "playing" and not settings_open and not paused:
			if key_event.keycode >= KEY_1 and key_event.keycode <= KEY_3:
				wire_type = key_event.keycode - KEY_1
			elif key_event.keycode == KEY_Q:
				wire_type = (wire_type + 1) % 3
		if key_event.keycode == KEY_ESCAPE and game_state in ["playing", "celebrating", "fainting"] and not settings_open:
			paused = not paused
			queue_redraw()
			return
		if key_event.keycode == KEY_F1 and (game_state == "playing" or game_state == "title"):
			settings_open = not settings_open
			remapping_action = ""
			get_viewport().set_input_as_handled()
			queue_redraw()
			return
		if settings_open:
			_handle_settings_key(key_event)
			get_viewport().set_input_as_handled()
			queue_redraw()
			return
		if game_state == "name_entry":
			if key_event.keycode == KEY_ENTER or key_event.keycode == KEY_KP_ENTER:
				_submit_score()
			elif key_event.keycode == KEY_BACKSPACE:
				name_entry = name_entry.left(maxi(name_entry.length() - 1, 0))
			elif key_event.unicode >= 32 and key_event.unicode <= 126 and name_entry.length() < 12:
				name_entry += char(key_event.unicode).to_upper()
			queue_redraw()
			return
		if game_state == "title":
			if key_event.keycode == KEY_H or key_event.keycode == KEY_TAB:
				leaderboard_entries = SaveGame.get_high_scores()
				game_state = "leaderboard"
			elif key_event.keycode == KEY_F11:
				_toggle_fullscreen()
			else:
				_begin_game()
			get_viewport().set_input_as_handled()
			queue_redraw()
			return
		if game_state == "leaderboard" and key_event.keycode == KEY_ENTER:
			_reset_run()
			return
		if key_event.keycode == KEY_F11:
			_toggle_fullscreen()
	if event is InputEventMouseButton and event.pressed:
		if event.button_index == MOUSE_BUTTON_LEFT and game_state == "celebrating" and not paused:
			_continue_stage()
			return
		if event.button_index == MOUSE_BUTTON_LEFT and Rect2(1100, 65, 150, 30).has_point(event.position) and game_state == "playing" and not settings_open:
			paused = not paused
			return
		if event.button_index == MOUSE_BUTTON_WHEEL_UP and game_state == "playing" and not settings_open and not paused:
			wire_type = (wire_type + 1) % 3
		if settings_open and event.button_index == MOUSE_BUTTON_LEFT:
			_handle_settings_click(event.position)
			get_viewport().set_input_as_handled()
			queue_redraw()
			return
		if game_state == "name_entry" and event.button_index == MOUSE_BUTTON_LEFT:
			_submit_score()
		elif game_state == "title" and event.button_index == MOUSE_BUTTON_LEFT:
			if Rect2(64, 611, 160, 44).has_point(event.position):
				settings_open = true
			elif Rect2(238, 611, 160, 44).has_point(event.position):
				leaderboard_entries = SaveGame.get_high_scores()
				game_state = "leaderboard"
			elif Rect2(412, 611, 190, 44).has_point(event.position):
				_toggle_music()
			else:
				_begin_game()
		elif game_state == "leaderboard" and event.button_index == MOUSE_BUTTON_LEFT:
			_reset_run()
		elif game_state == "playing" and not paused and not settings_open and event.button_index == MOUSE_BUTTON_RIGHT:
			_use_special()
	if event is InputEventJoypadButton and event.pressed:
		if event.button_index == JOY_BUTTON_START and game_state in ["playing", "celebrating", "fainting"] and not settings_open:
			paused = not paused
			return
		if event.button_index == JOY_BUTTON_A and game_state == "celebrating" and not paused:
			_continue_stage()
			return
		if event.button_index == JOY_BUTTON_RIGHT_SHOULDER and game_state == "playing" and not settings_open and not paused:
			wire_type = (wire_type + 1) % 3
		if settings_open and (event.button_index == JOY_BUTTON_B or event.button_index == JOY_BUTTON_START):
			settings_open = false
			SaveGame.save_audio_settings(music_volume, effects_volume)
			get_viewport().set_input_as_handled()
			queue_redraw()
			return
		if settings_open:
			return
		if game_state == "title":
			_begin_game()
		elif event.button_index == JOY_BUTTON_START and game_state == "leaderboard":
			_reset_run()
		elif event.button_index == JOY_BUTTON_Y and game_state == "playing":
			_use_special()
	queue_redraw()


func _handle_settings_key(key_event: InputEventKey) -> void:
	if not remapping_action.is_empty():
		if key_event.keycode == KEY_ESCAPE:
			remapping_action = ""
		else:
			var new_binding := InputEventKey.new()
			new_binding.physical_keycode = key_event.physical_keycode
			InputMap.action_erase_events(remapping_action)
			InputMap.action_add_event(remapping_action, new_binding)
			SaveGame.save_input_mapping(remapping_action, key_event.physical_keycode)
			remapping_action = ""
		return
	if key_event.keycode == KEY_ESCAPE or key_event.keycode == KEY_F1:
		settings_open = false
		SaveGame.save_audio_settings(music_volume, effects_volume)
	elif key_event.keycode == KEY_UP:
		settings_cursor = posmod(settings_cursor - 1, 2 + REMAPPABLE_ACTIONS.size())
	elif key_event.keycode == KEY_DOWN:
		settings_cursor = posmod(settings_cursor + 1, 2 + REMAPPABLE_ACTIONS.size())
	elif key_event.keycode == KEY_LEFT or key_event.keycode == KEY_RIGHT:
		var amount := -0.05 if key_event.keycode == KEY_LEFT else 0.05
		if settings_cursor == 0:
			music_volume = clampf(music_volume + amount, 0.0, 1.0)
			music_player.volume_db = _volume_db(music_volume) - (5.0 if music_player.stream is AudioStreamMP3 else 0.0)
		elif settings_cursor == 1:
			effects_volume = clampf(effects_volume + amount, 0.0, 1.0)
			audio_player.volume_db = _volume_db(effects_volume)
		SaveGame.save_audio_settings(music_volume, effects_volume)
	elif key_event.keycode == KEY_ENTER or key_event.keycode == KEY_KP_ENTER:
		if settings_cursor >= 2:
			remapping_action = REMAPPABLE_ACTIONS[settings_cursor - 2]


func _gamepad_fire_pressed() -> bool:
	var gamepad := _first_gamepad()
	return gamepad >= 0 and (Input.is_joy_button_pressed(gamepad, JOY_BUTTON_A) or Input.is_joy_button_pressed(gamepad, JOY_BUTTON_X))


func _gamepad_special_just_pressed() -> bool:
	var gamepad := _first_gamepad()
	return gamepad >= 0 and Input.is_joy_button_pressed(gamepad, JOY_BUTTON_Y)


func _first_gamepad() -> int:
	var connected := Input.get_connected_joypads()
	return int(connected[0]) if not connected.is_empty() else -1


func _toggle_music() -> void:
	music_enabled = not music_enabled
	music_player.stream_paused = not music_enabled
	SaveGame.save_music_enabled(music_enabled)
	queue_redraw()


func _handle_settings_click(position: Vector2) -> void:
	if Rect2(365, 183, 550, 35).has_point(position):
		_toggle_music()
		return
	var row := int(floor((position.y - 230.0) / 43.0))
	if row < 0 or row >= 2 + REMAPPABLE_ACTIONS.size():
		return
	settings_cursor = row
	if row == 0 or row == 1:
		var value := clampf((position.x - 620.0) / 245.0, 0.0, 1.0)
		if row == 0:
			music_volume = value
			music_player.volume_db = _volume_db(music_volume) - (5.0 if music_player.stream is AudioStreamMP3 else 0.0)
		else:
			effects_volume = value
			audio_player.volume_db = _volume_db(effects_volume)
		SaveGame.save_audio_settings(music_volume, effects_volume)
	else:
		remapping_action = REMAPPABLE_ACTIONS[row - 2]


func _reset_run() -> void:
	paused = false
	continues = STARTING_CONTINUES
	score = 0
	stage_score = 0
	_start_stage(1)


func _toggle_fullscreen() -> void:
	var current_mode := DisplayServer.window_get_mode()
	var next_mode := DisplayServer.WINDOW_MODE_WINDOWED if current_mode == DisplayServer.WINDOW_MODE_FULLSCREEN else DisplayServer.WINDOW_MODE_FULLSCREEN
	DisplayServer.window_set_mode(next_mode)


func _weather_for_stage() -> String:
	return ["clear", "clear", "snow", "storm", "rain", "clear", "rain", "clear", "rain", "clear"][stage - 1]


func _is_night() -> bool:
	var cycle := float(Time.get_ticks_msec()) / 1000.0
	return sin(cycle * 0.075 + float(stage)) < -0.1


func _draw() -> void:
	if game_state == "title":
		_draw_title()
		if settings_open:
			_draw_settings()
		return
	_draw_background()
	_draw_weather()
	_draw_field()
	_draw_balls()
	_draw_hazards()
	_draw_pickups()
	_draw_player()
	_draw_projectiles()
	for effect in pop_effects:
		var fade: float = effect["life"] / 0.3
		draw_arc(effect["position"], effect["radius"] + (1.0 - fade) * 24.0, 0, TAU, 32, Color(1, 0.9, 0.5, fade), 3.0, true)
	_draw_hud()
	if stage_banner_time > 0.0 and game_state == "playing":
		_draw_fancy_text("STAGE %02d" % stage, Vector2(0.0, 385.0), 1280.0, 44, Color.WHITE, HORIZONTAL_ALIGNMENT_CENTER)
	if game_state == "name_entry":
		_draw_name_entry()
	elif game_state == "leaderboard":
		_draw_leaderboard()
	elif game_state == "title":
		_draw_title()
	if game_state == "celebrating":
		_draw_celebration()
	if paused:
		draw_rect(Rect2(0, 0, 1280, 720), Color(0.02, 0.04, 0.1, 0.7))
		_draw_fancy_text("PAUSED · ESC TO RESUME", Vector2(0, 360), 1280, 36, Color.WHITE, HORIZONTAL_ALIGNMENT_CENTER)
	if settings_open:
		_draw_settings()


func _draw_background() -> void:
	var night := _is_night()
	if stage_backgrounds.has(stage):
		var background: Texture2D = stage_backgrounds[stage]
		var source_width := float(background.get_width())
		var source_height := source_width * PLAYER_FOOT_Y / 1280.0
		draw_texture_rect_region(background, Rect2(0, 0, 1280, PLAYER_FOOT_Y), Rect2(0, 0, source_width, source_height))
		if night:
			draw_rect(Rect2(0.0, 0.0, 1280.0, 720.0), Color(0.025, 0.045, 0.18, 0.42), true)
	else:
		var palettes: Array = [
			[Color("56c8ed"), Color("a5efff"), Color("69bd7d")],
			[Color("ea8bba"), Color("ffe0b3"), Color("c77a75")],
			[Color("45b9b0"), Color("b4f1d2"), Color("3e897c")],
			[Color("e3795d"), Color("ffd47a"), Color("a34f49")],
			[Color("6989df"), Color("d5c8ff"), Color("5965a3")],
			[Color("4b9d79"), Color("d2e890"), Color("647d45")],
			[Color("d878d2"), Color("ffc3e7"), Color("825893")],
			[Color("3d84bd"), Color("a5def7"), Color("467a9e")],
			[Color("db9c4f"), Color("ffe5a5"), Color("9d6544")],
			[Color("57627f"), Color("c4cde2"), Color("555b78")]
		]
		var palette: Array = palettes[stage - 1]
		var day_top: Color = palette[0]
		var day_bottom: Color = palette[1]
		var day_hill: Color = palette[2]
		var top_color := day_top.lerp(Color("08102f"), 0.72 if night else 0.0)
		var bottom_color := day_bottom.lerp(Color("171d4b"), 0.82 if night else 0.0)
		draw_rect(Rect2(0.0, 0.0, 1280.0, 720.0), top_color, true)
		for y in range(720):
			var blend := float(y) / 720.0
			var color := top_color.lerp(bottom_color, blend)
			draw_rect(Rect2(0.0, y, 1280.0, 1.0), color, true)
		var hill_color := day_hill.lerp(Color("182b54"), 0.72 if night else 0.0)
		draw_colored_polygon(PackedVector2Array([Vector2(0.0, 545.0), Vector2(190.0, 430.0), Vector2(410.0, 545.0), Vector2(690.0, 410.0), Vector2(1000.0, 550.0), Vector2(1280.0, 445.0), Vector2(1280.0, 720.0), Vector2(0.0, 720.0)]), hill_color)
	if night and not stage_backgrounds.has(stage):
		for star in background_stars:
			draw_circle(star, 1.4, Color(0.87, 0.94, 1.0, 0.8))
	if lightning_time > 0.0:
		draw_rect(Rect2(0.0, 0.0, 1280.0, 720.0), Color(0.83, 0.91, 1.0, 0.42), true)
		var bolt := PackedVector2Array([Vector2(930, 0), Vector2(890, 120), Vector2(940, 120), Vector2(870, 255), Vector2(960, 130), Vector2(915, 130), Vector2(965, 0)])
		draw_colored_polygon(bolt, Color(0.95, 0.98, 1.0, 0.95))


func _draw_field() -> void:
	var floor_colors := [Color("ac713b"), Color("c5b6a5"), Color("a9d9ec"), Color("be975d"), Color("303957"), Color("526274"), Color("7f8a77"), Color("a69180"), Color("657452"), Color("697689")]
	var floor_color: Color = floor_colors[stage - 1]
	draw_rect(Rect2(0, PLAYER_FOOT_Y, 1280, 110), floor_color.darkened(0.35))
	draw_rect(Rect2(0, PLAYER_FOOT_Y, 1280, 8), floor_color.lightened(0.3))
	for x in range(0, 1280, 80):
		draw_rect(Rect2(x + 2, PLAYER_FOOT_Y + 12, 76, 30), floor_color)
		draw_line(Vector2(x + 4, PLAYER_FOOT_Y + 14), Vector2(x + 74, PLAYER_FOOT_Y + 14), floor_color.lightened(0.15), 2)
	draw_rect(FIELD, Color(0.025, 0.06, 0.13, 0.22), true)
	draw_rect(FIELD, Color(0.75, 0.88, 1.0, 0.45), false, 2.0)
	var weather := _weather_for_stage().to_upper()
	_ui_text( Vector2(1010.0, 137.0), weather, HORIZONTAL_ALIGNMENT_RIGHT, 220.0, 13, Color(0.87, 0.93, 1.0, 0.85))


func _draw_weather() -> void:
	var weather := _weather_for_stage()
	if weather == "snow":
		for particle in weather_particles:
			draw_circle(particle["position"], 2.5, Color(0.95, 0.99, 1.0, 0.8))
	elif weather == "rain" or weather == "storm":
		for particle in weather_particles:
			var position: Vector2 = particle["position"]
			draw_line(position, position + Vector2(-4.0, 12.0), Color(0.72, 0.88, 1.0, 0.52), 1.4)


func _draw_balls() -> void:
	var colors := [Color("ff5c6c"), Color("ffbe45"), Color("6ce5bb"), Color("53b7ff"), Color("cc8cff")]
	for ball in balls:
		var position: Vector2 = Vector2(ball.get("previous_position", ball["position"])).lerp(ball["position"], _render_fraction())
		var radius: float = ball["radius"]
		var color: Color = colors[int(ball["color"]) % colors.size()]
		draw_circle(position + Vector2(3, 6), radius + 2, Color(0.01, 0.02, 0.07, 0.45))
		draw_circle(position, radius, color.darkened(0.65))
		# Offset nested surfaces approximate a lit spherical volume.
		for layer in range(18):
			var t := float(layer) / 17.0
			var center := position + Vector2(-radius * 0.21, -radius * 0.24) * t
			draw_circle(center, radius * (0.96 - t * 0.55), color.darkened(0.4).lerp(color.lightened(0.4), t))
		draw_arc(position, radius * 0.91, 0.2, 1.6, 24, Color(0.65, 0.9, 1.0, 0.65), 2.0, true)
		draw_arc(position, radius * 0.73, -2.8, -1.55, 20, Color(1, 1, 1, 0.7), 3.0, true)
		draw_circle(position + Vector2(-radius * 0.3, -radius * 0.38), radius * 0.13, Color(1, 1, 1, 0.85))
		draw_circle(position + Vector2(radius * 0.22, radius * 0.37), radius * 0.06, Color(0.8, 0.94, 1, 0.65))



func _draw_projectiles() -> void:
	for projectile in projectiles:
		var x := float(projectile["x"])
		var base_y := float(projectile["base_y"])
		var tip_y := lerpf(float(projectile.get("previous_tip", projectile["tip_y"])), float(projectile["tip_y"]), _render_fraction())
		var tint: Color = [Color("d7b778"), Color("bd8e53"), Color("e5cd9a")][int(projectile.get("wire", 0))]
		var rope_base := Vector2(x, base_y)
		var rope_tip := Vector2(x, tip_y)
		if bool(projectile.get("roof", false)) and tip_y <= FIELD.position.y + 1:
			draw_rect(Rect2(x - 9, tip_y - 3, 18, 7), Color("b5ff74"))
			draw_rect(Rect2(x - 11, PLAYER_FOOT_Y - 5, 22, 5), Color("b5ff74"))
			_ui_text( Vector2(x + 12, tip_y + 18), "%.1fs" % float(projectile["roof_left"]), HORIZONTAL_ALIGNMENT_LEFT, 60, 14, Color("b5ff74"))
		draw_line(rope_base + Vector2(2, 1), rope_tip + Vector2(2, 1), Color(0.04, 0.02, 0.01, 0.5), 8.0)
		draw_line(rope_base, rope_tip, Color("503b25"), 7.0)
		draw_line(rope_base, rope_tip, tint, 9.0 if int(projectile.get("weapon", 0)) == 6 else 5.0)
		var weave_y := base_y - 8.0
		while weave_y > tip_y + 6.0:
			draw_line(Vector2(x - 2.2, weave_y), Vector2(x + 2.2, weave_y - 4.0), Color("755334"), 1.8)
			weave_y -= 6.0
		draw_colored_polygon(PackedVector2Array([
			Vector2(x, tip_y - 8.0),
			Vector2(x + 4.2, tip_y + 1.0),
			Vector2(x, tip_y - 1.0),
			Vector2(x - 4.2, tip_y + 1.0)
		]), Color("d8e2e6"))


func _prepare_collapse_frames() -> void:
	var atlas := load("res://assets/characters/collapse_atlas.png") as Texture2D
	var bitmap := atlas.get_image()
	var cell := Vector2(atlas.get_width() / 4.0, atlas.get_height() / 10.0)
	for number in range(1, MAX_STAGE + 1):
		var frames: Array = []
		for frame in range(4):
			var region := Rect2i(int(frame * cell.x), int((number - 1) * cell.y), int(cell.x), int(cell.y))
			var bottom := region.size.y - 1
			while bottom > 0:
				var found := false
				for x in range(region.size.x):
					if bitmap.get_pixel(region.position.x + x, region.position.y + bottom).a >= 0.8:
						found = true
						break
				if found:
					break
				bottom -= 1
			var texture := AtlasTexture.new()
			texture.atlas = atlas
			texture.region = Rect2(region)
			frames.append({"texture": texture, "bottom": float(bottom + 1) / region.size.y})
		collapse_frames[number] = frames


func _collapse_frame_index() -> int:
	if faint_time < 0.22:
		return 0
	if faint_time < 0.48:
		return 1
	if faint_time < 0.85:
		return 2
	return 3


func _collapse_frame_rect() -> Rect2:
	var frame: Dictionary = collapse_frames[stage][_collapse_frame_index()]
	return Rect2(-COLLAPSE_CELL_SIZE * 0.5, -float(frame["bottom"]) * COLLAPSE_CELL_SIZE, COLLAPSE_CELL_SIZE, COLLAPSE_CELL_SIZE)


func _draw_collapse() -> void:
	var x := clampf(faint_origin.x, 100, 1180)
	draw_set_transform(Vector2(x, PLAYER_FOOT_Y + 1), 0, Vector2(1, 0.13))
	draw_circle(Vector2.ZERO, 45 if faint_grounded else 24, Color(0, 0, 0, 0.3))
	draw_set_transform(Vector2.ZERO)
	if not faint_grounded:
		var airborne_texture: Texture2D = player_walk_frames[stage][2] if player_walk_frames.has(stage) else player_costumes[stage - 1]
		draw_set_transform(faint_origin, 0, Vector2(player_facing, 1))
		var air_size := _character_size(airborne_texture)
		draw_texture_rect(airborne_texture, Rect2(-air_size * 0.5, air_size), false)
		draw_set_transform(Vector2.ZERO)
		return
	var frame: Dictionary = collapse_frames[stage][_collapse_frame_index()]
	draw_set_transform(Vector2(x, PLAYER_FOOT_Y), 0, Vector2(player_facing, 1))
	draw_texture_rect(frame["texture"], _collapse_frame_rect(), false)
	draw_set_transform(Vector2.ZERO)
	if faint_time > 0.85:
		for i in range(3):
			var star := Vector2(x + player_facing * 38 + sin(faint_time * 4 + i * 2) * 9, PLAYER_FOOT_Y - 40 - cos(faint_time * 4 + i * 2) * 4)
			draw_circle(star, 2.5, Color("ffe2a0"))


func _draw_hurt_player() -> void:
	if player_costumes.is_empty():
		return
	if game_state == "fainting":
		_draw_collapse()
		return
	var position := previous_player_position.lerp(player_position, _render_fraction())
	var seated := player_is_sitting and player_sitting_poses.has(stage)
	var texture: Texture2D = player_sitting_poses[stage] if seated else player_walk_frames[stage][1]
	var size := _character_size(texture, SEATED_HEIGHT if seated else PLAYER_DRAW_SIZE.y)
	var strength := clampf(hurt_time / HURT_DURATION, 0, 1)
	var feet := position + Vector2(sin(hurt_time * 32) * 2 * strength, PLAYER_HALF_HEIGHT)
	draw_set_transform(feet, 0, Vector2(player_facing, 1))
	draw_texture_rect(texture, Rect2(-size.x * 0.5, -size.y, size.x, size.y), false, Color(1, 1 - strength * 0.22, 1 - strength * 0.22))
	var face := Vector2(size.x * 0.19, -size.y * 0.79)
	for offset in [-5.0, 5.0]:
		var eye := face + Vector2(offset, 0)
		draw_line(eye + Vector2(-2, -2), eye + Vector2(2, 2), Color("39252c"), 1.6, true)
		draw_line(eye + Vector2(-2, 2), eye + Vector2(2, -2), Color("39252c"), 1.6, true)
	draw_circle(face + Vector2(0, 8), 2.7, Color("532c37"))
	draw_set_transform(Vector2.ZERO)
	_hud_text("OUCH!", feet + Vector2(-31, -size.y - 12), 20, Color("ffb2be"))


func _draw_player() -> void:
	if game_state == "fainting" or hurt_time > 0.0:
		_draw_hurt_player()
		return
	if invulnerable_time > 0.0 and int(Time.get_ticks_msec() / 90) % 2 == 0:
		return
	var position := previous_player_position.lerp(player_position, _render_fraction())
	draw_set_transform(Vector2(position.x, PLAYER_FOOT_Y + 2), 0, Vector2(1, 0.22))
	draw_circle(Vector2.ZERO, 27, Color(0.02, 0.04, 0.1, 0.32))
	draw_set_transform(Vector2.ZERO)
	if shield_time > 0.0:
		draw_arc(position, 61, 0, TAU, 48, Color(0.4, 0.9, 1, 0.65), 2, true)
	if player_fire_pose_time > 0.0 and stage > 0 and stage <= player_costumes.size():
		var front_pose: Texture2D = player_sitting_poses[stage] if player_is_sitting and player_sitting_poses.has(stage) else player_costumes[stage - 1]
		var front_size := _character_size(front_pose, SEATED_HEIGHT if player_is_sitting else PLAYER_DRAW_SIZE.y)
		draw_texture_rect(front_pose, Rect2(position.x - front_size.x * 0.5, position.y + PLAYER_HALF_HEIGHT - front_size.y, front_size.x, front_size.y), false)
		var muzzle := _launcher_origin() + position - player_position
		draw_line(muzzle + Vector2(0, 34), muzzle, Color("17223b"), 13.0, true)
		draw_line(muzzle + Vector2(0, 30), muzzle, Color("d8e9ee"), 7.0, true)
		draw_circle(muzzle + Vector2(0, 22), 6, Color("ffc789"))
		draw_circle(muzzle, 5, Color("64e7ff"))
		return
	if player_is_sitting and player_sitting_poses.has(stage):
		var seated_pose: Texture2D = player_sitting_poses[stage]
		var seated_facing := player_facing
		var sit_size := _character_size(seated_pose, SEATED_HEIGHT)
		draw_set_transform(Vector2(position.x, PLAYER_FOOT_Y - sit_size.y), 0.0, Vector2(seated_facing, 1.0))
		draw_texture_rect(seated_pose, Rect2(-sit_size.x * 0.5, 0.0, sit_size.x, sit_size.y), false)
		draw_set_transform(Vector2.ZERO, 0.0, Vector2.ONE)
		return
	if stage > 0 and stage <= player_costumes.size():
		var moving := absf(player_movement.x) > 0.08 and not player_is_airborne and not player_is_sitting
		var bounce := 0.0
		var costume_texture: Texture2D = player_costumes[stage - 1]
		if player_walk_frames.has(stage):
			var frame_list: Array = player_walk_frames[stage]
			var frame_index := 1
			if moving:
				frame_index = mini(int(player_walk_phase / TAU * 4.0), 3)
			elif player_is_airborne:
				frame_index = 0 if player_vertical_speed < 0.0 else 2
			costume_texture = frame_list[frame_index]
		var landing_squash := clampf(player_landing_time / 0.16, 0.0, 1.0)
		var sprite_height_scale := 1.0
		var sprite_width_scale := 1.0
		if player_is_airborne and player_vertical_speed < 0.0:
			sprite_height_scale = 1.02
			sprite_width_scale = 0.99
		elif player_is_airborne:
			sprite_height_scale = 0.98
			sprite_width_scale = 1.01
		elif landing_squash > 0.0:
			sprite_height_scale = 1.0 - landing_squash * 0.06
			sprite_width_scale = 1.0 + landing_squash * 0.04
		var floor_anchor_offset := PLAYER_HALF_HEIGHT * (1.0 - sprite_height_scale) if not player_is_airborne else 0.0
		var facing := player_facing
		var body_tilt := 0.0
		if moving:
			body_tilt = -player_facing * sin(player_walk_phase * 2.0) * 0.025
		elif player_is_airborne:
			body_tilt = -player_facing * (-0.045 if player_vertical_speed < 0.0 else 0.035)
		elif idle_glancing:
			body_tilt = -player_facing * 0.035
		var recoil_offset := Vector2(-player_facing * player_shot_kick * 2.5, 0.0)
		if moving:
			var dust_x := -17.0 if sin(player_walk_phase) > 0.0 else 17.0
			draw_circle(position + Vector2(dust_x, PLAYER_HALF_HEIGHT - 3.0), 3.5, Color(0.94, 0.81, 0.62, 0.42))
		draw_set_transform(position + recoil_offset + Vector2(0.0, floor_anchor_offset - bounce), body_tilt, Vector2(facing * sprite_width_scale, sprite_height_scale))
		var drawn_size := _character_size(costume_texture)
		draw_texture_rect(costume_texture, Rect2(-drawn_size.x * 0.5, -PLAYER_HALF_HEIGHT, drawn_size.x, drawn_size.y), false)
		draw_set_transform(Vector2.ZERO, 0.0, Vector2.ONE)
		return
	var facing := Vector2.UP
	var side := Vector2(-facing.y, facing.x)
	draw_colored_polygon(PackedVector2Array([
		position - side * 12.0 + Vector2(0.0, 5.0),
		position + side * 12.0 + Vector2(0.0, 5.0),
		position + side * 8.0 - Vector2(0.0, 16.0),
		position - side * 8.0 - Vector2(0.0, 16.0)
	]), Color("2356a5"))
	draw_circle(position + Vector2(0.0, -17.0), 10.5, Color("ffd0a5"))
	draw_circle(position + facing * 4.0 + Vector2(0.0, -18.0), 2.0, Color("16213e"))
	draw_line(position, position + facing * 29.0, Color("fff3c6"), 5.0)


func _draw_hud() -> void:
	_draw_glass_panel(Rect2(40, 655, 1200, 44), WIRE_COLORS[wire_type])
	if notice_time > 0:
		_hud_text(pickup_notice, Vector2(500, 155), 18, Color("ffe4a0"))
	if weapon_time > 0:
		_hud_text("%s  %.0fs" % [["", "", "", "", "TRIPLE HARPOON", "RAPID REEL", "HEAVY SPEAR"][weapon_kind], weapon_time], Vector2(60, 642), 15, Color("bfe9ff"))
	if roof_ready:
		_ui_text( Vector2(800, 642), "ROOF TRAP READY · NEXT SHOT", HORIZONTAL_ALIGNMENT_LEFT, 420, 17, Color("b5ff74"))
	if (power_time > 0 or shield_time > 0) and weapon_time <= 0:
		_ui_text( Vector2(60, 642), "WEAPON BOOST %.0fs    SHIELD %.0fs" % [power_time, shield_time], HORIZONTAL_ALIGNMENT_LEFT, 700, 17, Color("fff0b0"))
	var benefits := ["Fast launch / quick reload", "Holds at ceiling for 1.4s", "Pops two balls per shot"]
	_ui_text( Vector2(60, 683), "ROPE: " + WIRE_NAMES[wire_type] + "  ·  " + benefits[wire_type], HORIZONTAL_ALIGNMENT_LEFT, 710, 18, WIRE_COLORS[wire_type])
	_ui_text( Vector2(790, 683), "1 / 2 / 3 · Q · Wheel · Pad RB", HORIZONTAL_ALIGNMENT_LEFT, 440, 16, Color.WHITE)
	# Compact arcade dashboard: separate score, ball count and player status.
	draw_rect(Rect2(0, 0, 1280, 106), Color("080f23"))
	draw_line(Vector2(0, 104), Vector2(1280, 104), Color("50dbe5"), 2)
	_draw_glass_panel(Rect2(14, 9, 300, 87), Color("47d7ef"))
	_draw_glass_panel(Rect2(325, 9, 345, 87), Color("ffd36a"))
	_draw_glass_panel(Rect2(681, 9, 145, 87), Color("7ce9ce"))
	_draw_glass_panel(Rect2(837, 9, 242, 87), Color("d896ff"))
	_draw_glass_panel(Rect2(1090, 9, 176, 87), Color("ffb470"))
	_draw_logo(Vector2(34, 51), 34)
	_hud_text("WORLD %02d / 10" % stage, Vector2(35, 91), 13, Color("bbd5ec"))
	for i in range(10):
		draw_rect(Rect2(182 + i * 10, 82, 7, 9), Color("65dfed") if i < stage else Color("28394e"))
	_hud_text("TOTAL SCORE", Vector2(346, 32), 11, Color("c3bda3"))
	_hud_text("%07d" % score, Vector2(345, 66), 30, Color("ffe6a4"))
	_hud_text("ROPE  " + WIRE_NAMES[wire_type], Vector2(347, 85), 11, WIRE_COLORS[wire_type])
	_hud_text("BALLS LEFT", Vector2(700, 32), 11, Color("a8d7d0"))
	_hud_text("%02d" % balls.size(), Vector2(719, 73), 34, Color("92ffda"))
	_hud_text("HEALTH", Vector2(852, 37), 11, Color("ffd6dc"))
	_hud_text("CONTINUES", Vector2(852, 75), 11, Color("e6d6ff"))
	for i in range(3):
		draw_circle(Vector2(966 + i * 34, 32), 14, Color("0b152b"))
		_draw_heart(Vector2(966 + i * 34, 32), i < hearts)
		draw_circle(Vector2(966 + i * 34, 71), 13, Color("0b152b"))
		_draw_diamond(Vector2(966 + i * 34, 71), i < continues)
	_hud_text("SPECIAL", Vector2(1107, 31), 11, Color("dfc4a6"))
	_hud_text("READY" if special_ready else "USED", Vector2(1107, 53), 18, Color("ffe08c") if special_ready else Color("8593a8"))
	draw_rect(Rect2(1100, 65, 150, 24), Color("263e59"))
	_hud_text("RESUME [ESC]" if paused else "PAUSE [ESC]", Vector2(1111, 82), 12, Color("d7efff"))


func _ui_text(position: Vector2, value: String, alignment: int, width: float, size: int, tint: Color) -> void:
	var font: Font = body_font if body_font != null else ThemeDB.fallback_font
	var fitted := size
	if width > 0:
		while fitted > 10 and font.get_string_size(value, HORIZONTAL_ALIGNMENT_LEFT, -1, fitted).x > width:
			fitted -= 1
	draw_string(font, position + Vector2(0, 2), value, alignment, width, fitted, Color(0.015,0.025,0.06,0.9))
	draw_string_outline(font, position, value, alignment, width, fitted, 2, Color("08162d"))
	draw_string(font, position, value, alignment, width, fitted, tint)


func _hud_text(value: String, position: Vector2, size: int, tint: Color) -> void:
	if size >= 20:
		_display_text(value, position, size, tint)
	else:
		_ui_text(position, value, HORIZONTAL_ALIGNMENT_LEFT, -1, maxi(size, 13), tint)


func _display_text(value: String, position: Vector2, size: int, tint: Color, alignment: int = HORIZONTAL_ALIGNMENT_LEFT, width: float = -1) -> void:
	var font: Font = logo_font if logo_font != null else ThemeDB.fallback_font
	for depth in range(3, 0, -1):
		draw_string(font, position + Vector2(0,depth), value, alignment, width, size, tint.darkened(0.75))
	draw_string_outline(font, position, value, alignment, width, size, 5, Color("07142c"))
	draw_string_outline(font, position, value, alignment, width, size, 1, tint.lightened(0.55))
	draw_string(font, position, value, alignment, width, size, tint)


func _draw_glass_panel(rect: Rect2, accent: Color) -> void:
	var panel := StyleBoxFlat.new()
	panel.bg_color = Color("16263e")
	panel.set_corner_radius_all(10)
	panel.set_border_width_all(1)
	panel.border_color = accent.darkened(0.55)
	panel.shadow_color = Color(0, 0, 0, 0.65)
	panel.shadow_size = 3
	panel.shadow_offset = Vector2(0, 3)
	draw_style_box(panel, rect)
	# Gloss band, luminous top edge and an inset lower edge.
	draw_rect(Rect2(rect.position + Vector2(9, 3), Vector2(rect.size.x - 18, 29)), Color(0.55, 0.72, 0.9, 0.065))
	draw_line(rect.position + Vector2(12, 2), rect.position + Vector2(rect.size.x - 12, 2), Color(accent, 0.85), 2, true)
	draw_line(rect.position + Vector2(12, rect.size.y - 3), rect.end - Vector2(12, 3), Color("060e20"), 2)



func _draw_bevel(rect: Rect2, color: Color) -> void:
	draw_rect(Rect2(rect.position + Vector2(0, 5), rect.size), Color(0.01, 0.02, 0.05, 0.8))
	draw_rect(rect, color.darkened(0.25))
	for y in range(int(rect.size.y)):
		var t := float(y) / rect.size.y
		draw_line(rect.position + Vector2(2, y), rect.position + Vector2(rect.size.x - 2, y), color.lightened(0.15).lerp(color.darkened(0.3), t), 1)
	draw_line(rect.position, rect.position + Vector2(rect.size.x, 0), color.lightened(0.55), 2)
	draw_line(rect.position, rect.position + Vector2(0, rect.size.y), color.lightened(0.3), 2)
	draw_line(rect.position + Vector2(0, rect.size.y), rect.end, Color("081224"), 3)


func _draw_logo(position: Vector2, size: int, centered: bool = true) -> void:
	if logo_art == null:
		return
	var slot := Rect2(position.x - 10, 6, 238, 70)
	if size > 50:
		slot = Rect2(position.x - 16, position.y - 140, 545, 180)
		if centered:
			slot.position.x = (1280.0 - slot.size.x) * 0.5
	var texture_size := logo_art.get_size()
	var ratio := minf(slot.size.x / texture_size.x, slot.size.y / texture_size.y)
	var drawn_size := texture_size * ratio
	draw_texture_rect(logo_art, Rect2(slot.position + (slot.size - drawn_size) * 0.5, drawn_size), false)


func _load_reaction_portraits() -> void:
	var atlas := load("res://assets/characters/reaction_portraits.png") as Texture2D
	var cell := atlas.get_size() * 0.5
	for i in range(4):
		var frame := AtlasTexture.new()
		frame.atlas = atlas
		frame.region = Rect2(Vector2(i % 2, i / 2) * cell, cell)
		reaction_frames.append(frame)


func _reaction_frame(happy: bool, time: float) -> int:
	if not happy:
		return 3
	var blink_phase := fmod(time, 2.7)
	if blink_phase > 0.75 and blink_phase < 0.95:
		return 2
	return int(time / 0.24) % 2


func _draw_reaction_portrait(rect: Rect2, happy: bool, time: float) -> void:
	if reaction_frames.size() != 4:
		return
	var bob := sin(time * 8) * 2.5 if happy else sin(time * 1.5) * 1.0
	var intro := smoothstep(0, 0.35, time)
	var offset := Vector2((1.0 - intro) * -24, bob)
	var texture := reaction_frames[_reaction_frame(happy, time)]
	draw_texture_rect(texture, Rect2(rect.position + offset, rect.size), false)


func _draw_celebration() -> void:
	draw_rect(Rect2(0, 106, 1280, 549), Color(0.01, 0.025, 0.07, 0.8))
	for i in range(80):
		var x := fposmod(i * 137.0 + sin(celebration_time * 2 + i) * 25, 1280)
		var y := 110 + fposmod(i * 53.0 + celebration_time * (65 + i % 45), 540)
		draw_rect(Rect2(x, y, 5, 10), WIRE_COLORS[i % 3])
	_draw_glass_panel(Rect2(155,155,970,440),Color("e7bf69"))
	_draw_reaction_portrait(Rect2(175,180,370,370), true, celebration_time)
	_hud_text("ALRIGHT!!!", Vector2(248,562), 31, Color("ffe394"))
	_hud_text("STAGE CLEAR!" if stage < MAX_STAGE else "WORLD CHAMPION!", Vector2(585,237), 30, Color("ffe09c"))
	_hud_text("WORLD %02d / 10" % stage, Vector2(588,282), 20, Color("c0e8ff"))
	_hud_text("BONUS  +%d" % stage_bonus, Vector2(585,345), 29, Color("a5f4c6"))
	_hud_text("SCORE  %07d" % score, Vector2(586,390), 25, Color.WHITE)
	var prompt := "TAKE A BREATHER..." if celebration_time < 1.5 else "ENTER / CLICK / PAD A"
	_hud_text(prompt, Vector2(588,465), 18, Color("ffe09c"))
	if celebration_time >= 1.5:
		_hud_text("TO CONTINUE YOUR ADVENTURE", Vector2(588,495), 14, Color("b6cfe7"))


func _draw_heart(center: Vector2, active: bool) -> void:
	var color := Color("ff405c") if active else Color("513b52")
	draw_circle(center + Vector2(-3.0, 1.0), 7.0, color.darkened(0.6))
	draw_circle(center + Vector2(6.0, 1.0), 7.0, color.darkened(0.6))
	draw_circle(center + Vector2(-4.5, -2.0), 6.0, color)
	draw_circle(center + Vector2(4.5, -2.0), 6.0, color)
	draw_colored_polygon(PackedVector2Array([center + Vector2(-10.0, 0.0), center + Vector2(10.0, 0.0), center + Vector2(0.0, 11.0)]), color)
	if active:
		draw_circle(center + Vector2(-5, -4), 2.2, Color("ffb7c4"))
		draw_line(center + Vector2(1, 8), center + Vector2(8, 1), color.darkened(0.3), 2)


func _draw_diamond(center: Vector2, active: bool) -> void:
	var color := Color("bd79ff") if active else Color("49395d")
	draw_colored_polygon(PackedVector2Array([center + Vector2(0.0, -9.0), center + Vector2(8.0, 0.0), center + Vector2(0.0, 9.0), center + Vector2(-8.0, 0.0)]), color)
	draw_colored_polygon(PackedVector2Array([center, center + Vector2(8, 0), center + Vector2(0, 9)]), color.darkened(0.35))
	draw_colored_polygon(PackedVector2Array([center, center + Vector2(-8, 0), center + Vector2(0, -9)]), color.lightened(0.35))
	draw_line(center + Vector2(-3.0, -2.0), center + Vector2(0.0, -5.0), Color(1.0, 0.9, 1.0, 0.6), 1.2)


func _draw_name_entry() -> void:
	draw_rect(Rect2(0,106,1280,549),Color(0.01,0.02,0.065,0.85))
	_draw_glass_panel(Rect2(155,155,970,440), Color("d8c087") if run_won else Color("8098bd"))
	_draw_reaction_portrait(Rect2(175,185,370,370), run_won, outcome_time)
	_hud_text("AN ADVENTURE TO REMEMBER" if run_won else "EVEN HEROES NEED A REST", Vector2(185,563), 13, Color("bed5e6"))
	_hud_text("YOU DID IT!" if run_won else "GAME OVER", Vector2(585,235), 34, Color("ffe09c") if run_won else Color("c3d9ef"))
	_hud_text("ALL TEN WORLDS CLEARED" if run_won else "NO CONTINUES LEFT", Vector2(588,279), 17, Color("a9c5de"))
	_hud_text("FINAL SCORE  %07d" % score, Vector2(587,335), 24, Color("fff1c0"))
	_hud_text("ENTER YOUR NAME", Vector2(588,395), 15, Color("bcd8ef"))
	var entry := name_entry if not name_entry.is_empty() else "TYPE NAME"
	_hud_text(entry + "_", Vector2(585,440), 29, Color("ffe394"))
	_hud_text("ENTER TO SAVE YOUR SCORE", Vector2(588,502), 15, Color("c3d7e8"))


func _begin_game() -> void:
	if game_state != "title" or settings_open:
		return
	game_state = "playing"
	start_fire_blocked = true
	if _audio_asset("title_music") != null:
		_load_music()
	_play_sound("pickup")


func _title_prompt_visible() -> bool:
	return fmod(title_time, 1.4) < 1.0


func _draw_title() -> void:
	draw_rect(Rect2(0, 0, 1280, 720), Color("08152c"))
	if title_art != null:
		var source := title_art.get_size()
		var scale := maxf(1280.0 / source.x, 720.0 / source.y)
		var drawn_size := source * scale
		draw_texture_rect(title_art, Rect2((Vector2(1280,720) - drawn_size) * 0.5, drawn_size), false)
	# A soft vignette keeps the interactive text legible over the illustration.
	for x in range(0, 720, 8):
		var opacity := 0.32 * pow(1.0 - x / 720.0, 1.4)
		draw_rect(Rect2(x,0,8,720),Color(0.01,0.025,0.075,opacity))
	for i in range(22):
		var spark := Vector2(fposmod(i * 137.0 + title_time * (4 + i % 5), 1280), 80 + fposmod(i * 73.0 - title_time * 12, 480))
		draw_circle(spark, 1.4, Color(1,0.85,0.5,0.2 + 0.2 * sin(title_time + i)))
	_hud_text("QUIETBYTE PRESENTS", Vector2(70, 100), 16, Color("b8dce9"))
	_draw_logo(Vector2(66, 257), 102, false)
	_hud_text("ONE EXPLORER. TEN WORLDS.", Vector2(73, 350), 21, Color("ffe9b8"))
	_hud_text("POP YOUR WAY TO THE MOON", Vector2(73, 386), 17, Color("a5e4ed"))
	draw_line(Vector2(73, 412), Vector2(520, 412), Color(0.98,0.78,0.36,0.7),2,true)
	_hud_text("BOUNCING BALLS · BRAIDED ROPES · BIG ADVENTURE", Vector2(73, 444), 12, Color("d5e7ee"))
	if _title_prompt_visible():
		_hud_text("PRESS ANY KEY TO START", Vector2(73, 548), 25, Color("fff1b0"))
	_hud_text("CLICK OR PRESS ANY GAMEPAD BUTTON", Vector2(75, 581), 12, Color("b7d5e5"))
	var cards := [Rect2(64,611,160,44),Rect2(238,611,160,44),Rect2(412,611,190,44)]
	var labels := ["F1  SETTINGS", "H  TOP TEN", "M  MUSIC " + ("ON" if music_enabled else "OFF")]
	var mouse := get_global_mouse_position()
	for i in range(cards.size()):
		_draw_glass_panel(cards[i], Color("ffd984") if cards[i].has_point(mouse) else Color("6095b0"))
		_hud_text(labels[i], cards[i].position + Vector2(14,28), 13, Color("e8eef2"))
	_hud_text("A SKYPOP ARCADE ADVENTURE", Vector2(72, 697), 11, Color("7996b0"))
	_hud_text("F11  FULLSCREEN", Vector2(1100, 697), 11, Color("a3b8c7"))


func _draw_fancy_text(text: String, position: Vector2, width: float, font_size: int, color: Color, alignment: int) -> void:
	_display_text(text, position, font_size, color, alignment, width)


func _draw_leaderboard() -> void:
	draw_rect(Rect2(0,106,1280,549),Color(0.01,0.02,0.065,0.72))
	_draw_glass_panel(Rect2(240,115,800,515),Color("69dce7"))
	_display_text("HALL OF FAME", Vector2(0,177), 46, Color("ffe093"), HORIZONTAL_ALIGNMENT_CENTER, 1280)
	_ui_text(Vector2(0,205), "TOP TEN EXPLORERS", HORIZONTAL_ALIGNMENT_CENTER,1280,14,Color("a8e7ee"))
	_ui_text(Vector2(289,239), "RANK", HORIZONTAL_ALIGNMENT_LEFT,80,12,Color("8cabc5"))
	_ui_text(Vector2(392,239), "EXPLORER", HORIZONTAL_ALIGNMENT_LEFT,250,12,Color("8cabc5"))
	_ui_text(Vector2(817,239), "HIGH SCORE", HORIZONTAL_ALIGNMENT_RIGHT,162,12,Color("8cabc5"))
	var medals := [Color("ffd46c"),Color("cde5f5"),Color("e9ac7a")]
	for i in range(10):
		var y := 254.0 + i * 29
		var has_entry := i < leaderboard_entries.size()
		var tint: Color = medals[i] if i < 3 else Color("b3d0e5")
		draw_rect(Rect2(274,y,732,27),Color(tint,0.085 if has_entry else 0.025))
		if i < 3 and has_entry:
			draw_circle(Vector2(319,y+13),12,tint.darkened(0.55))
			draw_arc(Vector2(319,y+13),11,0,TAU,24,tint,1.5,true)
			_ui_text(Vector2(306,y+19),str(i+1),HORIZONTAL_ALIGNMENT_CENTER,26,15,tint)
		else:
			_ui_text(Vector2(303,y+20),"%02d" % (i+1),HORIZONTAL_ALIGNMENT_LEFT,45,16,Color("8cabc5"))
		var entry_name := str(leaderboard_entries[i].get("name","PLAYER")) if has_entry else "—"
		var entry_score := "%07d" % int(leaderboard_entries[i].get("score",0)) if has_entry else "—"
		_ui_text(Vector2(392,y+21),entry_name,HORIZONTAL_ALIGNMENT_LEFT,340,20,Color("f2f6ff") if has_entry else Color("617c94"))
		draw_string(score_font,Vector2(785,y+21),entry_score,HORIZONTAL_ALIGNMENT_RIGHT,194,21,tint if has_entry else Color("617c94"))
		if i == 2:
			draw_line(Vector2(274,y+28),Vector2(1006,y+28),Color(0.7,0.8,1,0.18),1)
	_display_text("YOUR NEXT ADVENTURE AWAITS",Vector2(0,585),21,Color("f5d78d"),HORIZONTAL_ALIGNMENT_CENTER,1280)
	_ui_text(Vector2(0,610),"ENTER / GAMEPAD START / CLICK TO PLAY AGAIN",HORIZONTAL_ALIGNMENT_CENTER,1280,13,Color("b9d3e9"))


func _draw_settings() -> void:
	_draw_glass_panel(Rect2(280,125,720,500),Color("70dae6"))
	_display_text("SETTINGS", Vector2(0,172), 40, Color("ffe093"), HORIZONTAL_ALIGNMENT_CENTER, 1280)
	_ui_text( Vector2(365, 207), "MUSIC: " + ("ON" if music_enabled else "OFF") + "   [M / CLICK TO TOGGLE]", HORIZONTAL_ALIGNMENT_LEFT, 550, 18, Color("63d8ce"))
	for row in range(2 + REMAPPABLE_ACTIONS.size()):
		var y := 246.0 + row * 43.0
		var color := Color("ffe3a2") if row == settings_cursor else Color("e3ebf7")
		draw_rect(Rect2(344,y-25,590,35),Color(0.9,0.75,0.4,0.12) if row == settings_cursor else Color(0.6,0.8,1.0,0.035))
		if row == 0 or row == 1:
			var label := "MUSIC VOLUME" if row == 0 else "EFFECTS VOLUME"
			var value := music_volume if row == 0 else effects_volume
			_ui_text( Vector2(365.0, y), label, HORIZONTAL_ALIGNMENT_LEFT, 220.0, 17, color)
			draw_rect(Rect2(620.0, y - 15.0, 245.0, 16.0), Color("273956"), true)
			draw_rect(Rect2(620.0, y - 15.0, 245.0 * value, 16.0), Color("63d8ce"), true)
			_ui_text( Vector2(880.0, y), "%d%%" % roundi(value * 100.0), HORIZONTAL_ALIGNMENT_LEFT, 70.0, 16, color)
		else:
			var action_index := row - 2
			var action_name: String = REMAPPABLE_ACTIONS[action_index]
			var action_label := action_name.replace("_", " ").to_upper()
			var binding := _binding_label(action_name)
			if remapping_action == action_name:
				binding = "PRESS A KEY"
			_ui_text( Vector2(365.0, y), action_label, HORIZONTAL_ALIGNMENT_LEFT, 250.0, 17, color)
			_ui_text( Vector2(700.0, y), binding, HORIZONTAL_ALIGNMENT_LEFT, 220.0, 17, color)
	_ui_text( Vector2(0.0, 590.0), "UP/DOWN SELECT   LEFT/RIGHT ADJUST   ENTER REBIND   F1/ESC CLOSE", HORIZONTAL_ALIGNMENT_CENTER, 1280.0, 14, Color("b8c4d8"))


func _binding_label(action_name: String) -> String:
	for event in InputMap.action_get_events(action_name):
		if event is InputEventKey:
			var key_event := event as InputEventKey
			return OS.get_keycode_string(key_event.physical_keycode)
	return "UNBOUND"


func _exit_tree() -> void:
	# Release looping WAV playbacks before the audio server shuts down.
	if is_instance_valid(music_player):
		music_player.stop()
		music_player.stream = null
	if is_instance_valid(audio_player):
		audio_player.stop()
		audio_player.stream = null
