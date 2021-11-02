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

	const renderLoop = () => {
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

	function handleFilesSelect(e) {
		const { acceptedFiles, fileRejections } = e.detail;
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
		console.log(event.type, event.detail.text);
	}

</script>

<svelte:head>
	<title>CorrodedBoy - Gameboy Emulator</title>
	<link rel="icon" href="favicon.png">
	<html lang="en"/>
</svelte:head>

<main>
	<Dropzone on:drop={handleFilesSelect} noClick noKeyboard disableDefaultStyles multiple=false>
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