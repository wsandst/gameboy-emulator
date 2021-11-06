<script>
    import { controlButtonStyle} from './controlStyles.js';
    import { createEventDispatcher } from 'svelte';

    let buttons = [
        { id: 'btn-turbo', eventName : "TURBO", text: "⚡"},
        { id: 'btn-pause', eventName : "PAUSE", text: "⏸️"},
        { id: 'btn-save', eventName : "SAVE", text: "💾"},
        { id: 'btn-audio', eventName : "AUDIO", text: "🔊"},
    ];

    const dispatch = createEventDispatcher();

    function buttonDown(button) {
        if (button.defaultPrevented) {
            return; // Do nothing if event already handled
        }
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
        display: flex;
        justify-content: left;
        align-items: center;
        font-size: 16px;
        padding-top: 3.5em;
    }
</style>