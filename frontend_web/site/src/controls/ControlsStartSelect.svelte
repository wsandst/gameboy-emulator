<script>
    import { controlButtonStyle} from './controlStyles.js';
    import { createEventDispatcher } from 'svelte';

    let buttons = [
		{ id: 'btn-start', eventName : "start", text: "start"},
        { id: 'btn-select', eventName : "select", text: "select"},
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
            on:mousedown={() => buttonDown(button.eventName)} 
            on:touchstart={() => buttonDown(button.eventName)}
            on:touchmove={() => buttonDown(button.eventName)}
            on:mouseup={() => buttonUp(button.eventName)}
            on:touchend={() => buttonUp(button.eventName)}
            on:touchcancel={() => buttonUp(button.eventName)}
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