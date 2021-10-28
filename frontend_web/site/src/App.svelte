<script>
	import Dropzone from "svelte-file-dropzone";
	import { onMount } from 'svelte';
	export let emulatorLib;
	let emulator = emulatorLib.EmulatorWrapper.new();

	let canvas;
	let canvasCtx;

	const renderLoop = () => {
		while (emulator.run_until_frontend_event() != 0) {
		}

		let pixels = new Uint8ClampedArray(emulator.get_screen_bitmap());
		const imageData = new ImageData(pixels, canvas.width, canvas.height);
		canvasCtx.putImageData(imageData, 0, 0);

		requestAnimationFrame(renderLoop);
	};

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
	
	onMount(() => {
		canvas.height = 144;
		canvas.width = 160;
		canvasCtx = canvas.getContext('2d');
	})
	
</script>

<main>
	<Dropzone on:drop={handleFilesSelect} noClick noKeyboard disableDefaultStyles multiple=false>
		<h1>Emulator in Svelte!</h1>
		<canvas bind:this={canvas}>

		</canvas>
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

	canvas {
		image-rendering: optimizeSpeed;             /* Older versions of FF          */
		image-rendering: -moz-crisp-edges;          /* FF 6.0+                       */
		image-rendering: -webkit-optimize-contrast; /* Safari                        */
		image-rendering: -o-crisp-edges;            /* OS X & Windows Opera (12.02+) */
		image-rendering: pixelated;                 /* Awesome future-browsers       */
		background-color: #353535;
		box-shadow: inset 0 0 15px #000000;
		border-radius: 3px; 
		height: 576px; 
        width: 640px;
	}
</style>