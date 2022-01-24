<script>
    /**
     * This component represents a Gameboy control button,
     * ex A or Select. It implements general events for
     * listening for key up and key down.
    */
    import { createEventDispatcher } from 'svelte';
    import Fa from 'svelte-fa'

    export let text;
    export let eventName;
    export let title = "";
    export let fa = false;
    export let faSize = "1.2x";

    const dispatch = createEventDispatcher();

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

<button
    on:mousedown|preventDefault={() => buttonDown(eventName)} 
    on:touchstart|preventDefault={() => buttonDown(eventName)}
    on:touchmove|preventDefault={() => buttonDown(eventName)}
    on:mouseup|preventDefault={() => buttonUp(eventName)}
    on:touchend|preventDefault={() => buttonUp(eventName)}
    on:touchcancel|preventDefault={() => buttonUp(eventName)}
    title={title}
    > 
    {#if !fa}
        {text} 
    {:else}
        <Fa icon={text} size={faSize} color="white"/> 
    {/if}
</button>

<style>
    button {
        background-color: #474747;
        color: #ffffff;
        display: inline-block;
        border: none;
        margin: 0.5em;
        text-decoration: none;
        font-size: 1em;
        cursor: pointer;
        text-align: center;
        -webkit-appearance: none;
        -moz-appearance: none;
        border-radius: 8px; 
        text-align: center;
        box-shadow: 4px 4px #000000;
        -webkit-tap-highlight-color: transparent;
        -webkit-touch-callout: none; /* iOS Safari */
        -webkit-user-select: none; /* Safari */
        -moz-user-select: none; /* Old versions of Firefox */
        -ms-user-select: none; /* Internet Explorer/Edge */
        user-select: none;
        cursor: pointer;
        min-width: 40px;
        min-height: 32px;
        white-space: pre;
    }

    @media only screen and (orientation:portrait) and (max-width: 480px) {
        button {
            font-size: 0.9em;
        }
    }

    @media only screen and (orientation:landscape) {
        button {
            font-size: 0.94em;
        }
    }
</style>