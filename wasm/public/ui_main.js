function load_cartridge_by_url(url) {
  save_sram();
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

async function onready() {
  await emu_init();
  set_active_panels("#testId", null);

  document.querySelector("#activateAudio").addEventListener("click", try_to_start_audio);

  load_cartridge_by_url("tactus.nes");
}

document.addEventListener("DOMContentLoaded", onready);