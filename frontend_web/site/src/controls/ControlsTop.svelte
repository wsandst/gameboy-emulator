<script>
    import { controlButtonStyle} from './controlStyles.js';
    import { createEventDispatcher } from 'svelte';

    let buttons = [
        { id: 'btn-turbo', eventName : "turbo", text: "⚡"},
        { id: 'btn-pause', eventName : "pause", text: "⏸️"},
        { id: 'btn-save', eventName : "save", text: "💾"},
        { id: 'btn-audio', eventName : "audio", text: "🔊"},
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
        display: flex;
        justify-content: left;
        align-items: center;
        font-size: 16px;
        padding-top: 3.5em;
    }
</style>