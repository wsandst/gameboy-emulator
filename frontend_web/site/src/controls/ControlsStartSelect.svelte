<script>
    import ControlButton from './ControlButton.svelte';
</script>

<div id="top">
    <div id="btn-start">
        <ControlButton text="start" eventName="START" on:down on:up/>
    </div>
    <div id="btn-select">
        <ControlButton text="select" eventName="SELECT" on:down on:up/>
    </div>
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