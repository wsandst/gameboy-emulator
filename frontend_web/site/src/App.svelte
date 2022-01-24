<script>
	/*
	TODO:
		General cleanup 
		Mobile support for controls:
			Set the website to fullscreen
			Minimize Corrodedboy to CBoy
			Move around buttons
			Move down emulator control buttons slightly, decrease button size
	*/
	import FileSaver from "file-saver"

	import * as audio from './audio.js';
	import Screen from "./Screen.svelte"
	import ControlsTop from "./controls/ControlsTop.svelte"
	import ControlsArrows from "./controls/ControlsArrows.svelte"
	import ControlsAB from "./controls/ControlsAB.svelte"
	import ControlsStartSelect from "./controls/ControlsStartSelect.svelte"
	import Popup from "./Popup.svelte"
	import DebugInfo from "./DebugInfo.svelte"
	import Header from "./Header.svelte"
	import {media} from "./stores"
	
	// Element bindings
	let popup;
	let debugInfo;
	let topButtons;
	let screen;

	// Emulator
	export let emulatorLib;
	let emulator;

	// Emulator controls
	let emulatorPaused = false;
	let emulatorSpeedup = false;
	let emulatorAudio = true;
	let emulatorRunning = false;

	let mostRecentSaveExists = window.localStorage.getItem('mostRecentSave') != null;

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
		"ControlLeft" : "TURBO",
		"KeyM" : "DEBUG",
	}

	// Emulator loop

	const renderLoop = () => {
		let framesRun = 0;
		let audioBuffer;

		if (!emulatorPaused) {
			if (!emulatorSpeedup) {
				while (emulator.run_until_frontend_event() != 0) {
					if (emulatorAudio) {
						audioBuffer = emulator.get_sound_queue();
						audio.pushAudioSamples(audioBuffer);
					}
				}
				framesRun++;
			}
			else {
				// Run in speedup mode.
				// Allow emulator to run as much as it can during one frametime
				const start = performance.now();
				let delta = 0;
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
		debugInfo.audioDataUpdate(audio);
		debugInfo.update(framesRun);
	};

	function startEmulator() {
		if (!emulatorRunning) {
				emulatorRunning = true;
				debugInfo.init();
				audio.initAudio();
				renderLoop();
		}
	}

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

	function loadRomDataToEmulator(romData, romFilename) {
		emulator = emulatorLib.EmulatorWrapper.new();
		emulator.load_rom(romData);
		emulator.set_rom_name(romFilename);
		startEmulator();
	}

	function loadSaveDataToEmulator(saveData) {
		emulator = emulatorLib.EmulatorWrapper.new();
		emulator.load_save(saveData);
		startEmulator();
	}

	function loadSaveStringToEmulator(saveStr) {
		emulator = emulatorLib.EmulatorWrapper.new();
		emulator.load_save_str(saveStr);
		startEmulator();
	}

	function loadFileToEmulator(file) {
		let romFilename = file.name.split(".")[0];
		let isRomfile = file.name.endsWith('.gb') || file.name.endsWith('.bin');
  		let isSavefile = file.name.endsWith('.save');

		let fileData = new Blob([file]);
		let promise = new Promise(getFileBuffer(fileData));
		promise.then(function(data) {
			console.log(data);
			if (isRomfile) {
				loadRomDataToEmulator(data, romFilename);
			}
			else if (isSavefile) {
				loadSaveDataToEmulator(data);
			}
		}).catch(function(err) {
			console.log('Error: ',err);
		});
	}

	function loadRomBlobToEmulator(fileBlob, filename) {
		let promise = new Promise(getFileBuffer(fileBlob));
		promise.then(function(data) {
			loadRomDataToEmulator(data, filename);
		});
	}

	function loadMostRecentSave() {
		if (mostRecentSaveExists) {
			let saveStr = window.localStorage.getItem('mostRecentSave');
			loadSaveStringToEmulator(saveStr);
		}
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
		mostRecentSaveExists = true;

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
		if (!emulatorRunning) {
			return;
		}
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
				topButtons.togglePauseIcon();
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
			case "AUDIO":
				emulatorAudio = !emulatorAudio;
				topButtons.toggleAudioIcon();
				break;
		}
	}

	function handleButtonUp(buttonID) {
		//console.log("Keyup: ", buttonID);
		if (!emulatorRunning) {
			return;
		}
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
	<html lang="en"/>
	<meta name="theme-color" content="#202020">
	<meta name="apple-mobile-web-app-capable" content="yes">
	<meta name="apple-mobile-web-app-status-bar-style" content="black-translucent">
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
	<Header 
		on:loadFile={(e) => loadFileToEmulator(e.detail.file)}
		on:loadRomData={(e) => loadRomBlobToEmulator(e.detail.data, e.detail.filename)}
		on:loadMostRecentSave={loadMostRecentSave}
		mostRecentSaveExists={mostRecentSaveExists}
	/>
	<div id="game-row">
		<div class="landscape-controls" id="controls-left">
			<ControlsArrows on:down={handleButtonEvent} on:up={handleButtonEvent}/>
			<div id="controls-left-lower">
				<ControlsStartSelect on:down={handleButtonEvent} on:up={handleButtonEvent}/>
			</div>
		</div>
		<div id="game-column">
			<ControlsTop bind:this={topButtons} on:down={handleButtonEvent} on:up={handleButtonEvent}/>
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
		<div class="landscape-controls" id="controls-right">
			<ControlsAB on:down={handleButtonEvent} on:up={handleButtonEvent}/>
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
		background-color: #202020;
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
	}

	#game-row {
		display: flex;
		flex-direction: row;
		align-items: center;
		height: 100%;
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

	#controls-right {
		display: flex;
		flex-direction: column;
		justify-content: center;
		padding-bottom: 7.5em;
	}

	#controls-left {
		display: flex;
		flex-direction: column;
		justify-content: center;
		padding-top: 6em;
		width: 0;
	}

	#controls-left-lower {
		display: flex;
		align-items: center;
		padding-top: 2.5em;
		padding-right: 5em;
		align-self: flex-end;
		width: 10px;
		margin-right: 0.5em;
	}

	.landscape-controls {
		flex-grow: 1;
		height: 100%;
	}

	/* Responsiveness */

	@media (orientation:portrait) and (min-width: 640px) {
		main {
			max-width: none;
		}
	}


	@media only screen and (orientation:landscape) and (min-height: 480px) {
		#controls-left-lower {
			padding-top: 7em;
		}

		#controls-left {
			padding-top: 8em;
		}
	}


	@media only screen and (min-width: 1025px) {

		#controls {
			display: none;
			visibility: hidden;
			height: 0;
		}

		.landscape-controls {
			display: none;
			visibility: hidden;
			height: 0;
		}

		#game-column {
			flex-direction: column-reverse;
		}
	}
	
	@media only screen and (orientation:landscape) and (max-width: 1025px) {
		.landscape-controls {
			display: block;
			visibility: visible;
		}

		#controls {
			display: none;
			visibility: hidden;
			height: 0;
		}
	}

	@media only screen and (orientation:portrait) and (max-width: 1025px)  {
		#controls {
			display: block;
			visibility: visible;
			width: 100%;
		}

		.landscape-controls {
			display: none;
			visibility: hidden;
			height: 0;
		}
	}

</style>