const canvas = document.getElementById("screen-canvas");
var FileSaver = require('file-saver');
canvas.height = 144;
canvas.width = 160;

const ctx = canvas.getContext('2d');

emulator = null;
emulatorPaused = false;
emulatorSpeedup = false;
displayDebugInfo = true;

// Init the emulator and load from WASM
const runEmulator = (data, isRomfile, isSavefile) => {
  import("./node_modules/gb-emulator-web/gb_emulator_web.js").then((em) => {
    const ctx = canvas.getContext('2d');

    emulator = em.EmulatorWrapper.new();
    if (isRomfile) {
      emulator.load_rom(data);
    }
    else if (isSavefile) {
      emulator.load_save(data);
    }

    initAudio();
    initInputs();
    renderLoop(emulator)
  });
}

// Main render loop
const renderLoop = () => {
  var framesRun = 0;
  if (!emulatorPaused) {
    if (!emulatorSpeedup) {
      while (emulator.run_until_frontend_event() != 0) {
        buffer = emulator.get_sound_queue();
        pushAudioSamples(buffer);
      }
      framesRun++;
    }
    else {
      // Run in speedup mode.
      // Allow emulator to run as much as it can during one frametime
      const start = performance.now();
      var delta = 0;
      while (delta <= (1.0/60.0)*1000) {
        emulator.run_until_frontend_event();
        delta = (performance.now() - start);
        framesRun++;
      }
    }

    pixels = new Uint8ClampedArray(emulator.get_screen_bitmap());
    const imageData = new ImageData(pixels, canvas.width, canvas.height);
    ctx.putImageData(imageData, 0, 0);

  }
  requestAnimationFrame(renderLoop);
  debugInfo.update(framesRun);
};

// Inputs
function initInputs() {
  // Add keypress listeners
  window.addEventListener("keydown", keyDownInputEvent, true);
  window.addEventListener("keyup", keyUpInputEvent, true);
  // Add mobile button listeners
  button_bindings = [
    ["btn-arrow-left", "KeyA"], 
    ["btn-arrow-right", "KeyD"], 
    ["btn-arrow-up", "KeyW"],
    ["btn-arrow-down", "KeyS"], 
    ["btn-a", "Space"], 
    ["btn-b", "ShiftLeft"], 
    ["btn-start", "Enter"], 
    ["btn-select", "Backspace"]
  ];
  for (const button_binding of button_bindings) {
    const button_id = button_binding[0];
    const button_keycode = button_binding[1];
    document.getElementById(button_id).addEventListener("mousedown", (event) => keyDownMobileEvent(event, button_keycode));
    document.getElementById(button_id).addEventListener("mouseup", (event) => keyUpMobileEvent(event, button_keycode));
    document.getElementById(button_id).addEventListener("touchstart", (event) => keyDownMobileEvent(event, button_keycode));
    document.getElementById(button_id).addEventListener("touchend", (event) => keyUpMobileEvent(event, button_keycode));
  }
}

function keyDownInputEvent(event) {
    if (event.defaultPrevented) {
      return; // Do nothing if event already handled
    }

    handleKeyDown(event.code)
  
    // Consume the event so it doesn't get handled twice
    event.preventDefault();
}

function keyDownMobileEvent(event, keycode) {
  if (event.defaultPrevented) {
    return; // Do nothing if event already handled
  }

  handleKeyDown(keycode)

  // Consume the event so it doesn't get handled twice
  event.preventDefault();
}

function handleKeyDown(keycode) {
  console.log("Keydown: ", keycode);
  switch(keycode) {
    // Gameboy controls
    case "KeyS":
    case "ArrowDown":
      emulator.press_key_down();
      break;
    case "KeyW":
    case "ArrowUp":
      emulator.press_key_up();
      break;
    case "KeyA":
    case "ArrowLeft":
      emulator.press_key_left();
      break;
    case "KeyD":
    case "ArrowRight":
      emulator.press_key_right();
      break;
    case "KeyZ":
    case "Space":
      emulator.press_key_a();
      break;
    case "KeyX":
    case "ShiftLeft":
      emulator.press_key_b();
      break;
    case "Enter":
      emulator.press_key_start();
      break;
    case "Backspace":
      emulator.press_key_start();
      break;
    // Emulator state controls
    case "KeyP":
      emulatorPaused = !emulatorPaused;
      break;
    case "KeyM":
      displayDebugInfo = !displayDebugInfo;
      break;
    case "ControlLeft":
      emulatorSpeedup = !emulatorSpeedup;
      break;
    case "KeyN":
      emulatorPaused = true;
      saveEmulatorToFile('gbsave');
      emulatorPaused = false;
      break;
  }
}

function keyUpInputEvent(event) {
  if (event.defaultPrevented) {
    return; // Do nothing if event already handled
  }

  handleKeyUp(event.code)

  // Consume the event so it doesn't get handled twice
  event.preventDefault();
}

function keyUpMobileEvent(event, keycode) {
  if (event.defaultPrevented) {
    return; // Do nothing if event already handled
  }

  handleKeyUp(keycode)

  // Consume the event so it doesn't get handled twice
  event.preventDefault();
}

function handleKeyUp(keycode) {
  console.log("Keyup: ", keycode);
  switch(keycode) {
    case "KeyS":
    case "ArrowDown":
      emulator.clear_key_down();
      break;
    case "KeyW":
    case "ArrowUp":
      emulator.clear_key_up();
      break;
    case "KeyA":
    case "ArrowLeft":
      emulator.clear_key_left();
      break;
    case "KeyD":
    case "ArrowRight":
      emulator.clear_key_right();
      break;
    case "KeyZ":
    case "Space":
      emulator.clear_key_a();
      break;
    case "KeyX":
    case "ControlLeft":
      emulator.clear_key_b();
      break;
    case "Enter":
      emulator.clear_key_start();
      break;
    case "Backspace":
      emulator.clear_key_start();
      break;
  }
}

function loadFileToEmulator(file) {
  // Check that extension is .gb or .bin
  var isRomfile = file.name.endsWith('.gb') || file.name.endsWith('.bin');
  var isSavefile = file.name.endsWith('.save');
  if (!isRomfile && !isSavefile){ 
    console.log("Error: File type is not .gb or .save")
    return; 
  }

  fileData = new Blob([file]);
  var promise = new Promise(getFileBuffer(fileData));
  promise.then(function(data) {
    runEmulator(data, isRomfile, isSavefile);
  }).catch(function(err) {
    console.log('Error: ',err);
  });
  // access files via fileList
}

// File related code

function dropFile(event) {
  event.stopPropagation();
  event.preventDefault();

  var fileList = event.dataTransfer.files;
  const file = fileList[0];
  loadFileToEmulator(file);
}

function dragOverFile(event) {
  event.stopPropagation();
  event.preventDefault();
  event.dataTransfer.dropEffect = 'copy';
}

function getFileBuffer(fileData) {
  return function(resolve) {
      var reader = new FileReader();
      reader.readAsArrayBuffer(fileData);
      reader.onload = function() {
        var arrayBuffer = reader.result
        var bytes = new Uint8Array(arrayBuffer);
        resolve(bytes);
    }
  }
}

var dropZone = document.getElementById("main");
dropZone.addEventListener("dragover", dragOverFile, false);
dropZone.addEventListener("drop"    , dropFile, false);

dropZone.addEventListener("drop"    , dropFile, false);

var input = document.getElementById('file-input');

input.onchange = e => { 
   var file = e.target.files[0]; 
   loadFileToEmulator(file);
}

function saveEmulatorToFile(filename) {
  var isoDateString = new Date().toISOString().split(".")[0];
  data = new Uint8ClampedArray(emulator.save());
  var blob = new Blob([data], {type: "data:application/octet-stream"});
  FileSaver.saveAs(blob, filename+isoDateString+".save");
}

// Audio related code

let audioContext = null;
let audioStartTimestamp = null;
let audioDelay = 0.05;
let i = 0;
let currentSampleIndex = 0;


// Push audio samples to the audio queue
// This uses AudioNodeBuffers
function pushAudioSamples(sampleBuffer) {
  var audioBuffer = audioContext.createBuffer(1, 1024, 48000);
  var pcmBuffer = audioBuffer.getChannelData(0);
  for (let i = 0; i < audioBuffer.length; i++) {
    pcmBuffer[i] = sampleBuffer[i]
  }
  var source = audioContext.createBufferSource();
  source.buffer = audioBuffer;
  source.connect(audioContext.destination);
  // start the source playing
  currentTime = performance.now();
  playbackTime = currentSampleIndex * 1024/48000.0 + audioDelay;
  actualTime = performance.now() - audioStartTimestamp;
  if (actualTime > playbackTime*1000) {
    console.log("Audio falling behind! Creating audio gap");
    var offset = actualTime/1000.0 - playbackTime + 0.1;
    audioDelay += offset;
    playbackTime += offset;
  }
  if (currentSampleIndex % 60 == 0) {
    console.log("Audio delay: ", playbackTime*1000 - actualTime);
  }
  source.start(playbackTime);
  source.stop(playbackTime+1024/48000.0);
  currentSampleIndex += 1;
}

// Init the audio context
function initAudio() {
  audioContext = new AudioContext();
  audioStartTimestamp = performance.now();
  console.log("Audio Latency: ", audioContext.baseLatency);
}

// Class for displaying various debug info
const debugInfo = new class {
  constructor() {
    this.fps = document.getElementById("debug-info");
    this.frames = [];
    this.lastFrameTimeStamp = performance.now();
  }

  update(multiplier) {
    if (displayDebugInfo) {
      // Convert the delta time since the last frame render into a measure
      // of frames per second.
      const now = performance.now();
      const delta = now - this.lastFrameTimeStamp;
      this.lastFrameTimeStamp = now;
      const fps = 1 / delta * 1000;

      // Save only the latest 100 timings.
      this.frames.push(fps);
      if (this.frames.length > 100) {
        this.frames.shift();
      }

      let sum = 0;
      for (let i = 0; i < this.frames.length; i++) {
        sum += this.frames[i];
      }
      let mean = sum / this.frames.length;

      // Render the statistics.
      this.fps.textContent = `FPS: ${Math.round(fps)*multiplier}, mean: ${Math.round(mean)*multiplier}`.trim();
      }
      else {
        this.fps.textContent = "";
      }
    }
};