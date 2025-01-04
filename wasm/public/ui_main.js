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
      console.log(rawFile.responseType);
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

  console.log("new p1 keys", p1_keys);
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
  let desiredTouchPercentage = document.querySelector("#touchSize").value;
  document.querySelectorAll(".touch-overlay-dpad").forEach(function(e) {
    e.style.width = desiredTouchPercentage + "%";
  });
  document.querySelectorAll(".touch-overlay-buttons").forEach(function(e) {
    e.style.width = desiredTouchPercentage + "%";
  });
}

async function onready() {
  await rustico.init();
  rustico.set_active_panels("#testId", null);
  keyboard_input.onchange(set_keys);

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
  document.querySelector("#touchSize").addEventListener("change", resize_touch_controls);
  document.querySelector("#touchSize").addEventListener("input", resize_touch_controls);
  document.querySelector("#touchSize").value = 25; // TODO: load this from settings!

  window.setInterval(update_click_to_play_overlays, 100);

  load_cartridge_by_url("tactus.nes");
}

document.addEventListener("DOMContentLoaded", onready);