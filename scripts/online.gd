class_name SkyPopOnline
extends RefCounted

const SITE := "skypop.quietbytehq.uk"
const URL := "https://skypop.quietbytehq.uk/api/submit"
const MIN_SCORE := 1000
const MAX_SCORE := 1000000

static func qualifying(score: int) -> bool:
	return score >= MIN_SCORE and score <= MAX_SCORE and score % 50 == 0

static func clean_name(raw: String) -> String:
	var result := ""
	for character in raw.to_upper():
		if character in "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789 ._-":
			result += character
	return result.strip_edges().left(12)

static func payload(player: String, score: int) -> Dictionary:
	return {"name": clean_name(player), "score": score, "mode": "adventure", "version": str(ProjectSettings.get_setting("application/config/version", ""))}

static func submit(host: Node, player: String, score: int, endpoint: String = URL) -> void:
	if not qualifying(score):
		return
	var request := HTTPRequest.new()
	request.timeout = 5.0
	request.use_threads = true
	host.add_child(request)
	request.request_completed.connect(func(_result, _code, _headers, _body): request.queue_free())
	if request.request(endpoint, PackedStringArray(["Content-Type: application/json"]), HTTPClient.METHOD_POST, JSON.stringify(payload(player, score))) != OK:
		request.queue_free()
