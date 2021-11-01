<script>
	import Dropzone from "svelte-file-dropzone";
	import Screen from "./Screen.svelte"
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
	
</script>

<main>
	<Dropzone on:drop={handleFilesSelect} noClick noKeyboard disableDefaultStyles multiple=false>
		<h1>Emulator in Svelte!</h1>
		<Screen bind:this={screen}> 

		</Screen>
	</Dropzone>

</main>

<style>
	main {
		text-align: center;
		padding: 1em;
		max-width: 240px;
		margin: 0 auto;
	}

	h1 {
		color: #ff3e00;
		text-transform: uppercase;
		font-size: 4em;
		font-weight: 100;
	}

	@media (min-width: 640px) {
		main {
			max-width: none;
		}
	}
</style>