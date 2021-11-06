<script>
    let hidden = true;
    let content;

    let frames;
    let lastFrameTimeStamp;
    let audioDelay;

    export function toggleVisibility() {
        hidden = !hidden;
    }

    export function init() {
        frames = []
        lastFrameTimeStamp = performance.now();
        audioDelay = 0;
    }

    export function update(multiplier) {
        if (hidden) {
            return;
        }
        // Convert the delta time since the last frame render into a measure
        // of frames per second.
        const now = performance.now();
        const delta = now - lastFrameTimeStamp;
        lastFrameTimeStamp = now;
        const fps = 1 / delta * 1000;

        // Save only the latest 100 timings.
        frames.push(fps);
        if (frames.length > 100) {
            frames.shift();
        }

        let sum = 0;
        for (let i = 0; i < frames.length; i++) {
            sum += frames[i];
        }
        let mean = sum / frames.length;

        /*if (currentSampleIndex % 30 == 0) {
            //let currentTime = performance.now();
            let playbackTime = currentSampleIndex * 1024/48000.0 + audioDelay;
            let actualTime = performance.now() - audioStartTimestamp;
            audioDelay = playbackTime*1000 - actualTime;
        }*/

        // Render the statistics.
        content.textContent = `FPS: ${Math.round(fps)*multiplier}, mean: ${Math.round(mean)*multiplier}. Audio delay: ${Math.round(this.audioDelay)}`.trim();
    }

</script>


<div bind:this={content} class:hidden>
    start game for debug info
</div>

<style>
    div {
        white-space: pre;
        font-family: monospace;
        padding-bottom: 0.5em;
        padding-top: 0;
        display: block;
    }

    .hidden {
        display: none;
    }

</style>