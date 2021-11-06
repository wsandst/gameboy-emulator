<script>
    import { controlButtonStyle} from './controlStyles.js';
    import { createEventDispatcher } from 'svelte';

    let buttons = [
		{ id: 'btn-a', eventName : "A", text: "A"},
        { id: 'btn-b', eventName : "B", text: "B"},
	];

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

<div>
    {#each buttons as button}
        <button 
            id={button.id} class={controlButtonStyle}
            on:mousedown|preventDefault={() => buttonDown(button.eventName)} 
            on:touchstart|preventDefault={() => buttonDown(button.eventName)}
            on:touchmove|preventDefault={() => buttonDown(button.eventName)}
            on:mouseup|preventDefault={() => buttonUp(button.eventName)}
            on:touchend|preventDefault={() => buttonUp(button.eventName)}
            on:touchcancel|preventDefault={() => buttonUp(button.eventName)}
        > 
            {button.text} 
        </button>
    {/each}
</div>

<style>
    div {
        margin-top: 2em;
        margin-right: 0.5em;
        position: relative
    }

    #btn-a {
        position: absolute;
        right: 0px;
        top: 0px;
    }

    #btn-b {
        position: absolute;
        right: 60px;
        top: 20px;
    }
</style>