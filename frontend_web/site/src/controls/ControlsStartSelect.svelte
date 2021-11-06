<script>
    import { controlButtonStyle} from './controlStyles.js';
    import { createEventDispatcher } from 'svelte';

    let buttons = [
		{ id: 'btn-start', eventName : "START", text: "start"},
        { id: 'btn-select', eventName : "SELECT", text: "select"},
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
        margin-top: 7em;
        margin-left: 0;
        text-align: center;
    }
</style>