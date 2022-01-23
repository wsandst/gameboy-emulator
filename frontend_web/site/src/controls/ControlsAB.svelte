<script>
    import ControlButton from './ControlButton.svelte';
</script>

<div id="top">
    <div id="btn-a">
        <ControlButton text="A" eventName="A" on:down on:up/>
    </div>
    <div id="btn-b">
        <ControlButton text="B" eventName="B" on:down on:up/>
    </div>
</div>

<style>
    #top {
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