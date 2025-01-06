import rustico from "./rustico.js";
import keyboard_input from "./keyboard_input.js";
import touch_input from "./touch_input.js";

// Mostly for ease of debugging
window.rustico = rustico;


// Global UI state
let last_run_state = "";

function load_cartridge_by_url(url) {
  var rawFile = new XMLHttpRequest();
  rawFile.overrideMimeType("application/octet-stream");
  rawFile.open("GET", url, true);
  rawFile.responseType = "arraybuffer";
  rawFile.onreadystatechange = function() {
    if (rawFile.readyState === 4 && rawFile.status == "200") {
      let cart_data = new Uint8Array(rawFile.response);
      rustico.load_cartridge(cart_data);
    }
  }
  rawFile.send(null);
}

function update_click_to_play_overlays() {
  let run_status = rustico.run_status();
  if (run_status == last_run_state) {
    return;
  }
  last_run_state = run_status;
  if (run_status == "audio-stuck") {
    document.querySelectorAll(".canvas-container").forEach(function(element) {
      element.classList.add("inactive");
    });
  } else {
    document.querySelectorAll(".canvas-container").forEach(function(element) {
      element.classList.remove("inactive");
    });
  }
}

function collect_and_set_volume() {
    let desiredVolume = document.querySelector("#volume").value;
    rustico.set_volume(desiredVolume);
}

// TODO: combine several input sources!
function set_keys() {
  let p1_keys = keyboard_input.p1_keys() | touch_input.p1_keys();
  let p2_keys = keyboard_input.p2_keys() | touch_input.p2_keys();
  rustico.set_p1_keys(p1_keys);
  rustico.set_p2_keys(p2_keys);
}

function hide_touch_controls() {
  document.querySelectorAll(".touch-controls-active").forEach(function(e) {
    e.classList.remove("touch-controls-active");
    e.classList.add("touch-controls-inactive");
  });
}

function show_touch_controls() {
  document.querySelectorAll(".touch-controls-inactive").forEach(function(e) {
    e.classList.remove("touch-controls-inactive");
    e.classList.add("touch-controls-active");
  });
}

function toggle_touch_controls() {
  let elementsWithTouchActiveClass = document.querySelectorAll(".touch-controls-active");
  let touchControlsActive = elementsWithTouchActiveClass.length > 0;
  if (touchControlsActive) {
    hide_touch_controls();  
  } else {
    show_touch_controls();
  }
}

function resize_touch_controls() {
  let desiredTouchPercentage = touch_input.touch_controls_size
  document.querySelectorAll(".touch-overlay-dpad").forEach(function(e) {
    e.style.width = desiredTouchPercentage + "%";
  });
  document.querySelectorAll(".touch-overlay-buttons").forEach(function(e) {
    e.style.width = desiredTouchPercentage + "%";
  });
}

function set_main_canvas_size() {
  // Grab the reported width/height of our container element
  let panelRect = document.querySelector("#gameplay-panel").getBoundingClientRect();
  let canvasElement = document.querySelector("#gameplay-panel .canvas-container");
  // Also grab the device pixel ratio for DPI purposes, which we'll try to use to obtain
  // a pixel-perfect canvas size no matter the device
  let devicePixelRatio = window.devicePixelRatio;

  // We want the largest integer multiple, in device pixels, that we can actually accomodate
  let canvasWidthsThatWouldFit = panelRect.width * devicePixelRatio / 256.0;
  let canvasHeightsThatWouldFit = panelRect.height * devicePixelRatio / 240.0;

  // Special case: if either of these is 0, we are on an unusually small device and must cheat
  if (canvasWidthsThatWouldFit < 1.0 || canvasHeightsThatWouldFit < 1.0) {
    // Which axis is constraining us the most?
    if (canvasWidthsThatWouldFit < canvasHeightsThatWouldFit) {
      // Size to the available width
      canvasElement.width = panelRect.width / devicePixelRatio;
      canvasElement.height = (panelRect.width / 256.0) * 240.0 / devicePixelRatio;
    } else {
      // Size to the available height
      canvasElement.height = panelRect.height / devicePixelRatio;
      canvasElement.width = (panelRect.height / 240.0) * 256.0 / devicePixelRatio;
    }
    return;
  }
  // Otherwise, clamp to the smallest integer size, and use that as our scaling value
  let scalingValue = Math.floor(Math.min(canvasWidthsThatWouldFit, canvasHeightsThatWouldFit));
  canvasElement.style.width = (256.0 * scalingValue / devicePixelRatio) + "px";
  canvasElement.style.height = (240.0 * scalingValue / devicePixelRatio) + "px";
}

function closeAllPanels() {
  document.querySelectorAll(".ui-panel").forEach(function(e) {
    e.classList.remove("active");
  });
  document.querySelectorAll(".main-menu button").forEach(function(e) {
    e.classList.remove("active");
  });
}

function switchToPanel(panel, button) {
  closeAllPanels();
  document.querySelector(panel).classList.add("active");
  document.querySelector(button).classList.add("active");
}

function switchToGameplay() {
  switchToPanel("#gameplay-panel", "#switchToGameplay");
  rustico.set_active_panels("#mainGameplayCanvas", null);
  hide_main_menu();
}

function switchToSettings() {
  switchToPanel("#settings-panel", "#switchToSettings");
  rustico.set_active_panels(null, null);
  hide_main_menu();
}

function hide_main_menu() {
  document.querySelector(".main-menu").classList.remove("active");
}

function toggle_main_menu() {
  document.querySelector(".main-menu").classList.toggle("active");
}

function initialize_persistent_settings() {
  document.querySelectorAll(".persistent-setting-string").forEach(function(el) {
    el.value = JSON.parse(window.localStorage.getItem(el.dataset.field));
    el.addEventListener("change", persist_setting_string);
    el.addEventListener("input", persist_setting_string);
  });
  document.querySelectorAll(".persistent-setting-number").forEach(function(el) {
    el.valueAsNumber = JSON.parse(window.localStorage.getItem(el.dataset.field));
    el.addEventListener("change", persist_setting_number);
    el.addEventListener("input", persist_setting_number);
  });
}

function persist_setting_string(e) {
  window.localStorage.setItem(e.target.dataset.field, JSON.stringify(e.target.value));
  apply_settings();
}

function persist_setting_number(e) {
  window.localStorage.setItem(e.target.dataset.field, JSON.stringify(e.target.valueAsNumber));
  apply_settings();
}

function apply_settings() {
  // should we guard specific side effects on a datafield? reloading EVERYTHING
  // might get expensive... but it's not like we change settings often
  touch_input.load_settings();
  resize_touch_controls();
}

async function onready() {
  await rustico.init();
  rustico.set_active_panels("#mainGameplayCanvas", null);
  keyboard_input.onchange(set_keys);

  touch_input.init_settings();
  initialize_persistent_settings();

  touch_input.register_button("#button_a");
  touch_input.register_button("#button_b");
  touch_input.register_button("#button_ab");
  touch_input.register_button("#button_select");
  touch_input.register_button("#button_start");
  touch_input.register_d_pad("#d_pad");
  touch_input.initialize_touch(".touch-overlay-dpad");
  touch_input.initialize_touch(".touch-overlay-buttons");
  touch_input.onchange(set_keys);

  document.querySelector(".canvas-container").addEventListener("click", rustico.try_to_start_audio);
  document.querySelector("#volume").addEventListener("change", collect_and_set_volume);
  document.querySelector("#volume").addEventListener("input", collect_and_set_volume);
  document.querySelector("#volume").value = 100; // TODO: load this from settings!

  document.querySelector("#touchToggle").addEventListener("click", toggle_touch_controls);
  document.querySelector("#menuToggle").addEventListener("click", toggle_main_menu);
  resize_touch_controls();

  document.querySelector("#switchToGameplay").addEventListener("click", switchToGameplay);
  document.querySelector("#switchToSettings").addEventListener("click", switchToSettings);

  set_main_canvas_size();
  window.addEventListener("resize", set_main_canvas_size);

  window.setInterval(update_click_to_play_overlays, 100);

  load_cartridge_by_url("tactus.nes");
}

document.addEventListener("DOMContentLoaded", onready);