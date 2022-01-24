<script>
     /**
     * This component represents a group of Gameboy buttons containing
     * various general emulator controls.
    */
    import ControlButton from './ControlButton.svelte';
    import { faBolt, faPause, faPlay, faSave, faVolumeUp, faVolumeMute } from '@fortawesome/free-solid-svg-icons'

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
        <ControlButton fa faSize="1.1x" text={faBolt} eventName="TURBO" on:down on:up title="Turbo [T]"/>
        <ControlButton fa faSize="1.1x" text={paused ? faPlay : faPause} eventName="PAUSE" on:down on:up 
            title={paused ? "Resume [P]" : "Pause [P]"}/>
        <ControlButton fa faSize="1.1x" text={faSave} eventName="SAVE" on:down on:up title="Save [N]"/>
        <ControlButton fa faSize="1.1x" text={audio ? faVolumeUp : faVolumeMute} eventName="AUDIO" on:down on:up 
            title={audio ? "Disable Audio [K]" : "Enable Audio [K]"}/>
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
            padding-bottom: 0.2em;
		}
	}
</style>