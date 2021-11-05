<script>
	import Dropzone from "svelte-file-dropzone";

	import Screen from "./Screen.svelte"
	import ControlsTop from "./controls/ControlsTop.svelte"
	import ControlsArrows from "./controls/ControlsArrows.svelte"
	import ControlsAB from "./controls/ControlsAB.svelte"
	import ControlsStartSelect from "./controls/ControlsStartSelect.svelte"

	export let emulatorLib;
	let emulator = emulatorLib.EmulatorWrapper.new();
	let screen;

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
		const start = performance.now();
		while (emulator.run_until_frontend_event() != 0) {
		}

		let pixels = new Uint8ClampedArray(emulator.get_screen_bitmap())
		screen.update(pixels)

		requestAnimationFrame(renderLoop);
	};

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

	function handleFilesSelect(event) {
		const { acceptedFiles, fileRejections } = event.detail;
		let file = acceptedFiles[0];
		let romFilename = file.name.split(".")[0];

		let fileData = new Blob([file]);
		var promise = new Promise(getFileBuffer(fileData));
		promise.then(function(data) {
			emulator.load_rom(data);
			emulator.set_rom_name(romFilename);
			renderLoop();
		}).catch(function(err) {
			console.log('Error: ',err);
		});
	}

	function handleButtonEvent(event) {
		handleKeyDown(event.detail.text);
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
			/*case "KeyP":
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
			break;*/
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

<main>
	<Dropzone on:drop={handleFilesSelect} noClick noKeyboard disableDefaultStyles=true multiple=false>
		<div id="game-column">
			<ControlsTop on:down={handleButtonEvent} on:up={handleButtonEvent}/>
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
	</Dropzone>

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
		text-align: center;
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