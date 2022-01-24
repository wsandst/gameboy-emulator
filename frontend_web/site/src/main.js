import App from './App.svelte';
import wasm from '../../Cargo.toml';

const init = async() => {
	const emulatorLib = await wasm();

	// Start the application with the emulator library loaded as
	// wasm
	const app = new App({
		target: document.body,
		props: {
			emulatorLib: emulatorLib
		}
	});
};

init();