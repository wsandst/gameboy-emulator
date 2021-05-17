const canvas = document.getElementById("screen-canvas");
canvas.height = 144;
canvas.width = 160;

const ctx = canvas.getContext('2d');


emulator = null;

const runEmulator = (data, is_romfile, is_savefile) => {
  import("./node_modules/gb-emulator-web/gb_emulator_web.js").then((em) => {
    const ctx = canvas.getContext('2d');

    emulator = em.EmulatorWrapper.new();
    if (is_romfile) {
      emulator.load_rom(data);
    }
    else if (is_savefile) {
      emulator.load_save(data);
    }

    window.addEventListener("keydown", keyDownInput, true);
    window.addEventListener("keyup", keyUpInput, true);

    renderLoop(emulator)
  });
}

const renderLoop = () => {
  emulator.run_until_frontend_event()

  pixels = new Uint8ClampedArray(emulator.get_screen_bitmap());
  const imageData = new ImageData(pixels, canvas.width, canvas.height);
  ctx.putImageData(imageData, 0, 0);

  requestAnimationFrame(renderLoop);
};

function dropFile(event) {
  event.stopPropagation();
  event.preventDefault();

  var fileList = event.dataTransfer.files;
  const file = fileList[0];
  // Check that extension is .gb or .bin
  var is_romfile = file.name.endsWith('.gb') || file.name.endsWith('.bin');
  var is_savefile = file.name.endsWith('.save');
  if (!is_romfile && !is_savefile){ 
    console.log("Error: File type is not .gb or .save")
    return; 
  }

  fileData = new Blob([file]);
  var promise = new Promise(getFileBuffer(fileData));
  promise.then(function(data) {
    runEmulator(data, is_romfile, is_savefile);
  }).catch(function(err) {
    console.log('Error: ',err);
  });
  // access files via fileList
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

function keyDownInput(event) {
    if (event.defaultPrevented) {
      return; // Do nothing if event already handled
    }

    switch(event.code) {
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
      case "ControlLeft":
        emulator.press_key_b();
        break;
      case "Enter":
        emulator.press_key_start();
        break;
      case "Backspace":
        emulator.press_key_start();
        break;
    }
  
    // Consume the event so it doesn't get handled twice
    event.preventDefault();
}

function keyUpInput(event) {
  if (event.defaultPrevented) {
    return; // Do nothing if event already handled
  }
  console.log("keyup");
  switch(event.code) {
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

  // Consume the event so it doesn't get handled twice
  event.preventDefault();
}

var dropZone = document.getElementById("main");
dropZone.addEventListener("dragover", dragOverFile, false);
dropZone.addEventListener("drop"    , dropFile, false);