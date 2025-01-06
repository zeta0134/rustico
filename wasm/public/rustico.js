var rustico = {};

const AUDIO_STUCK_THRESHOLD_MS = 1000 * 2;
const DESIRED_SAMPLERATE = 44100;

// Bits for actually running the emulator.
// TODO: extract this whooole thing into a separate module and make
// at least a pseudo-API as you go. As you work: comment above each
// function that you want to be UI-accessible.
var g_worker;

let g_pending_frames = 0;
let g_frames_since_last_fps_count = 0;
let g_rendered_frames = [];

let g_last_frame_sample_count = DESIRED_SAMPLERATE / 60; // Close-ish enough. Will be overwritten almost right away.
let g_audio_samples_buffered = 0;
let g_new_frame_sample_threshold = 4096; // under which we request a new frame
let g_audio_overrun_sample_threshold = 8192; // over which we *drop* samples

let g_screen_target_element = null;
let g_piano_roll_target_element = null;

let g_screen_buffers = [];
let g_piano_roll_buffers = [];
let g_next_free_buffer_index = 0;
let g_last_rendered_buffer_index = 0;
let g_total_buffers = 16;

let g_frameskip = 0;
let g_frame_delay = 0;

let g_trouble_detector = {
  successful_samples: 0,
  failed_samples: 0,
  frames_requested: 0,
  trouble_count: 0,
  got_better_count: 0,
}

let g_profiling_results = {};

let g_increase_frameskip_threshold = 0.01; // percent of missed samples
let g_decrease_frameskip_headroom = 1.5 // percent of the time taken to render one frame

let g_audio_context = null;
let g_nes_audio_node = null;
let g_audio_last_packet = 0;

let g_p1_keys = 0;
let g_p2_keys = 0;

let g_game_checksum = -1;

for (let i = 0; i < g_total_buffers; i++) {
  // Allocate a good number of screen buffers
  g_screen_buffers[i] = new ArrayBuffer(256*240*4);
  g_piano_roll_buffers[i] = new ArrayBuffer(480*270*4);
}

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
    g_worker.postMessage({"type": "rpc", "func": task, "args": args}, [channel.port2]);
  });
}

async function init_audio_context() {
  g_audio_context = new AudioContext({
    latencyHint: 'interactive',
    sampleRate: DESIRED_SAMPLERATE,
  });
  await g_audio_context.audioWorklet.addModule('rustico_audio_processor.js');
  g_nes_audio_node = new AudioWorkletNode(g_audio_context, 'nes-audio-processor');
  g_nes_audio_node.connect(g_audio_context.destination);
  g_nes_audio_node.port.onmessage = handle_audio_message;
}

function handle_audio_message(e) {
  if (e.data.type == "samplesPlayed") {
    g_audio_samples_buffered -= e.data.count;
    g_trouble_detector.successful_samples += e.data.count;
    g_audio_last_packet = Date.now();
  }
  if (e.data.type == "audioUnderrun") {
    g_trouble_detector.failed_samples += e.data.count;
  }
}

function sync_to_audio() {
  // On mobile browsers, sometimes window.setTimeout isn't called often enough to reliably
  // queue up single frames; try to catch up by up to 4 of them at once.
  for (let i = 0; i < 4; i++) {
    // Never, for any reason, request more than 10 frames at a time. This prevents
    // the message queue from getting flooded if the emulator can't keep up.
    if (g_pending_frames < 10) {
      const actual_samples = g_audio_samples_buffered;
      const pending_samples = g_pending_frames * g_last_frame_sample_count;
      if (actual_samples + pending_samples < g_new_frame_sample_threshold) {
        request_frame();
      }
    }
  }
  window.setTimeout(sync_to_audio, 1);
}

function request_frame() {
  // updateTouchKeys();
  g_trouble_detector.frames_requested += 1;
  if (g_frame_delay > 0) {
    // frameskip: advance the emulation, but do not populate or render
    // any panels this time around
    g_worker.postMessage({"type": "requestFrame", "p1": g_p1_keys, "p2": g_p2_keys, "panels": []});
    g_frame_delay -= 1;
    g_pending_frames += 1;
    return;
  }
  let panels_to_render = [];
  let buffers_to_transfer = [];
  if (g_screen_target_element !== null) {
    panels_to_render.push({
      "id": "screen", 
        "target_element": g_screen_target_element,
        "dest_buffer": g_screen_buffers[g_next_free_buffer_index],
    });
    buffers_to_transfer.push(g_screen_buffers[g_next_free_buffer_index]);
  }
  if (g_piano_roll_target_element !== null) {
    panels_to_render.push({
      "id": "piano_roll_window", 
        "target_element": g_piano_roll_target_element,
        "dest_buffer": g_piano_roll_buffers[g_next_free_buffer_index],
    });
    buffers_to_transfer.push(g_piano_roll_buffers[g_next_free_buffer_index]);
  }
  g_worker.postMessage({
    "type": "requestFrame", 
    "p1": g_p1_keys, 
    "p2": g_p2_keys, 
    "panels": panels_to_render
  }, buffers_to_transfer);
  
  g_pending_frames += 1;
  g_next_free_buffer_index += 1;
  if (g_next_free_buffer_index >= g_total_buffers) {
    g_next_free_buffer_index = 0;
  }
  g_frame_delay = g_frameskip;
}

function render_loop() {
  if (g_rendered_frames.length > 0) {
    for (let panel of g_rendered_frames.shift()) {
      const typed_pixels = new Uint8ClampedArray(panel.image_buffer);
      // TODO: don't hard-code the panel size here
      let rendered_frame = new ImageData(typed_pixels, panel.width, panel.height);
      let canvas = document.querySelector(panel.target_element);
      let ctx = canvas.getContext("2d", { alpha: false });
      ctx.putImageData(rendered_frame, 0, 0);
      ctx.imageSmoothingEnabled = false;
    }
  }

  requestAnimationFrame(render_loop);
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
        //console.log("SRAM Loaded!", g_game_checksum);
      }
    } catch(e) {
      console.log("Local Storage is probably unavailable! SRAM saving and loading will not work.");
    }
  }
}

async function save_sram() {
  if (g_game_checksum != -1) {
    if (await rpc("has_sram")) {
      try {
        var sram_uint8 = await rpc("get_sram", [sram]);
        // Make it a normal array
        var sram = [];
        for (var i = 0; i < sram_uint8.length; i++) {
          sram[i] = sram_uint8[i];
        }
        window.localStorage.setItem(g_game_checksum, JSON.stringify(sram));
        //console.log("SRAM Saved!", g_game_checksum);
      } catch(e) {
        console.log("Local Storage is probably unavailable! SRAM saving and loading will not work.");
      }
    }
  }
}

function save_sram_periodically() {
  save_sram();
}

async function emu_onready() {
  await init_audio_context();

  window.setTimeout(sync_to_audio, 1);
  window.setInterval(automatic_frameskip, 1000);
  window.setInterval(save_sram_periodically, 10000);

  requestAnimationFrame(render_loop);
}

// Note: simply sets up the emulator's basic run context. Rustico boots with a default
// cartridge, so we choose not to perform any fancy autoloading behavior or other nonsense
// here. Call / Create those API functions as needed.
rustico.init = function() {
  const initPromise = new Promise((resolve, reject) => {
    g_worker = new Worker('rustico_worker.js');
    g_worker.onmessage = function(e) {
      if (e.data.type == "init") {
        emu_onready().then(resolve);
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
  });
  return initPromise;  
}

rustico.set_active_panels = function (screen_target_element, piano_roll_target_element) {
  g_screen_target_element = screen_target_element;
  g_piano_roll_target_element = piano_roll_target_element
}

rustico.set_p1_keys = function (key_state) {
  g_p1_keys = key_state;
}

rustico.set_p2_keys = function(key_state) {
  g_p2_keys = key_state;
}

rustico.load_cartridge = async function(cart_data) {
  save_sram();
  //console.log("Attempting to load cart with length: ", cart_data.length);
  await rpc("load_cartridge", [cart_data]);
  //console.log("Cart data loaded?");
  
  g_game_checksum = crc32(cart_data);
  load_sram();
}

rustico.try_to_start_audio = function() {
  g_audio_context.resume();
}

// Returns a simple string that echoes the true run state
// of the emulator. May indicate failure states such as "audio-stuck"
rustico.run_status = function() {
  let time_since_last_audio_packet = Date.now() - g_audio_last_packet;
  if (time_since_last_audio_packet > AUDIO_STUCK_THRESHOLD_MS) {
    return "audio-stuck";
  }
  // For now, the only other status is just "running" since we haven't implemented
  // pause or debug states.
  return "running";
}

rustico.set_volume = function(desiredVolume) {
  g_nes_audio_node.port.postMessage({"type": "volume", "volume": desiredVolume});
}

rustico._intentionally_break_audio = function() {
  g_audio_context.suspend();
}

export default rustico;
