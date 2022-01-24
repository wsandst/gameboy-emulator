<script>
    /**
     * This component represents a group of Gameboy buttons containing
     * the four directional arrows.
    */
    import ControlButton from './ControlButton.svelte';
    import { faChevronLeft, faChevronRight, faChevronUp, faChevronDown } from '@fortawesome/free-solid-svg-icons'
</script>

<div id="top">
    <div id="btn-left">
        <ControlButton text={faChevronLeft} fa eventName="LEFT" on:down on:up/>
    </div>
    <div id="btn-right">
        <ControlButton text={faChevronRight} fa eventName="RIGHT" on:down on:up/>
    </div>
    <div id="btn-up">
        <ControlButton text={faChevronUp} fa eventName="UP" on:down on:up/>
    </div>
    <div id="btn-down">
        <ControlButton text={faChevronDown} fa eventName="DOWN" on:down on:up/>
    </div>
</div>

<style>
    #top {
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