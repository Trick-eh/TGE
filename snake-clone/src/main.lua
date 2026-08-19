--[[
	snake-clone/main.lua

	A reference game for the engine's Lua API. Every major system the engine
	exposes to Lua is used here at least once:

	  - input:      bind_action / is_action_pressed / is_action_held
	  - entities:   spawn_entity / despawn / set_position / set_velocity
	  - sprites:    load_texture / create_sprite_sheet / set_sprite / set_animated_sprite
	  - text:       load_font / draw_text / measure_text
	  - shapes:     draw_rect
	  - audio:      load_sound / play_sound
	  - camera:     set_camera
	  - simulation: set_paused
	  - persistence:save_data / load_data   (survives process restarts)

    Tho it wasn't used here, its great to mention the most useful engine system
    for Lua devs:
      - hot_reload: persist / get_persisted (allows data to survive a hot_reload, 
                                             which is triggered upon saving a change mid execution)

	Lifecycle callbacks the engine calls into, in the order they matter:
	  on_start()          -- once, before the first frame
	  on_update(dt)        -- every frame; dt is the real (unclamped) frame time
	  on_fixed_update(dt)  -- fixed-rate ticks (dt == GameConfig.fixed_timestep);
	                          put anything that must be deterministic here
	  on_background()      -- render, drawn before world sprites
	  on_render()          -- render, drawn after world sprites (UI/overlays)
	  on_stop()            -- once, on window close
]]

local game_root = "snake-clone/"

-- ---------------------------------------------------------------------------
-- Tunables
-- ---------------------------------------------------------------------------

local GRID_W, GRID_H = 20, 20
local CELL_SIZE = 32

-- Movement speed is expressed as "fixed ticks per grid move" rather than a
-- raw speed, because the game logic advances on fixed ticks (see
-- on_fixed_update). Fewer ticks per move = faster snake.

local MAX_TICKS_PER_MOVE = 10
local MIN_TICKS_PER_MOVE = 7
local STEPS_PER_TICKS_DECREASE = 5 -- every N points, shave one tick off the move interval

-- ---------------------------------------------------------------------------
-- Game state
-- ---------------------------------------------------------------------------

local snake = {} -- array of { x, y } grid cells, [1] is the head
local direction = { x = 1, y = 0 } -- current movement direction (locked in for this tick)
local next_direction = { x = 1, y = 0 } -- direction that will apply on the *next* move
local queued_direction = nil -- input received since the last move, applied next move

local food = { x = 0, y = 0 }
local segment_entities = {} -- entity ids, parallel array to `snake`
local food_entity = nil

local tick_count = 0
local alive = true
local paused = false
local score = 0
local high_score = 0

local sheet -- SpriteSheetHandle for snake.png
local font -- FontHandle for UI text
local title_font -- FontHandle for UI Titles

-- ---------------------------------------------------------------------------
-- Grid <-> world space
-- ---------------------------------------------------------------------------

-- The grid is centered on the world origin, so the camera can stay at (0,0).
local function grid_to_world(gx, gy)
	return gx * CELL_SIZE - ((GRID_W - 1) * CELL_SIZE) / 2, gy * CELL_SIZE - ((GRID_H - 1) * CELL_SIZE) / 2
end

-- ---------------------------------------------------------------------------
-- Segment entities
--
-- Every segment is a persistent entity reused across moves (entities are
-- expensive to spawn/despawn relative to just repositioning them). Segments
-- are always spawned WITH a `velocity` component, even though it starts at
-- zero: engine.set_velocity() is a no-op on entities that were never given a
-- velocity component in the first place, so the component has to exist from
-- spawn time or later velocity-based movement below silently does nothing.
-- ---------------------------------------------------------------------------

local function set_segment(index, gx, gy, tile)
	local wx, wy = grid_to_world(gx, gy)

	if segment_entities[index] then
		-- Existing segment: only the sprite (which tile/orientation) is set
		-- here. Position is NOT set directly -- it's driven every fixed tick
		-- by the engine's built-in velocity integration (see
		-- set_segment_velocities below), which is what makes movement glide
		-- between cells instead of teleporting.
		engine.set_sprite(segment_entities[index], sheet, tile)
	else
		local id = engine.spawn_entity({
			transform = {
				x = wx,
				y = wy,
				scale_x = CELL_SIZE,
				scale_y = CELL_SIZE,
				rotation = 0,
			},
			velocity = { x = 0, y = 0 },
		})
		engine.set_sprite(id, sheet, tile)
		segment_entities[index] = id
	end
end

local function trim_segments(count)
	for i = count + 1, #segment_entities do
		engine.despawn(segment_entities[i])
		segment_entities[i] = nil
	end
end

-- ---------------------------------------------------------------------------
-- Food
-- ---------------------------------------------------------------------------

local function spawn_food()
	local function occupied(gx, gy)
		for _, seg in ipairs(snake) do
			if seg.x == gx and seg.y == gy then
				return true
			end
		end
		return false
	end

	local gx, gy
	repeat
		gx = math.random(0, GRID_W - 1)
		gy = math.random(0, GRID_H - 1)
	until not occupied(gx, gy)

	food = { x = gx, y = gy }

	local wx, wy = grid_to_world(gx, gy)
	if food_entity then
		engine.despawn(food_entity)
	end
	food_entity = engine.spawn_entity({
		transform = {
			x = wx,
			y = wy,
			scale_x = CELL_SIZE,
			scale_y = CELL_SIZE,
			rotation = 0,
		},
	})
	-- Animated sprite: alternates between tiles 5 and 6 every 0.3s, looping.
	engine.set_animated_sprite(food_entity, sheet, { 5, 6 }, 0.3, true)
end

-- ---------------------------------------------------------------------------
-- Game setup / reset
-- ---------------------------------------------------------------------------

local function despawn_entities()
	for _, id in ipairs(segment_entities) do
		engine.despawn(id)
	end
	segment_entities = {}
	if food_entity then
		engine.despawn(food_entity)
		food_entity = nil
	end
end

local function init_game()
	queued_direction = nil
	tick_count = 0

	despawn_entities()

	local cx = math.floor(GRID_W / 2)
	local cy = math.floor(GRID_H / 2)
	snake = {
		{ x = cx, y = cy },
		{ x = cx - 1, y = cy },
		{ x = cx - 2, y = cy },
	}
	direction = { x = 1, y = 0 }
	next_direction = { x = 1, y = 0 }
	alive = true
	paused = false
	score = 0

	spawn_food()
end

-- ---------------------------------------------------------------------------
-- Collision
--
-- `tail_will_move` matters because the tail segment is about to vacate its
-- current cell on any move that isn't a growth move (food not eaten). If the
-- head is moving into that soon-to-be-empty cell, it should NOT count as a
-- collision, the tail won't actually be there anymore once the move
-- completes. On a growth move, the tail stays put (nothing is removed from
-- `snake`), so its cell is still genuinely occupied and must still block.
-- ---------------------------------------------------------------------------

local function check_collision(hx, hy, tail_will_move)
	if hx < 0 or hx >= GRID_W or hy < 0 or hy >= GRID_H then
		return true
	end

	local last = #snake
	for i = 2, last do
		if snake[i].x == hx and snake[i].y == hy and not (tail_will_move and i == last) then
			return true
		end
	end
end

local function on_death()
	engine.play_sound("death")

	if score > high_score then
		high_score = score
		-- save_data persists to disk and survives process restarts, unlike
		-- engine.persist (which only survives Lua hot-reloads during dev mode).
		engine.save_data("high_score", high_score)
	end
end

-- ---------------------------------------------------------------------------
-- Grid-authoritative movement
--
-- This is deliberately instant: `snake`, collisions, and food are all
-- resolved as a single atomic step with no interpolation of their own. The
-- SMOOTH visual glide is a separate concern, handled entirely by
-- set_segment_velocities() below. Keeping grid logic instant and simple
-- means the rules (collision, growth, scoring) can be reasoned about
-- independently of how they're presented on screen.
-- ---------------------------------------------------------------------------

local function move_snake()
	if queued_direction then
		-- Reject reversing directly into yourself.
		if not (queued_direction.x == -direction.x and queued_direction.y == -direction.y) then
			next_direction = queued_direction
		end
		queued_direction = nil
	end
	direction = { x = next_direction.x, y = next_direction.y }

	local head = snake[1]
	local new_head = { x = head.x + direction.x, y = head.y + direction.y }
	local eating = (new_head.x == food.x and new_head.y == food.y)

	if check_collision(new_head.x, new_head.y, not eating) then
		alive = false
		on_death()
		return
	end

	table.insert(snake, 1, new_head)

	if eating then
		score = score + 1
		engine.play_sound("eat")
		spawn_food()
	else
		table.remove(snake) -- tail moves up; not a growth move
	end
end

local function head_tile()
	if direction.x == -1 then
		return 0
	end
	if direction.x == 1 then
		return 1
	end
	if direction.y == -1 then
		return 2
	end
	if direction.y == 1 then
		return 3
	end
end

local function update_visuals()
	for i, seg in ipairs(snake) do
		local tile = 4 -- body
		if i == 1 then
			tile = head_tile()
		end
		set_segment(i, seg.x, seg.y, tile)
	end
	trim_segments(#snake)
end

local function ticks_per_move()
	return math.max(MIN_TICKS_PER_MOVE, MAX_TICKS_PER_MOVE - math.floor(score / STEPS_PER_TICKS_DECREASE))
end

-- ---------------------------------------------------------------------------
-- Visual glide via velocity
--
-- Called exactly once per grid move (not every fixed tick). For each
-- segment, the distance between its pre-move and post-move cell is known
-- ahead of time, so a constant velocity can be computed that will carry it
-- exactly from one cell to the next over the course of `ticks_per_move()`
-- fixed ticks. The engine's own fixed-tick velocity integration then does
-- the rest, every tick, without any further Lua involvement until the next
-- move.
--
-- `engine.set_position` here is a defensive resync, not the primary way
-- position is driven: it snaps each segment to where our own grid bookkeeping
-- says it should already be, so floating point drift can never accumulate
-- across many hops.
-- ---------------------------------------------------------------------------

local function set_segment_velocities(prev_snake, dt)
	local duration = ticks_per_move() * dt

	for i, seg in ipairs(snake) do
		local prev = prev_snake[i]
		if prev then
			local pwx, pwy = grid_to_world(prev.x, prev.y)
			engine.set_position(segment_entities[i], pwx, pwy)

			local dx = (seg.x - prev.x) * CELL_SIZE
			local dy = (seg.y - prev.y) * CELL_SIZE
			engine.set_velocity(segment_entities[i], dx / duration, dy / duration)
		else
			-- Brand new tail segment from growth -- already placed correctly
			-- by set_segment() this tick, nothing to animate toward.
			engine.set_velocity(segment_entities[i], 0, 0)
		end
	end
end

-- ---------------------------------------------------------------------------
-- UI and Background
-- ---------------------------------------------------------------------------

local function draw_game_background()
	local grid_w, grid_h = GRID_W * CELL_SIZE, GRID_H * CELL_SIZE
	local border = 4

	-- draw_rect draws a solid, batched quad centered at (x, y). Because it
	-- shares the same batch as sprites, drawing the whole checkerboard here
	-- costs a handful of GPU draw calls total, not one per cell.
	engine.draw_rect(0, 0, grid_w + border * 2, grid_h + border * 2, 0.8, 0.8, 0.8, 1.0)
	engine.draw_rect(0, 0, grid_w, grid_h, 0.1, 0.3, 0.1, 1.0)

	for i = 1, GRID_H do
		for j = 1, GRID_W do
			if i % 2 == 0 then
				if j % 2 == 1 then
					engine.draw_rect(
						j * CELL_SIZE - (grid_w / 2) - CELL_SIZE / 2,
						i * CELL_SIZE - (grid_h / 2) - CELL_SIZE / 2,
						CELL_SIZE,
						CELL_SIZE,
						0.1,
						0.35,
						0.1,
						1.0
					)
				end
			else
				if j % 2 == 0 then
					engine.draw_rect(
						j * CELL_SIZE - (grid_w / 2) - CELL_SIZE / 2,
						i * CELL_SIZE - (grid_h / 2) - CELL_SIZE / 2,
						CELL_SIZE,
						CELL_SIZE,
						0.1,
						0.35,
						0.1,
						1.0
					)
				end
			end
		end
	end
end

local function show_game_over_screen()
	local gameover_text = "Game Over!"
	local score_text = "You achieved " .. score .. " points!"
	local restart_text = "Press space to restart"

	local gameover_size = engine.measure_text(gameover_text, font)
	local score_size = engine.measure_text(score_text, font)
	local restart_size = engine.measure_text(restart_text, font)

	engine.draw_rect(0, 0, 2000, 1000, 0, 0, 0, 0.8)
	engine.draw_text(gameover_text, title_font, -gameover_size.x, GRID_H * CELL_SIZE / 5, 1, 0, 0, 1)
	engine.draw_text(score_text, font, -score_size.x / 2, -GRID_H * CELL_SIZE / 5, 1, 1, 0, 1)
	engine.draw_text(restart_text, font, -restart_size.x / 2, 0, 1, 0.5, 0, 1)
end

local function show_game_paused_screen()
	local paused_text = "Game paused"
	local continue_text = "Press space to continue playing"

	local paused_size = engine.measure_text(paused_text, font)
	local continue_size = engine.measure_text(continue_text, font)

	engine.draw_rect(0, 0, 2000, 1000, 0, 0, 0, 0.8)
	engine.draw_text(paused_text, title_font, -paused_size.x, GRID_H * CELL_SIZE / 5, 1, 0, 0, 1)
	engine.draw_text(continue_text, font, -continue_size.x / 2, -GRID_H * CELL_SIZE / 5, 1, 0.5, 0, 1)
end

local function show_game_ui()
	local sc_text = "Score: " .. score
	local sc_size = engine.measure_text(sc_text, font)
	engine.draw_text(
		"Score: " .. score,
		font,
		CELL_SIZE * (GRID_W - 1) / 2 - sc_size.x,
		CELL_SIZE * (GRID_H + 1) / 2,
		1,
		1,
		1,
		1
	)

	local hs_text = "Best: " .. math.floor(high_score)
	local hs_size = engine.measure_text(hs_text, font)
	engine.draw_text(
		hs_text,
		font,
		CELL_SIZE * (GRID_W - 2) / 2 - hs_size.x - sc_size.x,
		CELL_SIZE * (GRID_H + 1) / 2,
		1,
		1,
		0,
		1
	)

	local mv_text_1 = "w / up"
	local mv_text_2 = "a / left  |  s / down  |  d / right"
	local mv_size_1 = engine.measure_text(mv_text_1, font)
	local mv_size_2 = engine.measure_text(mv_text_2, font)
	engine.draw_text(mv_text_2, font, -CELL_SIZE * GRID_W / 2, -CELL_SIZE * (GRID_H + 2) / 2, 0.7, 0.7, 0.7, 1)
	engine.draw_text(
		mv_text_1,
		font,
		-CELL_SIZE * GRID_W / 2 + mv_size_2.x / 2 - mv_size_1.x / 2,
		-CELL_SIZE * (GRID_H + 1) / 2,
		0.7,
		0.7,
		0.7,
		1
	)
	local pause_text = "press space to pause"
	local pause_size = engine.measure_text(pause_text, font)
	engine.draw_text(
		pause_text,
		font,
		CELL_SIZE * GRID_W / 2 - pause_size.x,
		-CELL_SIZE * (GRID_H + 1.5) / 2,
		0.7,
		0.7,
		0.7,
		1
	)
end

-- ---------------------------------------------------------------------------
-- Lifecycle
-- ---------------------------------------------------------------------------

function on_start()
	font = engine.load_font(game_root .. "assets/font.ttf", 24)
	title_font = engine.load_font(game_root .. "assets/font.ttf", 48)
	high_score = engine.load_data("high_score") or 0

	math.randomseed(os.time())
	engine.load_sound("eat", game_root .. "assets/eat.wav")
	engine.load_sound("death", game_root .. "assets/death.wav")

	local tex = engine.load_texture(game_root .. "assets/snake.png")
	sheet = engine.create_sprite_sheet(tex, 32, 32)

	engine.set_background_color(0.1, 0.1, 0.1, 1)

	engine.set_camera(0, 0, 1)
	engine.bind_action("up", "up")
	engine.bind_action("up", "w")
	engine.bind_action("down", "down")
	engine.bind_action("down", "s")
	engine.bind_action("left", "left")
	engine.bind_action("left", "a")
	engine.bind_action("right", "right")
	engine.bind_action("right", "d")
	engine.bind_action("restart", "space")
	engine.bind_action("pause", "space")

	engine.set_paused(false)

	init_game()
end

function on_update(dt)
	if not alive then
		if engine.is_action_pressed("restart") then
			init_game()
		end
		return
	end

	-- Pause is toggled on the press edge, not held state, so a single tap
	-- flips it. `engine.set_paused` only freezes systems that check it
	-- (currently: the engine's velocity integration) -- on_fixed_update
	-- still runs while paused, which is why the `paused` gate below is also
	-- needed on the Lua side to stop tick_count from silently advancing.
	if engine.is_action_pressed("pause") then
		paused = not paused
		engine.set_paused(paused)
	end

	if paused then
		return
	end

	if engine.is_action_pressed("up") and direction.y == 0 then
		queued_direction = { x = 0, y = 1 }
	elseif engine.is_action_pressed("down") and direction.y == 0 then
		queued_direction = { x = 0, y = -1 }
	elseif engine.is_action_pressed("left") and direction.x == 0 then
		queued_direction = { x = -1, y = 0 }
	elseif engine.is_action_pressed("right") and direction.x == 0 then
		queued_direction = { x = 1, y = 0 }
	end
end

function on_fixed_update(dt)
	if not alive or paused then
		return
	end

	tick_count = tick_count + 1

	if tick_count >= ticks_per_move() then
		tick_count = 0

		-- Snapshot cell positions before the move so set_segment_velocities
		-- can diff "where each segment index was" against "where it is now".
		local prev_snake = {}
		for i, seg in ipairs(snake) do
			prev_snake[i] = { x = seg.x, y = seg.y }
		end

		move_snake()
		update_visuals()
		set_segment_velocities(prev_snake, dt)
	end
end

function on_render()
	if alive then
		show_game_ui()
		if paused then
			show_game_paused_screen()
		end
	else
		show_game_over_screen()
	end
end

function on_background()
	draw_game_background()
end

function on_stop() end
