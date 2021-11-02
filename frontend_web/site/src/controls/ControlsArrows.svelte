<script>
    import { controlButtonStyle} from './controlStyles.js';
    import { createEventDispatcher } from 'svelte';

    let buttons = [
		{ id: 'btn-left', eventName : "left", text: "←"},
        { id: 'btn-right', eventName : "right", text: "→"},
        { id: 'btn-up', eventName : "up", text: "↑"},
        { id: 'btn-down', eventName : "down", text: "↓"},
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
        margin-right: auto;
        margin-left: 0.5em;
        position: relative;
    }

    #btn-left {
        position: absolute;
        left: 0px;
        top: 45px;
        width: 40px;
        height: 40px;
    }

    #btn-right {
        position: absolute;
        left: 90px;
        top: 45px;
        width: 40px;
        height: 40px;
    }

    #btn-up {
        position: absolute;
        left: 45px;
        top: 0px;
        width: 40px;
        height: 40px;
    }

    #btn-down {
        position: absolute;
        left: 45px;
        top: 90px;
        width: 40px;
        height: 40px;
    }
</style>