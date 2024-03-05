// ========== Global Application State ==========

let g_pending_frames = 0;
let g_frames_since_last_fps_count = 0;
let g_rendered_frames = [];

let g_last_frame_sample_count = 44100 / 60; // Close-ish enough
let g_audio_samples_buffered = 0;
let g_new_frame_sample_threshold = 4096; // under which we request a new frame
let g_audio_overrun_sample_threshold = 8192; // over which we *drop* samples

let g_game_checksum = -1;

let g_screen_buffers = [];
let g_piano_roll_buffers = [];
let g_next_free_buffer_index = 0;
let g_last_rendered_buffer_index = 0;
let g_total_buffers = 16;

let g_frameskip = 0;
let g_frame_delay = 0;

let g_audio_confirmed_working = false;
let g_profiling_results = {};

let g_trouble_detector = {
  successful_samples: 0,
  failed_samples: 0,
  frames_requested: 0,
  trouble_count: 0,
  got_better_count: 0,
}

let g_increase_frameskip_threshold = 0.01; // percent of missed samples
let g_decrease_frameskip_headroom = 1.5 // percent of the time taken to render one frame

let embed_autostart_url = null;

// ========== Init which does not depend on DOM ========

for (let i = 0; i < g_total_buffers; i++) {
  // Allocate a good number of screen buffers
  g_screen_buffers[i] = new ArrayBuffer(256*240*4);
  g_piano_roll_buffers[i] = new ArrayBuffer(480*270*4);
}

// ========== Worker Setup and Utility ==========

var worker = new Worker('emu_worker.js');

function rpc(task, args) {
  return new Promise((resolve, reject) => {
    const channel = new MessageChannel();
    channel.port1.onmessage = ({data}) => {
      if (data.error) {
        reject(data.error);
      } else {
        resolve(data.result);
      }
    };
    worker.postMessage({"type": "rpc", "func": task, "args": args}, [channel.port2]);
  });
}

worker.onmessage = function(e) {
  if (e.data.type == "init") {
    onready();
  }
  if (e.data.type == "deliverFrame") {
    if (e.data.panels.length > 0) {
      g_rendered_frames.push(e.data.panels);
      for (let panel of e.data.panels) {
        if (panel.id == "screen") {
          g_screen_buffers[g_last_rendered_buffer_index] = panel.image_buffer;
        }
        if (panel.id == "piano_roll_window") {
          g_piano_roll_buffers[g_last_rendered_buffer_index] = panel.image_buffer;
        }
      }
      g_last_rendered_buffer_index += 1;
      if (g_last_rendered_buffer_index >= g_total_buffers) {
        g_last_rendered_buffer_index = 0;
      }
      g_frames_since_last_fps_count += 1;
    }
    g_pending_frames -= 1;
    if (g_audio_samples_buffered < g_audio_overrun_sample_threshold) {
      g_nes_audio_node.port.postMessage({"type": "samples", "samples": e.data.audio_buffer});
      g_audio_samples_buffered += e.data.audio_buffer.length;
      g_last_frame_sample_count = e.data.audio_buffer.length;
    } else {
      // Audio overrun, we're running too fast! Drop these samples on the floor and bail.
      // (This can happen in fastforward mode.)
    }
    if (g_rendered_frames.length > 3) {
      // Frame rendering running behing, dropping one frame
      g_rendered_frames.shift(); // and throw it away
    }
  }
  if (e.data.type == "reportPerformance") {
    g_profiling_results[e.data.event] = e.data.average_milliseconds;
  }
}

function render_profiling_results() {
  let results = "";
  for (let event_name in g_profiling_results) {
    let time = g_profiling_results[event_name].toFixed(2);
    results += `${event_name}: ${time}\n`
  }
  var results_box = document.querySelector("#profiling-results");
  if (results_box != null) {
    results_box.innerHTML = results;
  }
}

function automatic_frameskip() {
  // first off, do we have enough profiling data collected?
  if (g_trouble_detector.frames_requested >= 60) {
    let audio_fail_percent = g_trouble_detector.failed_samples / g_trouble_detector.successful_samples;
    if (g_frameskip < 2) {
      // if our audio context is running behind, let's try
      // rendering fewer frames to compensate
      if (audio_fail_percent > g_increase_frameskip_threshold) {
        g_trouble_detector.trouble_count += 1;
        g_trouble_detector.got_better_count = 0;
        console.log("Audio failure percentage: ", audio_fail_percent);
        console.log("Trouble count incremented to: ", g_trouble_detector.trouble_count);
        if (g_trouble_detector.trouble_count > 3) {
          // that's quite enough of that
          g_frameskip += 1;
          g_trouble_detector.trouble_count = 0;
          console.log("Frameskip increased to: ", g_frameskip);
          console.log("Trouble reset")
        }
      } else {
        // Slowly recover from brief trouble spikes
        // without taking action
        if (g_trouble_detector.trouble_count > 0) {
          g_trouble_detector.trouble_count -= 1;
          console.log("Trouble count relaxed to: ", g_trouble_detector.trouble_count);
        }
      }
    }
    if (g_frameskip > 0) {
      // Perform a bunch of sanity checks to see if it looks safe to
      // decrease frameskip.
      if (audio_fail_percent < g_increase_frameskip_threshold) {
        // how long would it take to render one frame right now?
        let frame_render_cost = g_profiling_results.render_all_panels;
        let cost_with_headroom = frame_render_cost * g_decrease_frameskip_headroom;
        // Would a full render reliably fit in our idle time?
        if (cost_with_headroom < g_profiling_results.idle) {
          console.log("Frame render costs: ", frame_render_cost);
          console.log("With headroom: ", cost_with_headroom);
          console.log("Idle time currently: ", g_profiling_results.idle);
          g_trouble_detector.got_better_count += 1;
          console.log("Recovery count increased to: ", g_trouble_detector.got_better_count);
        }
        if (cost_with_headroom > g_profiling_results.idle) {
          if (g_trouble_detector.got_better_count > 0) {
            g_trouble_detector.got_better_count -= 1;
            console.log("Recovery count decreased to: ", g_trouble_detector.got_better_count);
          }
        }
        if (g_trouble_detector.got_better_count >= 10) {
          g_frameskip -= 1;
          console.log("Performance recovered! Lowering frameskip by 1 to: ");
          g_trouble_detector.got_better_count = 0;
        }
      }
    }

    // now reset the counters for the next run
    g_trouble_detector.frames_requested = 0;
    g_trouble_detector.failed_samples = 0;
    g_trouble_detector.successful_samples = 0;
  }
}

// ========== Audio Setup ==========

let g_audio_context = null;
let g_nes_audio_node = null;

async function init_audio_context() {
  g_audio_context = new AudioContext({
    latencyHint: 'interactive',
    sampleRate: 44100,
  });
  await g_audio_context.audioWorklet.addModule('audio_processor.js');
  g_nes_audio_node = new AudioWorkletNode(g_audio_context, 'nes-audio-processor');
  g_nes_audio_node.connect(g_audio_context.destination);
  g_nes_audio_node.port.onmessage = handle_audio_message;
}

function handle_audio_message(e) {
  if (e.data.type == "samplesPlayed") {
    g_audio_samples_buffered -= e.data.count;
    g_trouble_detector.successful_samples += e.data.count;
    if (!g_audio_confirmed_working && g_trouble_detector.successful_samples > 44100) {
      let audio_context_banner = document.querySelector("#audio-context-warning");
      if (audio_context_banner != null) {
        audio_context_banner.classList.remove("active");
      }
      g_audio_confirmed_working = true;
    }
  }
  if (e.data.type == "audioUnderrun") {
    g_trouble_detector.failed_samples += e.data.count;
  }
}

// ========== Main ==========

async function onready() {
  // Initialize audio context, this will also begin audio playback
  await init_audio_context();

  // Initialize everything else
  init_ui_events();
  initializeButtonMappings();

  // Kick off the events that will drive emulation
  requestAnimationFrame(renderLoop);
  // run the scheduler as often as we can. It will frequently decide not to schedule things, this is fine.
  //window.setInterval(schedule_frames_at_top_speed, 1);
  window.setTimeout(sync_to_audio, 1);
  window.setInterval(compute_fps, 1000);
  window.setInterval(render_profiling_results, 1000);
  window.setInterval(automatic_frameskip, 1000);
  window.setInterval(save_sram_periodically, 10000);

  // Attempt to load a cartridge by URL, if one is provided
  let params = new URLSearchParams(location.search.slice(1));
  if (params.get("cartridge")) {
    load_cartridge_by_url(params.get("cartridge"));
    display_banner(params.get("cartridge"));
  }
  if (params.get("tab")) {
    switchToTab(params.get("tab"));
  }
  if (embed_autostart_url != null) {
    load_cartridge_by_url(embed_autostart_url);
  }
}

function init_ui_events() {
  // Setup UI events
  document.getElementById('file-loader').addEventListener('change', load_cartridge_by_file, false);

  var buttons = document.querySelectorAll("#main_menu button");
  buttons.forEach(function(button) {
    button.addEventListener("click", clickTab);
  });

  window.addEventListener("click", function() {
    // Needed to play audio in certain browsers, notably Chrome, which restricts playback until user action.
    g_audio_context.resume();
  });

  document.querySelector("#playfield").addEventListener("dblclick", enterFullscreen);
  document.addEventListener("fullscreenchange", handleFullscreenSwitch);
  document.addEventListener("webkitfullscreenchange", handleFullscreenSwitch);
  document.addEventListener("mozfullscreenchange", handleFullscreenSwitch);
  document.addEventListener("MSFullscreenChange", handleFullscreenSwitch);

  if (document.querySelector("#piano_roll_window")) {
    document.querySelector("#piano_roll_window").addEventListener("click", handle_piano_roll_window_click);
  }

  register_touch_button("#button_a");
  register_touch_button("#button_b");
  register_touch_button("#button_ab");
  register_touch_button("#button_select");
  register_touch_button("#button_start");
  register_d_pad("#d_pad");
  initialize_touch("#playfield");
}

// ========== Cartridge Management ==========

async function load_cartridge(cart_data) {
  console.log("Attempting to load cart with length: ", cart_data.length);
  await rpc("load_cartridge", [cart_data]);
  console.log("Cart data loaded?");
  
  g_game_checksum = crc32(cart_data);
  load_sram();
  let power_light = document.querySelector("#power_light #led");
  power_light.classList.add("powered");
}

function load_cartridge_by_file(e) {
  if (g_game_checksum != -1) {
    save_sram();
  }
  var file = e.target.files[0];
  if (!file) {
    return;
  }
  var reader = new FileReader();
  reader.onload = function(e) {
    cart_data = new Uint8Array(e.target.result);
    load_cartridge(cart_data);
    hide_banners();
  }
  reader.readAsArrayBuffer(file);

  // we're done with the file loader; unfocus it, so keystrokes are captured
  // by the game instead
  this.blur();
}

function load_cartridge_by_url(url) {
  if (g_game_checksum != -1) {
    save_sram();
  }
  var rawFile = new XMLHttpRequest();
  rawFile.overrideMimeType("application/octet-stream");
  rawFile.open("GET", url, true);
  rawFile.responseType = "arraybuffer";
  rawFile.onreadystatechange = function() {
    if (rawFile.readyState === 4 && rawFile.status == "200") {
      console.log(rawFile.responseType);
      cart_data = new Uint8Array(rawFile.response);
      load_cartridge(cart_data);
    }
  }
  rawFile.send(null);
}

// ========== Emulator Runtime ==========

function schedule_frames_at_top_speed() {
  if (g_pending_frames < 10) {
    requestFrame();
  }
  window.setTimeout(schedule_frames_at_top_speed, 1);
}

function sync_to_audio() {
  // On mobile browsers, sometimes window.setTimeout isn't called often enough to reliably
  // queue up single frames; try to catch up by up to 4 of them at once.
  for (i = 0; i < 4; i++) {
    // Never, for any reason, request more than 10 frames at a time. This prevents
    // the message queue from getting flooded if the emulator can't keep up.
    if (g_pending_frames < 10) {
      const actual_samples = g_audio_samples_buffered;
      const pending_samples = g_pending_frames * g_last_frame_sample_count;
      if (actual_samples + pending_samples < g_new_frame_sample_threshold) {
        requestFrame();
      }
    }
  }
  window.setTimeout(sync_to_audio, 1);
}

function requestFrame() {
  updateTouchKeys();
  g_trouble_detector.frames_requested += 1;
  let active_tab = document.querySelector(".tab_content.active").id;
  if (g_frame_delay > 0) {
    // frameskip: advance the emulation, but do not populate or render
    // any panels this time around
    worker.postMessage({"type": "requestFrame", "p1": keys[1] | touch_keys[1], "p2": keys[2] | touch_keys[2], "panels": []});
    g_frame_delay -= 1;
    g_pending_frames += 1;
    return;
  }
  if (active_tab == "jam") {
    worker.postMessage(
      {"type": "requestFrame", "p1": keys[1] | touch_keys[1], "p2": keys[2] | touch_keys[2], "panels": [
        {
          "id": "screen", 
          "target_element": "#jam_pixels",
          "dest_buffer": g_screen_buffers[g_next_free_buffer_index],
        },
        {
          "id": "piano_roll_window", 
          "target_element": "#piano_roll_window",
          "dest_buffer": g_piano_roll_buffers[g_next_free_buffer_index],
        },
      ]},
      [
        g_screen_buffers[g_next_free_buffer_index], 
        g_piano_roll_buffers[g_next_free_buffer_index]
      ]
    );
  } else {
    worker.postMessage(
      {"type": "requestFrame", "p1": keys[1] | touch_keys[1], "p2": keys[2] | touch_keys[2], "panels": [
        {
          "id": "screen", 
          "target_element": "#pixels",
          "dest_buffer": g_screen_buffers[g_next_free_buffer_index],
        }
      ]},
      [g_screen_buffers[g_next_free_buffer_index]]
    );
  }
  g_pending_frames += 1;
  g_next_free_buffer_index += 1;
  if (g_next_free_buffer_index >= g_total_buffers) {
    g_next_free_buffer_index = 0;
  }
  g_frame_delay = g_frameskip;
}

function renderLoop() {
  if (g_rendered_frames.length > 0) {
    for (let panel of g_rendered_frames.shift()) {
      const typed_pixels = new Uint8ClampedArray(panel.image_buffer);
      // TODO: don't hard-code the panel size here
      let rendered_frame = new ImageData(typed_pixels, panel.width, panel.height);
      canvas = document.querySelector(panel.target_element);
      ctx = canvas.getContext("2d", { alpha: false });
      ctx.putImageData(rendered_frame, 0, 0);
      ctx.imageSmoothingEnabled = false;
    }
  }

  requestAnimationFrame(renderLoop);
}

// ========== SRAM Management ==========

// CRC32 checksum generating functions, yanked from this handy stackoverflow post and modified to work with arrays:
// https://stackoverflow.com/questions/18638900/javascript-crc32
// Used to identify .nes files semi-uniquely, for the purpose of saving SRAM
var makeCRCTable = function(){
    var c;
    var crcTable = [];
    for(var n =0; n < 256; n++){
        c = n;
        for(var k =0; k < 8; k++){
            c = ((c&1) ? (0xEDB88320 ^ (c >>> 1)) : (c >>> 1));
        }
        crcTable[n] = c;
    }
    return crcTable;
}

var crc32 = function(byte_array) {
    var crcTable = window.crcTable || (window.crcTable = makeCRCTable());
    var crc = 0 ^ (-1);

    for (var i = 0; i < byte_array.length; i++ ) {
        crc = (crc >>> 8) ^ crcTable[(crc ^ byte_array[i]) & 0xFF];
    }

    return (crc ^ (-1)) >>> 0;
};

async function load_sram() {
  if (await rpc("has_sram")) {
    try {
      var sram_str = window.localStorage.getItem(g_game_checksum);
      if (sram_str) {
        var sram = JSON.parse(sram_str);
        await rpc("set_sram", [sram]);
        console.log("SRAM Loaded!", g_game_checksum);
      }
    } catch(e) {
      console.log("Local Storage is probably unavailable! SRAM saving and loading will not work.");
    }
  }
}

async function save_sram() {
  if (await rpc("has_sram")) {
    try {
      var sram_uint8 = await rpc("get_sram", [sram]);
      // Make it a normal array
      var sram = [];
      for (var i = 0; i < sram_uint8.length; i++) {
        sram[i] = sram_uint8[i];
      }
      window.localStorage.setItem(g_game_checksum, JSON.stringify(sram));
      console.log("SRAM Saved!", g_game_checksum);
    } catch(e) {
      console.log("Local Storage is probably unavailable! SRAM saving and loading will not work.");
    }
  }
}

function save_sram_periodically() {
  save_sram();
}

// ========== User Interface ==========

// This runs *around* once per second, ish. It's fine.
function compute_fps() {
  let counter_element = document.querySelector("#fps-counter");
  if (counter_element != null) {
    counter_element.innerText = "FPS: " + g_frames_since_last_fps_count;
  }
  g_frames_since_last_fps_count = 0;
}

function clearTabs() {
  var buttons = document.querySelectorAll("#main_menu button");
  buttons.forEach(function(button) {
    button.classList.remove("active");
  });

  var tabs = document.querySelectorAll("div.tab_content");
  tabs.forEach(function(tab) {
    tab.classList.remove("active");
  });
}

function switchToTab(tab_name) {
  tab_elements = document.getElementsByName(tab_name);
  if (tab_elements.length == 1)  {
    clearTabs();
    tab_elements[0].classList.add("active");
    content_element = document.getElementById(tab_name);
    content_element.classList.add("active");
  }
}

function clickTab() {
  let tabName = this.getAttribute("name");
  if (tabName == "fullscreen") {
    switchToTab("playfield");
    enterFullscreen();
  } else {
    switchToTab(tabName); 
  }
}

function enterFullscreen() {
  var viewport = document.querySelector("#playfield");
  if (viewport.requestFullscreen) {
    viewport.requestFullscreen();
  } else if (viewport.mozRequestFullScreen) {
    viewport.mozRequestFullScreen();
  } else if (viewport.webkitRequestFullscreen) {
    viewport.webkitRequestFullscreen();
  }
}

function handleFullscreenSwitch() {
  if (document.fullscreenElement || document.mozFullScreenElement || document.webkitFullscreenElement || document.msFullscreenElement) {
    console.log("Entering fullscreen...");
    // Entering fullscreen
    var viewport = document.querySelector("#playfield");
    viewport.classList.add("fullscreen");
    viewport.classList.remove("horizontal");
    viewport.classList.remove("vertical");
    if (is_touch_detected) {
      viewport.classList.add("touchscreen");
    } else {
      viewport.classList.remove("touchscreen");
    }

    setTimeout(function() {
      var viewport = document.querySelector("#playfield");

      var viewport_width = viewport.clientWidth;
      var viewport_height = viewport.clientHeight;

      var canvas_container = document.querySelector("#playfield div.canvas_container");
      if ((viewport_width / 256) * 240 > viewport_height) {
        var target_height = viewport_height;
        var target_width = target_height / 240 * 256;
        canvas_container.style.width = target_width + "px";
        canvas_container.style.height = target_height + "px";
        viewport.classList.add("horizontal");
      } else {
        var target_width = viewport_width;
        var target_height = target_width / 256 * 240;
        canvas_container.style.width = target_width + "px";
        canvas_container.style.height = target_height + "px";
        viewport.classList.add("vertical");
      }
    }, 100);
  } else {
    // Exiting fullscreen
    console.log("Exiting fullscreen...");
    var viewport = document.querySelector("#playfield");
    var canvas_container = document.querySelector("#playfield div.canvas_container");
    viewport.classList.remove("fullscreen");
    canvas_container.style.width = "";
    canvas_container.style.height = "";
  }
}

function hide_banners() {
  banner_elements = document.querySelectorAll(".banner");
  banner_elements.forEach(function(banner) {
    banner.classList.remove("active");
  });
}

function display_banner(cartridge_name) {
  hide_banners();
  banner_elements = document.getElementsByName(cartridge_name);
  if (banner_elements.length == 1)  {
    banner_elements[0].classList.add("active");
  }
}

function handle_piano_roll_window_click(e) {
  rpc("handle_piano_roll_window_click", [e.offsetX / 2, e.offsetY / 2]).then();
}