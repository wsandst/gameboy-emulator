const canvas = document.getElementById("screen-canvas");
var FileSaver = require('file-saver');

const enableCachedSaveCompression = false;

canvas.height = 144;
canvas.width = 160;

const ctx = canvas.getContext('2d');

emulator = null;
emulatorPaused = false;
emulatorSpeedup = false;
emulatorAudio = true;
emulatorRunning = false;
displayDebugInfo = false;
bootRomData = null;
romFilename = null;
audioInitiated = false;

// Compatability stuff
var AudioContext = window.AudioContext // Default
    || window.webkitAudioContext; // IOs


let isIOS = /iPad|iPhone|iPod/.test(navigator.platform)
|| (navigator.platform === 'MacIntel' && navigator.maxTouchPoints > 1)

// Fix to make divs and such clickable on Safari on Iphone
if (isIOS) {
  document.querySelector('html').classList.add('is-ios');
}

mostRecentSaveExists = window.localStorage.getItem('mostRecentSave') != null;

// Init the emulator and load from WASM
const startEmulator = (data, isRomfile, isSavefile, isStrSavefile) => {
  if (!audioInitiated) {
    initAudio();
  }
  import("./node_modules/gb-emulator-web/gb_emulator_web.js").then((em) => {
    const ctx = canvas.getContext('2d');

    emulator = em.EmulatorWrapper.new();
    // Use bootrom if loaded
    if (bootRomData != null) {
      console.log("Trying to load: ", bootRomData);
      emulator.load_bootrom(bootRomData);
    }
    if (isRomfile) {
      console.log("Trying to load rom");
      emulator.load_rom(data);
      emulator.set_rom_name(romFilename);

    }
    else if (isSavefile) {
      if (isStrSavefile) { // Format used by local cache saves
        emulator.load_save_str(data)
      }
      else {
        emulator.load_save(data);
      }
    }

    // If the emulator is already running we just overwrite the emulator object
    // We don't need to reinit everything
    if (!emulatorRunning) {
      console.log("Starting emulator");
      initInputs();
      console.log("Starting to render");
      emulatorRunning = true;
      renderLoop(emulator)
    }
  });
}

// Main render loop
const renderLoop = () => {
  var framesRun = 0;
  if (!emulatorPaused) {
    if (!emulatorSpeedup) {
      while (emulator.run_until_frontend_event() != 0) {
        if (emulatorAudio) {
          buffer = emulator.get_sound_queue();
          pushAudioSamples(buffer);
        }
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
  // Gameboy keys
  buttonBindings = [
    ["btn-arrow-left", "KeyA"], 
    ["btn-arrow-right", "KeyD"], 
    ["btn-arrow-up", "KeyW"],
    ["btn-arrow-down", "KeyS"], 
    ["btn-a", "Space"], 
    ["btn-b", "ShiftLeft"], 
    ["btn-start", "Enter"], 
    ["btn-select", "Backspace"]
  ];
  for (const buttonBinding of buttonBindings) {
    const buttonId = buttonBinding[0];
    const buttonKeycode = buttonBinding[1];
    // These are all needed for nice mobile controls
    document.getElementById(buttonId).addEventListener("mousedown", (event) => keyDownMobileEvent(event, buttonKeycode));
    document.getElementById(buttonId).addEventListener("mouseup", (event) => keyUpMobileEvent(event, buttonKeycode));
    document.getElementById(buttonId).addEventListener("touchstart", (event) => keyDownMobileEvent(event, buttonKeycode));
    document.getElementById(buttonId).addEventListener("touchmove", (event) => keyDownMobileEvent(event, buttonKeycode));
    document.getElementById(buttonId).addEventListener("touchend", (event) => keyUpMobileEvent(event, buttonKeycode));
    document.getElementById(buttonId).addEventListener("touchcancel", (event) => keyUpMobileEvent(event, buttonKeycode));
  }
  // Top row of other emulator control buttons
  document.getElementById("btn-turbo").addEventListener("click", (event) => emulatorSpeedup = !emulatorSpeedup);
  document.getElementById("btn-pauseplay").addEventListener("click", (event) => emulatorPaused = !emulatorPaused);
  document.getElementById("btn-save").addEventListener("click", (event) => saveEmulatorToFile());
  document.getElementById("btn-audio").addEventListener("click", (event) => emulatorAudio = !emulatorAudio);
  console.log("Initiated inputs")
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
  //console.log("Keydown: ", keycode);
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
      emulator.press_key_select();
      break;
    // Emulator state controls
    case "KeyP":
      emulatorPaused = !emulatorPaused;
      break;
    case "KeyM":
      toggleDebugDisplay();
      break;
    case "ControlLeft":
      emulatorSpeedup = !emulatorSpeedup;
      break;
    case "KeyN":
      emulatorPaused = true;
      saveEmulatorToFile();
      emulatorPaused = false;
      break;
  }
}

function toggleDebugDisplay() {
  displayDebugInfo = !displayDebugInfo;
  if (displayDebugInfo) {
    document.getElementById("debug-info").style.display = "block";
  }
  else {
    document.getElementById("debug-info").style.display = "none";
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
  //console.log("Keyup: ", keycode);
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

// File related code
function loadFileToEmulator(file) {
  // Check that extension is .gb or .bin
  var isRomfile = file.name.endsWith('.gb') || file.name.endsWith('.bin');
  var isSavefile = file.name.endsWith('.save');
  if (!isRomfile && !isSavefile){ 
    console.log("Error: File type is not .gb or .save")
    return; 
  }

  romFilename = file.name.split(".")[0];

  fileData = new Blob([file]);
  loadRomDataToEmulator(fileData, isRomfile, isSavefile);
}

// Takes a blob input
function loadRomDataToEmulator(fileData, isRomfile, isSavefile) {
  var promise = new Promise(getFileBuffer(fileData));
  promise.then(function(data) {
    startEmulator(data, isRomfile, isSavefile, false);
  }).catch(function(err) {
    console.log('Error: ',err);
  });
}

function loadBootRomToEmulator(file) {
    // Check that extension is .gb or .bin
    var isRomfile = file.name.endsWith('.gb') || file.name.endsWith('.bin') || file.name.endsWith('.boot') || file.name.endsWith('.bootrom');
  
    fileData = new Blob([file]);
    var promise = new Promise(getFileBuffer(fileData));
    promise.then(function(data) {
      if (data.length == 256) {
        bootRomData = data;
        displayPopupMessage("✔️ BootROM loaded", 3000);
      }
      else {
        displayPopupMessage("❌ Error loading BootROM: Invalid size!", 3000);
      }
    }).catch(function(err) {
      console.log('Error: ',err);
    });
}

function loadServersideRomFile(romname) {
  hideDropdownMobile();
  const url = "roms/"+romname;
  fetch(url).then(function(response) {
    response.blob().then(function(data) {
      loadRomDataToEmulator(data, true, false);
    });
  });
}

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


// Map file loading buttons to correct functions

var romInput = document.getElementById('file-rom-input');

romInput.onchange = e => { 
  hideDropdownMobile()
  var file = e.target.files[0]; 
  loadFileToEmulator(file);
}

var saveInput = document.getElementById('file-save-input');

saveInput.onchange = e =>  {
  hideDropdownMobile()
  var file = e.target.files[0]; 
  loadFileToEmulator(file);
}

var bootromInput = document.getElementById('file-bootrom-input');

bootromInput.onchange = e => { 
  hideDropdownMobile();
  var file = e.target.files[0]; 
  loadBootRomToEmulator(file);
}

document.getElementById('load-demo-blargg-cpu-instrs')

// Functinality for making dropdown menu work on mobile
// Enable dropdown through a click, then disable by placing
// a click handler on the main document and disable dropdown
// if click is outside dropdown

// Is click event inside or outside the dropdown?
const outsideDropdownClickListener = event => {
  var element = document.getElementById("dropdown");
  if (!element.contains(event.target)) {
    hideDropdownMobile();
  }
}


// Hide the dropdown
function hideDropdownMobile() {
  let hasHover = window.matchMedia("(hover: hover)").matches;
  if (!hasHover) {
    for (elem of document.getElementsByClassName("dropdown-hide")) {
      elem.style.display = "none";
    }
    document.removeEventListener('click', outsideDropdownClickListener);
  }
}

// Show the dropdown
function showDropdownMobile(dropdownID) {
  let hasHover = window.matchMedia("(hover: hover)").matches;
  display = document.getElementById(dropdownID).style.display;
  if (!hasHover && (display === "none" || display === "")) {
    // Hide other subdropdowns
    for (elem of document.getElementsByClassName("sub-dropdown-content")) {
      elem.style.display = "none";
    }
    //document.getElementById(dropdownID).classList.remove("dropdown-hide");
    document.getElementById(dropdownID).style.display = "inline-block";
    document.addEventListener('click', outsideDropdownClickListener);
  }
}

document.getElementById('dropbtn').addEventListener("click", () => showDropdownMobile("dropdown-content"));
document.getElementById('sub-dropdown1').addEventListener("click", () => showDropdownMobile("sub-dropdown1-content"));
document.getElementById('sub-dropdown2').addEventListener("click", () => showDropdownMobile("sub-dropdown2-content"));

document.getElementById('load-rom-button').addEventListener("click", () => romInput.click());
document.getElementById('load-save-button').addEventListener("click", () => saveInput.click());
document.getElementById('load-bootrom-button').addEventListener("click", () => bootromInput.click());

// Submenu 1, load serverside demo roms
document.getElementById('load-demo-flappy-boy').addEventListener("click", 
  () => loadServersideRomFile("flappy_boy.gb"));

document.getElementById('load-demo-rex-run').addEventListener("click", 
  () => loadServersideRomFile("rex-run.gb"));

document.getElementById('load-demo-pocket').addEventListener("click", 
  () => loadServersideRomFile("pocket.gb"));

document.getElementById('load-demo-dmgp').addEventListener("click", 
  () => loadServersideRomFile("dmgp-01.gb "));

// Submenu 2, load serverside test roms
document.getElementById('load-test-blargg-cpu-instrs').addEventListener("click", 
  () => loadServersideRomFile("blargg_cpu_instrs.gb"));

document.getElementById('load-test-blargg-instr-timings').addEventListener("click", 
  () => loadServersideRomFile("blargg_instr_timing.gb"));
  
document.getElementById('load-test-acid2').addEventListener("click", 
  () => loadServersideRomFile("acid2.gb"));


function saveEmulatorToFile(filename) {
  var shouldUnpauseEmulator = emulator;
  emulatorPaused = true;
  var filename = emulator.get_rom_name();
  var isoDateString = new Date().toISOString().split(".")[0];
  data = new Uint8ClampedArray(emulator.save());
  var blob = new Blob([data], {type: "data:application/octet-stream"});
  FileSaver.saveAs(blob, filename+isoDateString+".save");

  // Keep most recent save as local storage on user
  // Convert to string because for some reason you can only save strings
  var dataStr = emulator.save_as_str();
  console.log("Saved most recent save to user cache with size of ", dataStr.length, " characters");
  window.localStorage.setItem('mostRecentSave', dataStr);

  displayPopupMessage("✔️ Game saved", 1500);
  if (shouldUnpauseEmulator) {
    emulatorPaused = false;
  }
}

// Enable menu option for most recent save if local memory exists
if (mostRecentSaveExists) {
  var localSaveMenuOption = document.getElementById("load-local-save");
  localSaveMenuOption.className = "dropdown-content-btn";
  localSaveMenuOption.addEventListener("click", () => loadMostRecentSave());
}

// Load the most recent save from local user storage
// This is updated everytime the user saves
function loadMostRecentSave() {
  hideDropdownMobile();
  saveStr = window.localStorage.getItem('mostRecentSave');
  startEmulator(saveStr, false, true, true);
}

// Audio related code

var audioContext = null;
var audioFilterNode = null;
var audioStartTimestamp = null;
var audioDelay = 0.05;
var currentSampleIndex = 0;
var previousAudioNode = null;
var i = 0;

var queuedNodes = []

// Push audio samples to the audio queue
// This uses AudioNodeBuffers
function pushAudioSamples(sampleBuffer) {
  var audioBuffer = audioContext.createBuffer(2, 1024, 48000);
  var leftPcmBuffer = audioBuffer.getChannelData(0);
  var rightPcmBuffer = audioBuffer.getChannelData(1);
  for (let i = 0; i < audioBuffer.length; i++) {
    leftPcmBuffer[i] = sampleBuffer[i*2+0];
    rightPcmBuffer[i] = sampleBuffer[i*2+1];
  }
  var source = audioContext.createBufferSource();
  source.buffer = audioBuffer;
  source.connect(audioFilterNode);
  // start the source playing
  currentTime = performance.now();
  playbackTime = currentSampleIndex * 1024/48000.0 + audioDelay;
  actualTime = performance.now() - audioStartTimestamp;
  const actualAudioDelay = playbackTime*1000-actualTime;
  if (actualAudioDelay <= 0) {
    console.log("Audio falling behind! Creating audio gap");
    var offset = actualTime/1000.0 - playbackTime + 0.1;
    audioDelay += offset;
    playbackTime += offset;
  }
  else if (actualAudioDelay > 150) {
    console.log("Audio too fast! Skipping audio");
    audioDelay -= 0.1;
    playbackTime -= 0.1;
  }
  queuedNodes.push(source);
  //previousAudioNode.onended = startNextNodeClosure(playbackTime, currentSampleIndex);
  source.start(playbackTime); 
  source.stop(playbackTime+1024/48000.0)
  previousAudioNode = source;
  currentSampleIndex += 1;
  //console.log(audioContext.state);
}

// Used to chain nodes using onended
// This might be required for iOS, but sounds terrible for some reason
function startNextNodeClosure(playbackTime, i) {
  return function(e) {
    var node = queuedNodes.shift();
    node.start(playbackTime); 
    //node.stop(playbackTime+1024/48000.0)
  }
}

// Push silence to the audio
function pushAudioSilence(length) {
  var audioBuffer = audioContext.createBuffer(1, length, 48000);
  var pcmBuffer = audioBuffer.getChannelData(0);
  for (let i = 0; i < audioBuffer.length; i++) {
    pcmBuffer[i] = 0;
  }
  var source = audioContext.createBufferSource();
  source.buffer = audioBuffer;
  source.connect(audioFilterNode);
  playbackTime = currentSampleIndex * 1024/48000.0 + audioDelay;
  source.start(playbackTime);
  //source.stop(playbackTime+length/48000.0);
  previousAudioNode = source;
  currentSampleIndex += 1;
}

// Init the audio context
function initAudio() {
  audioContext = new AudioContext();
  audioStartTimestamp = performance.now();

  // Apply lowpass filter ontop
  audioFilterNode = audioContext.createBiquadFilter();
  audioFilterNode.connect(audioContext.destination);
  audioFilterNode.type = "highpass";
  audioFilterNode.frequency.value = 200;

  pushAudioSilence(1024);
  audioInitiated = true;
  console.log("Audio Latency: ", audioContext.baseLatency);
  console.log("Initiated audio")
}

// Class for displaying various debug info
const debugInfo = new class {
  constructor() {
    this.fps = document.getElementById("debug-info");
    this.frames = [];
    this.lastFrameTimeStamp = performance.now();
    this.audioDelay = 0;
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

      var currentTime, playbackTime, actualTime;
      if (currentSampleIndex % 30 == 0) {
        currentTime = performance.now();
        playbackTime = currentSampleIndex * 1024/48000.0 + audioDelay;
        actualTime = performance.now() - audioStartTimestamp;
        this.audioDelay = playbackTime*1000 - actualTime;
      }

      // Render the statistics.
      this.fps.textContent = `FPS: ${Math.round(fps)*multiplier}, mean: ${Math.round(mean)*multiplier}. Audio delay: ${Math.round(this.audioDelay)}`.trim();
      }
      else {
        this.fps.textContent = "";
      }
    }
};

// Functions for displaying small popup message
function displayPopupMessage(message, duration) {
  console.log("Displaying popup message: ", message);
  document.getElementById("popup-message").style.visibility = 'visible';
  document.getElementById("popup-message").style.opacity = '1';
  document.getElementById("popup-message-content").textContent = message;
  setTimeout(fadeOutPopupMessage, duration);
}

function fadeOutPopupMessage() {
  document.getElementById("popup-message").style.visibility = 'hidden';
  document.getElementById("popup-message").style.opacity = '0';
}

//document.getElementById("load-local-save").style.

// Log console output to screen
/*(function () {
  displayPopupMessage("", 10000000);
  var old = console.log;
  var logger = document.getElementById('popup-message-content');
  console.log = function (message) {
      if (typeof message == 'object') {
          logger.innerHTML += (JSON && JSON.stringify ? JSON.stringify(message) : message) + '<br />';
      } else {
          logger.innerHTML += message + '<br />';
      }
  }
})();*/
