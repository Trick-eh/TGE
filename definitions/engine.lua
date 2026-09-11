---@meta engine

---@class engine
engine = {}

---Binds an action name to an input binding string
---@param action string The action name e.g. "jump"
---@param binding string The key/button e.g. "space", "south", "left"
engine.bind_action = function(action, binding) end

---Removes all bindings for an action
---@param action string
engine.unbind_action = function(action) end

---Returns true if the action is currently held
---@param action string
---@return boolean
engine.is_action_held = function(action) end

---Returns true if the action was pressed this frame
---@param action string
---@return boolean
engine.is_action_pressed = function(action) end

---Returns true if the action was released this frame
---@param action string
---@return boolean
engine.is_action_released = function(action) end

---@return { x: number, y: number }
engine.mouse_position = function() end

---@return { x: number, y: number }
engine.scroll_delta = function() end

---@param key string
---@return boolean
engine.is_key_held = function(key) end

---@param key string
---@return boolean
engine.is_key_pressed = function(key) end

---@param key string
---@return boolean
engine.is_key_released = function(key) end

---@param button string "left"|"right"|"middle"|"back"|"forward"
---@return boolean
engine.is_mouse_button_held = function(button) end

---@param button string
---@return boolean
engine.is_mouse_button_pressed = function(button) end

---@param button string
---@return boolean
engine.is_mouse_button_released = function(button) end

---@param button string "south"|"north"|"east"|"west"|"lbumper"|"rbumper"|"ltrigger"|"rtrigger"|"start"|"select"|"dpad_up"|"dpad_down"|"dpad_left"|"dpad_right"|"lstick"|"rstick"
---@return boolean
engine.is_gamepad_button_held = function(button) end

---@param button string
---@return boolean
engine.is_gamepad_button_pressed = function(button) end

---@param button string
---@return boolean
engine.is_gamepad_button_released = function(button) end

---@param axis string "left_x"|"left_y"|"right_x"|"right_y"|"left_trigger"|"right_trigger"
---@return number
engine.gamepad_axis = function(axis) end

---Loads a sound from a file path. Call from on_start only.
---@param name string The name to reference this sound by
---@param path string File path to the audio file
engine.load_sound = function(name, path) end

---Loads music from a file path. Call from on_start only.
---@param name string
---@param path string
engine.load_music = function(name, path) end

---@param name string
engine.play_sound = function(name) end

---@param name string
---@param volume number 0.0 to 1.0
engine.play_sound_with_volume = function(name, volume) end

---@param name string
engine.play_music = function(name) end

engine.pause_music = function() end
engine.resume_music = function() end
engine.stop_music = function() end

---@param volume number 0.0 to 1.0
engine.set_master_volume = function(volume) end

---@param volume number
engine.set_sfx_volume = function(volume) end

---@param volume number
engine.set_music_volume = function(volume) end

---Loads a texture from a file path. Call from on_start only.
---@param path string
---@return number TextureHandle ID
engine.load_texture = function(path) end

---Creates a sprite sheet from a texture. Call from on_start only.
---@param texture_id number TextureHandle ID from load_texture
---@param tile_width number
---@param tile_height number
---@return number SpriteSheetHandle ID
engine.create_sprite_sheet = function(texture_id, tile_width, tile_height) end

---Loads a font from a file path at a given rasterization size. Call from on_start only.
---@param path string
---@param size number
---@return number FontHandle ID
engine.load_font = function(path, size) end

---Draws text at a world position with the given RGBA color. Call from on_render only.
---@param text string
---@param font_id number FontHandle ID from load_font
---@param x number
---@param y number
---@param r number 0.0 to 1.0
---@param g number 0.0 to 1.0
---@param b number 0.0 to 1.0
---@param a number 0.0 to 1.0
engine.draw_text = function(text, font_id, x, y, r, g, b, a) end

---Measures the pixel size a string of text would occupy with the given font
---@param text string
---@param font_id number FontHandle ID from load_font
---@return { x: number, y: number }
engine.measure_text = function(text, font_id) end

---Draws a solid-colored rectangle centered at (x, y). Call from on_render or on_background.
---@param x number
---@param y number
---@param width number
---@param height number
---@param r number 0.0 to 1.0
---@param g number 0.0 to 1.0
---@param b number 0.0 to 1.0
---@param a number 0.0 to 1.0
engine.draw_rect = function(x, y, width, height, r, g, b, a) end

---Spawns a new entity with the given components
---@param components { transform?: { x: number, y: number, scale_x: number, scale_y: number, rotation: number }, velocity?: { x: number, y: number }, player?: boolean }
---@return number Entity ID
engine.spawn_entity = function(components) end

---@param entity_id number
engine.despawn = function(entity_id) end

---@param entity_id number
---@return { x: number, y: number }|nil
engine.get_position = function(entity_id) end

---@param entity_id number
---@param x number
---@param y number
engine.set_position = function(entity_id, x, y) end

---@param entity_id number
---@return { x: number, y: number }|nil
engine.get_scale = function(entity_id) end

---@param entity_id number
---@param x number
---@param y number
engine.set_scale = function(entity_id, x, y) end

---@param entity_id number
---@return number|nil Rotation in radians
engine.get_rotation = function(entity_id) end

---@param entity_id number
---@param radians number
engine.set_rotation = function(entity_id, radians) end

---@param entity_id number
---@return { x: number, y: number }|nil
engine.get_velocity = function(entity_id) end

---Sets an entity's velocity. The entity must have been spawned with a `velocity`
---component already (spawn_entity components.velocity) - calling this on an entity
---that was never given a velocity component is a silent no-op.
---@param entity_id number
---@param x number
---@param y number
engine.set_velocity = function(entity_id, x, y) end

---@param entity_id number
---@param sheet_id number SpriteSheetHandle ID
---@param tile_index number
engine.set_sprite = function(entity_id, sheet_id, tile_index) end

---@param entity_id number
---@param tile_index number
engine.set_sprite_index = function(entity_id, tile_index) end

---@param entity_id number
---@param sheet_id number SpriteSheetHandle ID
---@param frames number[] List of tile indices
---@param frame_duration number Seconds per frame
---@param looping boolean
engine.set_animated_sprite = function(entity_id, sheet_id, frames, frame_duration, looping) end

---@param x number
---@param y number
---@param zoom number
engine.set_camera = function(x, y, zoom) end

---@param entity_id number
---@param key string
---@param value number|boolean|string
engine.set_component = function(entity_id, key, value) end

---@param entity_id number
---@param key string
---@return number|boolean|string|nil
engine.get_component = function(entity_id, key) end

---@param entity_id number
---@param key string
---@return boolean
engine.has_component = function(entity_id, key) end

---@param entity_id number
---@param key string
engine.remove_component = function(entity_id, key) end

---Stores a value that survives Lua hot-reloads (dev mode). Does NOT survive
---process restarts - use save_data/load_data for that.
---@param key string
---@param value number|boolean|string
engine.persist = function(key, value) end

---Retrieves a value previously stored with persist. Returns nil if unset.
---@param key string
---@return number|boolean|string|nil
engine.get_persisted = function(key) end

---Stores a value to disk, associated with this game. Persists across process
---restarts. Call flush happens automatically each frame.
---@param key string
---@param value number|boolean|string
engine.save_data = function(key, value) end

---Retrieves a value previously stored with save_data. Returns nil if unset.
---@param key string
---@return number|boolean|string|nil
engine.load_data = function(key) end

---Pauses or resumes the fixed-timestep simulation clock. While paused, systems
---that check Time.is_paused (currently: velocity integration) stop advancing.
---Does NOT stop on_fixed_update or on_update from being called - it is up to
---the game to decide what should still run while paused.
---@param paused boolean
engine.set_paused = function(paused) end

---Sets the window background color. By default, background color is [r,g,b,a] = [1,1,1,1] (black)
---Only works inside on_start(), on_render() and on_background()
---@param r number
---@param g number
---@param b number
---@param a number
engine.set_background_color = function(r, g, b, a) end

---Begins capturing typed text into an internal buffer (backspace deletes
---the last character). Does NOT affect normal key state -- is_key_held
---etc. keep working independently, so gate other input yourself (e.g.
---pause movement) if typing shouldn't also drive gameplay.
---@param initial string|nil Starting text, defaults to empty
engine.start_text_input = function(initial) end

---Stops capturing. The buffer itself is left intact -- read it with
---get_text_input before stopping if you need the final value.
engine.stop_text_input = function() end

---@return boolean
engine.is_text_input_active = function() end

---@return string
engine.get_text_input = function() end

---@param text string
engine.set_text_input = function(text) end

-- Requests the engine to exit the event loop
engine.request_exit = function() end

return engine
