from xml.etree import ElementTree as ET

class IVR(object):

	def __init__(self, name, greet_long, greet_short, invalid_sound, exit_sound, confirm_macro, confirm_key, tts_engine, tts_voice, confirm_attempts, timeout, inter_digit_timeout, max_failures, max_timeouts, digit_len, action_list):
		self.name = name
		self.greet_long = greet_long
		self.greet_short = greet_short
		self.invalid_sound = invalid_sound
		self.exit_sound = exit_sound
		self.confirm_macro = confirm_macro
		self.confirm_key = confirm_key
		self.tts_engine = tts_engine
		self.tts_voice = tts_voice
		self.confirm_attempts = confirm_attempts
		self.timeout = timeout
		self.inter_digit_timeout = inter_digit_timeout
		self.max_failures = max_failures
		self.max_timeouts = max_timeouts
		self.digit_len = digit_len
		self.action_list = action_list

	def write(self, root):
		main_menu = ET.SubElement(root, "menu")
		main_menu.set("name", self.name)
		main_menu.set("greet-long", self.greet_long)
		main_menu.set("greet-short", self.greet_short)
		main_menu.set("invalid-sound", self.invalid_sound)
		main_menu.set("exit-sound", self.exit_sound)
		main_menu.set("confirm-macro", self.confirm_macro)
		main_menu.set("confirm-key", self.confirm_key)
		main_menu.set("tts-engine", self.tts_engine)
		main_menu.set("tts-voice", self.tts_voice)
		main_menu.set("confirm-attempts", self.confirm_attempts)
		main_menu.set("timeout", self.timeout)
		main_menu.set("inter-digit-timeout", self.inter_digit_timeout)
		main_menu.set("max-failures", self.max_failures)
		main_menu.set("max-timeouts", self.max_timeouts)
		main_menu.set("digit-len", self.digit_len)
		for action in self.action_list:
			action_tag = ET.SubElement(main_menu, "entry")
			action_tag.set("action", action.get("action"))
			action_tag.set("digits", action.get("digits"))
			action_tag.set("params", action.get("params"))