<script>
    import ControlButton from './ControlButton.svelte';

    let paused = false;
    let audio = true;

    export function togglePauseIcon() {
        paused = !paused;
    }

    export function toggleAudioIcon() {
        audio = !audio;
    }

</script>

<div id="top">
        <ControlButton text="⚡" eventName="TURBO" on:down on:up title="Turbo"/>
        <ControlButton text={paused ? "▶️" : "⏸️"} eventName="PAUSE" on:down on:up 
            title={paused ? "Resume" : "Pause"}/>
        <ControlButton text="💾" eventName="SAVE" on:down on:up title="Save"/>
        <ControlButton text={audio ? "🔊" : "🔇"} eventName="AUDIO" on:down on:up 
            title={audio ? "Disable Audio" : "Enable Audio"}/>
</div>

<style>
    #top {
        display: flex;
        justify-content: left;
        align-items: center;
        font-size: 16px;
        padding-top: 3.5em;
        padding-bottom: 0.4em;
    }

    /* Mobile controls */

	@media only screen and (min-width: 1025px) {
		#top {
			padding-top: 0em;
		}
	}

	@media only screen and (max-width: 1025px) {
		#top {
			padding-top: 3.5em;
		}
	}

	@media only screen and (max-height: 768px) and (min-width: 1000px) {
		#top {
			margin-top: 2.5em;
		}
	}

	@media only screen and (orientation:portrait) and (min-width: 1025px),
		(orientation:landscape) {
        #top {
			padding-top: 0.4em;
            padding-bottom: 0em;
		}
	}
</style>