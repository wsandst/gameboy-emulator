<script>
    import { controlButtonStyle} from './controlStyles.js';
    import { createEventDispatcher } from 'svelte';

    let paused = false;
    let audio = true;

    const dispatch = createEventDispatcher();

    export function togglePauseIcon() {
        paused = !paused;
    }

    export function toggleAudioIcon() {
        audio = !audio;
    }

    function buttonDown(button) {
        dispatch('down', {
            text: button
        });
    }

    function buttonUp(button) {
        dispatch('up', {
            text: button
        });
    }

</script>

<div>
        <button 
            class={controlButtonStyle}
            on:mousedown|preventDefault={() => buttonDown("TURBO")} 
            on:touchstart|preventDefault={() => buttonDown("TURBO")}
            on:touchmove|preventDefault={() => buttonDown("TURBO")}
            on:mouseup|preventDefault={() => buttonUp("TURBO")}
            on:touchend|preventDefault={() => buttonUp("TURBO")}
            on:touchcancel|preventDefault={() => buttonUp("TURBO")}
        > 
            ⚡
        </button>

        <button 
            class={controlButtonStyle}
            on:mousedown|preventDefault={() => buttonDown("PAUSE")} 
            on:touchstart|preventDefault={() => buttonDown("PAUSE")}
            on:touchmove|preventDefault={() => buttonDown("PAUSE")}
            on:mouseup|preventDefault={() => buttonUp("PAUSE")}
            on:touchend|preventDefault={() => buttonUp("PAUSE")}
            on:touchcancel|preventDefault={() => buttonUp("PAUSE")}
        > 
            {paused ? "▶️" : "⏸️"}
        </button>

        <button 
            class={controlButtonStyle}
            on:mousedown|preventDefault={() => buttonDown("SAVE")} 
            on:touchstart|preventDefault={() => buttonDown("SAVE")}
            on:touchmove|preventDefault={() => buttonDown("SAVE")}
            on:mouseup|preventDefault={() => buttonUp("SAVE")}
            on:touchend|preventDefault={() => buttonUp("SAVE")}
            on:touchcancel|preventDefault={() => buttonUp("SAVE")}
        > 
            💾
        </button>

        <button 
            class={controlButtonStyle}
            on:mousedown|preventDefault={() => buttonDown("AUDIO")} 
            on:touchstart|preventDefault={() => buttonDown("AUDIO")}
            on:touchmove|preventDefault={() => buttonDown("AUDIO")}
            on:mouseup|preventDefault={() => buttonUp("AUDIO")}
            on:touchend|preventDefault={() => buttonUp("AUDIO")}
            on:touchcancel|preventDefault={() => buttonUp("AUDIO")}
        > 
            {audio ? "🔊" : "🔇"}
        </button>
</div>

<style>
    div {
        display: flex;
        justify-content: left;
        align-items: center;
        font-size: 16px;
        padding-top: 3.5em;
        padding-bottom: 0.4em;
        cursor: pointer;
    }

    /* Mobile controls */

	@media only screen and (min-width: 1025px) {
		div {
			padding-top: 0em;
		}
	}

	@media only screen and (max-width: 1025px) {
		div {
			padding-top: 3.5em;
		}
	}

	@media only screen and (max-height: 768px) and (min-width: 1000px) {
		div {
			margin-top: 2.5em;
		}
	}
</style>