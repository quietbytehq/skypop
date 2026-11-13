extends Node

const SAVE_PATH := "user://skypop_save.cfg"
const SECTION := "progress"
const LEADERBOARD_SECTION := "leaderboard"
const LEADERBOARD_KEY := "top_ten"
const LEADERBOARD_LIMIT := 10
const SETTINGS_SECTION := "settings"


func save_progress(level: int, data: Dictionary = {}) -> Error:
	var config := _load_config()
	config.set_value(SECTION, "level", clampi(level, 1, 10))
	config.set_value(SECTION, "data", data)
	return config.save(SAVE_PATH)


func load_progress() -> Dictionary:
	var config := _load_config()
	return {
		"level": clampi(int(config.get_value(SECTION, "level", 1)), 1, 10),
		"data": config.get_value(SECTION, "data", {})
	}


func get_high_scores() -> Array:
	var config := _load_config()
	var entries: Variant = config.get_value(LEADERBOARD_SECTION, LEADERBOARD_KEY, [])
	if not entries is Array:
		return []
	var clean_entries: Array = []
	for entry in entries:
		if entry is Dictionary:
			clean_entries.append({
				"name": str(entry.get("name", "PLAYER")).left(12),
				"score": maxi(int(entry.get("score", 0)), 0)
			})
	clean_entries.sort_custom(func(a: Dictionary, b: Dictionary) -> bool: return int(a["score"]) > int(b["score"]))
	return clean_entries.slice(0, LEADERBOARD_LIMIT)


func add_high_score(player_name: String, score: int) -> Error:
	var config := _load_config()
	var entries := get_high_scores()
	entries.append({
		"name": player_name.strip_edges().left(12) if not player_name.strip_edges().is_empty() else "PLAYER",
		"score": maxi(score, 0)
	})
	entries.sort_custom(func(a: Dictionary, b: Dictionary) -> bool: return int(a["score"]) > int(b["score"]))
	config.set_value(LEADERBOARD_SECTION, LEADERBOARD_KEY, entries.slice(0, LEADERBOARD_LIMIT))
	return config.save(SAVE_PATH)


func get_audio_settings() -> Dictionary:
	var config := _load_config()
	return {
		"music_enabled": bool(config.get_value(SETTINGS_SECTION, "music_enabled", true)),
		"music": clampf(float(config.get_value(SETTINGS_SECTION, "music_volume", 0.35)), 0.0, 1.0),
		"effects": clampf(float(config.get_value(SETTINGS_SECTION, "effects_volume", 0.8)), 0.0, 1.0)
	}


func save_audio_settings(music_volume: float, effects_volume: float) -> Error:
	var config := _load_config()
	config.set_value(SETTINGS_SECTION, "music_volume", clampf(music_volume, 0.0, 1.0))
	config.set_value(SETTINGS_SECTION, "effects_volume", clampf(effects_volume, 0.0, 1.0))
	return config.save(SAVE_PATH)


func get_input_mappings() -> Dictionary:
	var config := _load_config()
	var mappings: Variant = config.get_value(SETTINGS_SECTION, "input_mappings", {})
	return mappings if mappings is Dictionary else {}


func save_input_mapping(action_name: String, physical_keycode: int) -> Error:
	var config := _load_config()
	var mappings := get_input_mappings()
	mappings[action_name] = physical_keycode
	config.set_value(SETTINGS_SECTION, "input_mappings", mappings)
	return config.save(SAVE_PATH)


func _load_config() -> ConfigFile:
	var config := ConfigFile.new()
	config.load(SAVE_PATH)
	return config


func save_music_enabled(enabled: bool) -> Error:
	var config := _load_config()
	config.set_value(SETTINGS_SECTION, "music_enabled", enabled)
	return config.save(SAVE_PATH)
