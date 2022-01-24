<script>
    /**
     * This component represents a group of Gameboy buttons containing
     * start and select.
    */
    import ControlButton from './ControlButton.svelte';
    import { faChevronLeft, faChevronRight, faChevronUp, faChevronDown } from '@fortawesome/free-solid-svg-icons'
</script>

<div id="top">
    <ControlButton text="  start  " eventName="START" on:down on:up/>
    <ControlButton text="  select  " eventName="SELECT" on:down on:up/>
</div>

<style>
    #top {
        margin-top: 7em;
        margin-left: 0;
        text-align: center;
    }

    @media only screen and (max-width: 480px) {
        #top {
            margin-left: 8em;
        }
    }
    
</style>