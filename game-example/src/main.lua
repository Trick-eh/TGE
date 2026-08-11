local sheet
local player_id
local game_root = "game-example/"

function on_start()
	--

	local score = engine.get_persisted("score") or 0

	-- bindings
	engine.bind_action("move_up", "up")
	engine.bind_action("move_down", "down")
	engine.bind_action("move_left", "left")
	engine.bind_action("move_right", "right")
	engine.bind_action("rotate_left", "d")
	engine.bind_action("rotate_right", "a")

	-- assets
	local texture = engine.load_texture(game_root .. "assets/test.png")
	sheet = engine.create_sprite_sheet(texture, 90, 90)

	engine.load_sound("rotate", game_root .. "assets/rotate.mp3")
	engine.load_music("bg", game_root .. "assets/background.mp3")

	-- camera
	engine.set_camera(0, 0, 1.0)

	-- static sprites
	-- local e0 = engine.spawn_entity({
	-- 	transform = {
	-- 		x = 100,
	-- 		y = 50,
	-- 		scale_x = 200,
	-- 		scale_y = 100,
	-- 		rotation = math.pi / 4,
	-- 	},
	-- })
	-- engine.set_sprite(e0, sheet, 1)

	local e1 = engine.spawn_entity({
		transform = {
			x = 100,
			y = 0,
			scale_x = 200,
			scale_y = 100,
			rotation = math.pi / 4,
		},
	})
	engine.set_sprite(e1, sheet, 0)

	local e2 = engine.spawn_entity({
		transform = {
			x = 300,
			y = 200,
			scale_x = 50,
			scale_y = 50,
			rotation = math.pi / 5 + 1,
		},
	})
	engine.set_sprite(e2, sheet, 1)

	local e3 = engine.spawn_entity({
		transform = {
			x = -100,
			y = -200,
			scale_x = 50,
			scale_y = 100,
			rotation = math.pi / 3,
		},
	})
	engine.set_sprite(e3, sheet, 2)

	-- player (animated)
	player_id = engine.spawn_entity({
		transform = {
			x = 0,
			y = 0,
			scale_x = 100,
			scale_y = 100,
			rotation = 0,
		},
		velocity = { x = 120, y = 120 },
		player = true,
	})
	engine.set_animated_sprite(player_id, sheet, { 0, 1, 2, 3 }, 0.5, true)

	-- music
	engine.set_music_volume(0.2)
	engine.play_music("bg")
end

function on_update(dt)
	local score = (engine.get_persisted("score") or 0) + 1
	engine.persist("score", score)

	local pos = engine.get_position(player_id)
	local vel_x = 120
	local vel_y = 120

	if engine.is_action_held("move_up") then
		engine.set_position(player_id, pos.x, pos.y + vel_y * dt)
	end
	if engine.is_action_held("move_down") then
		engine.set_position(player_id, pos.x, pos.y - vel_y * dt)
	end

	-- re-fetch position after potential y change
	pos = engine.get_position(player_id)

	if engine.is_action_held("move_left") then
		engine.set_position(player_id, pos.x - vel_x * dt, pos.y)
	end
	if engine.is_action_held("move_right") then
		engine.set_position(player_id, pos.x + vel_x * dt, pos.y)
	end

	local rot = engine.get_rotation(player_id)
	if engine.is_action_held("rotate_left") then
		engine.set_rotation(player_id, rot - 0.10 * dt)
		engine.play_sound("rotate")
	end
	if engine.is_action_held("rotate_right") then
		engine.set_rotation(player_id, rot + 0.10 * dt)
		engine.play_sound("rotate")
	end
end

function on_stop()
	print("stopped")
end
