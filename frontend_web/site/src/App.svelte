<script>
	/*
	TODO:
		Implement navigation menu
		Implement audio
		Implement debug info
	*/
	import FileSaver from "file-saver"
	
	import Screen from "./Screen.svelte"
	import ControlsTop from "./controls/ControlsTop.svelte"
	import ControlsArrows from "./controls/ControlsArrows.svelte"
	import ControlsAB from "./controls/ControlsAB.svelte"
	import ControlsStartSelect from "./controls/ControlsStartSelect.svelte"
	import Popup from "./Popup.svelte"
	import DebugInfo from "./DebugInfo.svelte"

	let popup;
	let debugInfo;

	export let emulatorLib;
	let emulator;
	let screen;

	let emulatorPaused = false;
	let emulatorSpeedup = false;
	let emulatorAudio = true;
	let emulatorRunning = false;

	const keyBindings = {
		"KeyW": "UP", "ArrowDown": "UP",
		"KeyS": "DOWN", "ArrowDown": "DOWN",
		"KeyA": "LEFT", "ArrowDown": "LEFT",
		"KeyD": "RIGHT", "ArrowDown": "RIGHT",
		"Space" : "A", "KeyZ": "A",
		"KeyX": "B", "ShiftLeft": "B",
		"Enter" : "START",
		"Backspace" : "SELECT",
		"KeyP" : "PAUSE",
		"KeyN" : "SAVE",
		"ControlLeft" : "SAVE",
		"KeyM" : "DEBUG",
	}

	const renderLoop = () => {
		var framesRun = 0;

		if (!emulatorPaused) {
			if (!emulatorSpeedup) {
				while (emulator.run_until_frontend_event() != 0) {
					/*if (emulatorAudio) {
						buffer = emulator.get_sound_queue();
						pushAudioSamples(buffer);
					}*/
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
			let pixels = new Uint8ClampedArray(emulator.get_screen_bitmap())
			screen.update(pixels)
		}
		requestAnimationFrame(renderLoop);
		debugInfo.update(framesRun);
	};

	// File handling

	function getFileBuffer(fileData) {
		return function(resolve) {
			let reader = new FileReader();
			reader.readAsArrayBuffer(fileData);
			reader.onload = function() {
				let arrayBuffer = reader.result;
				let bytes = new Uint8Array(arrayBuffer);
				resolve(bytes);
			}
		}
	}

	function loadFileToEmulator(file) {
		let romFilename = file.name.split(".")[0];
		let isRomfile = file.name.endsWith('.gb') || file.name.endsWith('.bin');
  		let isSavefile = file.name.endsWith('.save');

		let fileData = new Blob([file]);
		let promise = new Promise(getFileBuffer(fileData));
		promise.then(function(data) {
			emulator = emulatorLib.EmulatorWrapper.new();
			if (isRomfile) {
				emulator.load_rom(data);
				emulator.set_rom_name(romFilename);
			}
			else if (isSavefile) {
				emulator.load_save(data);
			}
			if (!emulatorRunning) {
				emulatorRunning = true;
				debugInfo.init();
				renderLoop();
			}
		}).catch(function(err) {
			console.log('Error: ',err);
		});
	}

	function dropFile(event) {
		var fileList = event.dataTransfer.files;
		const file = fileList[0];
		loadFileToEmulator(file);
	}

	function dragOverFile(event) {
		event.dataTransfer.dropEffect = 'copy';
	}

	function saveEmulatorToFile(filename) {
		if (!emulatorRunning) {
			return;
		}
		let shouldUnpauseEmulator = emulator;
		emulatorPaused = true;
		filename = emulator.get_rom_name();
		let isoDateString = new Date().toISOString().split(".")[0];
		let data = new Uint8ClampedArray(emulator.save());
		let blob = new Blob([data], {type: "data:application/octet-stream"});
		FileSaver.saveAs(blob, filename+isoDateString+".save");

		// Keep most recent save as local storage on user
		// Convert to string because for some reason you can only save strings
		let dataStr = emulator.save_as_str();
		console.log("Saved most recent save to user cache with size of ", dataStr.length, " characters");
		window.localStorage.setItem('mostRecentSave', dataStr);

		popup.display("✔️ Game saved", 1500);
		
		if (shouldUnpauseEmulator) {
			emulatorPaused = false;
		}
	}

	// Button handling

	function handleButtonEvent(event) {
		if (event.type == 'down') {
			handleButtonDown(event.detail.text);
		}
		else if (event.type == 'up') {
			handleButtonUp(event.detail.text);
		}
	}

	function handleButtonDown(buttonID) {
		//console.log("Keydown: ", buttonID);
		switch(buttonID) {
			// Gameboy controls
			case "DOWN":
				emulator.press_key_down();
				break;
			case "UP":
				emulator.press_key_up();
				break;
			case "LEFT":
				emulator.press_key_left();
				break;
			case "RIGHT":
				emulator.press_key_right();
				break;
			case "A":
				emulator.press_key_a();
				break;
			case "B":
				emulator.press_key_b();
				break;
			case "START":
				emulator.press_key_start();
				break;
			case "SELECT":
				emulator.press_key_select();
				break;
			// Emulator state controls
			case "PAUSE":
				emulatorPaused = !emulatorPaused;
				break;
			case "DEBUG":
				debugInfo.toggleVisibility();
				break;
			case "TURBO":
				emulatorSpeedup = !emulatorSpeedup;
				break;
			case "SAVE":
				saveEmulatorToFile();
				break;
		}
	}

	function handleButtonUp(buttonID) {
		//console.log("Keyup: ", buttonID);
		switch(buttonID) {
			// Gameboy controls
			case "DOWN":
				emulator.clear_key_down();
				break;
			case "UP":
				emulator.clear_key_up();
				break;
			case "LEFT":
				emulator.clear_key_left();
				break;
			case "RIGHT":
				emulator.clear_key_right();
				break;
			case "A":
				emulator.clear_key_a();
				break;
			case "B":
				emulator.clear_key_b();
				break;
			case "START":
				emulator.clear_key_start();
				break;
			case "SELECT":
				emulator.clear_key_select();
				break;
		}
	}

</script>

<svelte:head>
	<title>CorrodedBoy - Gameboy Emulator</title>
	<link rel="icon" href="favicon.png">
	<html lang="en"/>
</svelte:head>

<svelte:window 
	on:keydown={(e) => handleButtonDown(keyBindings[e.code])} 
	on:keyup={(e) => handleButtonUp(keyBindings[e.code])}
/>

<main 
	on:drop|preventDefault|stopPropagation={dropFile} 
	on:dragover|preventDefault|stopPropagation={dragOverFile}
>
	<Popup bind:this={popup}/>
	<div id="game-column">
		<ControlsTop on:down={handleButtonEvent} on:up={handleButtonEvent}/>
		<DebugInfo bind:this={debugInfo}/>
		<Screen bind:this={screen}> 

		</Screen>
		<div id="controls">
			<div id="controls-upper-row">
				<ControlsArrows on:down={handleButtonEvent} on:up={handleButtonEvent}/>
				<ControlsAB on:down={handleButtonEvent} on:up={handleButtonEvent}/>
			</div>
			<ControlsStartSelect on:down={handleButtonEvent} on:up={handleButtonEvent}/>
		</div>
	</div>
</main>

<style>
	:global(html) {
		height: 100%;
		background-color: #202020;
		color: #fff;
	}

	:global(body) {
		margin: 0;
    	height: 100%;
	}

	main {
		display: block;
		height: 100%;
		width: 100%;
	}

	#game-column {
		margin-left: auto;
		margin-right: auto;
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		height: 100%;
		width: 576px;
	}

	#controls {
        display: block;
        visibility: visible;
        width: 100%;
    }

	#controls-upper-row {
		margin-top: 1em;
		display: flex;
		flex-direction: row;
	}


	@media (min-width: 640px) {
		main {
			max-width: none;
		}
	}
</style>