// Audio related code
// Play audio from a sample buffer

export let audioStartTimestamp = null;
export let audioDelay = 0.05;
export let currentSampleIndex = 0;

let audioContext = null;
let audioFilterNode = null;
let previousAudioNode = null;

let queuedNodes = []

/**
 * Initiate the audio context.
 */
export function initAudio() {
    audioContext = new AudioContext();
    audioStartTimestamp = performance.now();

    // Apply lowpass filter ontop
    audioFilterNode = audioContext.createBiquadFilter();
    audioFilterNode.connect(audioContext.destination);
    audioFilterNode.type = "highpass";
    audioFilterNode.frequency.value = 200;

    pushAudioSilence(1024);
    console.log("Audio Latency: ", audioContext.baseLatency);
    console.log("Initiated audio");
}

/**
 * Push audio samples to the audio queue.
 * This uses AudioNodeBuffers.
 */ 
export function pushAudioSamples(sampleBuffer) {
    let length = sampleBuffer.length / 2;
    let audioBuffer = audioContext.createBuffer(2, length, 48000);
    let leftPcmBuffer = audioBuffer.getChannelData(0);
    let rightPcmBuffer = audioBuffer.getChannelData(1);
    for (let i = 0; i < audioBuffer.length; i++) {
        leftPcmBuffer[i] = sampleBuffer[i*2+0];
        rightPcmBuffer[i] = sampleBuffer[i*2+1];
    }
    let source = audioContext.createBufferSource();
    source.buffer = audioBuffer;
    source.connect(audioFilterNode);
    // start the source playing
    let currentTime = performance.now();
    let playbackTime = currentSampleIndex * 1024/48000.0 + audioDelay;
    let actualTime = performance.now() - audioStartTimestamp;
    const actualAudioDelay = playbackTime*1000-actualTime;
    if (actualAudioDelay <= 0) {
        console.log("Audio falling behind! Creating audio gap");
        let offset = actualTime/1000.0 - playbackTime + 0.1;
        audioDelay += offset;
        playbackTime += offset;
    }
    else if (actualAudioDelay > 150) {
        console.log("Audio too fast! Skipping audio");
        audioDelay -= 0.1;
        playbackTime -= 0.1;
    }
    queuedNodes.push(source);
    //previousAudioNode.onended = startNextNodeClosure(playbackTime, currentSampleIndex);
    source.start(playbackTime); 
    source.stop(playbackTime+length/48000.0)
    previousAudioNode = source;
    currentSampleIndex += 1;
}

// Push silence to the audio, used for syncing
/**
 * Push silence to the audio, used for syncing
 * @param length length of silence in milliseconds
 */
function pushAudioSilence(length) {
    let audioBuffer = audioContext.createBuffer(1, length, 48000);
    let pcmBuffer = audioBuffer.getChannelData(0);
    for (let i = 0; i < audioBuffer.length; i++) {
        pcmBuffer[i] = 0;
    }
    let source = audioContext.createBufferSource();
    source.buffer = audioBuffer;
    source.connect(audioFilterNode);
    let playbackTime = currentSampleIndex * 1024/48000.0 + audioDelay;
    source.start(playbackTime);
    //source.stop(playbackTime+length/48000.0);
    previousAudioNode = source;
    currentSampleIndex += 1;
}

// Used to chain nodes using onended, not currently used
// This might be required for iOS, but sounds terrible for some reason
/**
 * Used to chain nodes using the onended event, not currently used.
 * This might be required for iOS, but sounds terrible for some reason
 * @param playbackTime at what time should we start the next node
 * @returns a closure
 */
function startNextNodeClosure(playbackTime) {
    return function(e) {
        var node = queuedNodes.shift();
        node.start(playbackTime); 
        //node.stop(playbackTime+1024/48000.0)
    }
}
