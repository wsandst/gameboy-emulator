const canvas = document.getElementById("screen-canvas");
canvas.height = 144;
canvas.width = 160;

const ctx = canvas.getContext('2d');

rom = null;
emulator = null;

const runEmulator = () => {
  import("./node_modules/gb-emulator-web/gb_emulator_web.js").then((em) => {
    const ctx = canvas.getContext('2d');

    console.log("Trying to load rom");
    emulator = em.EmulatorWrapper.new();
    emulator.load_rom(rom);

    renderLoop(emulator)
  });
}

const renderLoop = () => {
  emulator.run_until_draw()

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
  if (!file.name.endsWith('.gb') && !file.name.endsWith('.bin')){ 
    console.log("Error: File type is not .gb or .bin")
    return; 
  }

  fileData = new Blob([file]);
  var promise = new Promise(getFileBuffer(fileData));
  promise.then(function(data) {
    // Here you can pass the bytes to another function.
    rom = data;
    runEmulator();
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

var dropZone = document.getElementById("main");
dropZone.addEventListener("dragover", dragOverFile, false);
dropZone.addEventListener("drop"    , dropFile, false);