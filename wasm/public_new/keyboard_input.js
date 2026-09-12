var keyboard_input = {};

const KEY_A = 1
const KEY_B = 2
const KEY_SELECT = 4
const KEY_START = 8
const KEY_UP = 16
const KEY_DOWN = 32
const KEY_LEFT = 64
const KEY_RIGHT = 128

var keys = [0,0];
var remap_key = false;
var remap_index = 0;
var remap_slot = 1;

var controller_keymaps = [];

var onchange_callback = function() {}

// These are effectively the default input mappings, which the user should be able to remap at will.
controller_keymaps[0] = [
"x",
"z",
"Shift",
"Enter",
"ArrowUp",
"ArrowDown",
"ArrowLeft",
"ArrowRight"];

controller_keymaps[1] = ["-","-","-","-","-","-","-","-"];

// When this key is pressed, un-press the corresponding key. This
// is mostly meant to block invalid D-Pad inputs, while favoring
// the most recent incoming press.
const suppression_mask = [
	0, // KEY_A
	0, // KEY_B
	0, // KEY_SELECT
	0, // KEY_START
	KEY_DOWN, // KEY_UP
	KEY_UP, // KEY_DOWN
	KEY_RIGHT, // KEY_LEFT
	KEY_LEFT, // KEY_RIGHT
];

window.addEventListener('keydown', function(event) {
  if (remap_key) {
    if (event.key != "Escape") {
      controller_keymaps[remap_slot][remap_index] = event.key;
    } else {
      controller_keymaps[remap_slot][remap_index] = "-";
    }
    remap_key = false;
    displayButtonMappings();
    saveInputConfig();
    return;
  }
  for (var c = 0; c <= 1; c++) {
    for (var i = 0; i < 8; i++) {
      if (event.key == controller_keymaps[c][i]) {
        keys[c] = keys[c] | (0x1 << i);
        keys[c] = keys[c] & ~(suppression_mask[i]);
      }
    }
  }
  onchange_callback(keys[0], keys[1]);
});

window.addEventListener('keyup', function(event) {
  for (var c = 0; c <= 1; c++) {
    for (var i = 0; i < 8; i++) {
      if (event.key == controller_keymaps[c][i]) {
        keys[c] = keys[c] & ~(0x1 << i);
      }
    }
  }
  onchange_callback(keys[0], keys[1]);
});

keyboard_input.p1_keys = function() {
	return keys[0];
}

keyboard_input.p2_keys = function() {
	return keys[1];
}

keyboard_input.onchange = function(callback) {
	onchange_callback = callback;
}

export default keyboard_input;
