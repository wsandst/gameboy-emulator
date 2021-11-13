<script>
    import { onMount } from 'svelte';
    let canvas;
    let ctx;

    onMount(() => {
        canvas.height = 144;
        canvas.width = 160;
        ctx = canvas.getContext('2d');
    })

    export function update(pixels) {
        const imageData = new ImageData(pixels, canvas.width, canvas.height);
		ctx.putImageData(imageData, 0, 0);
    }
</script>

<canvas bind:this={canvas}>

</canvas>

<style>
    canvas {
		image-rendering: optimizeSpeed;             /* Older versions of FF          */
		image-rendering: -moz-crisp-edges;          /* FF 6.0+                       */
		image-rendering: -webkit-optimize-contrast; /* Safari                        */
		image-rendering: -o-crisp-edges;            /* OS X & Windows Opera (12.02+) */
		image-rendering: pixelated;                 /* Awesome future-browsers       */
		background-color: #353535;
		box-shadow: inset 0 0 15px #000000;
		border-radius: 3px; 
		height: 576px; 
        width: 640px;
	}

    @media only screen and (max-width: 480px) {
        canvas {
            width: 320px;
            height: 288px;
        }
    }

	@media only screen and (min-width: 480px) {
		canvas {
			width: 480px;
			height: 432px;
		}
	}

	@media only screen and (min-width: 640px) {
		canvas {
			height: 576px; 
			width: 640px;
		}
	}

	/* Mobile controls */

	@media only screen and (min-width: 1025px) {
		canvas {
			margin-bottom: 4em;
		}
	}

	@media only screen and (max-width: 1025px) {
		canvas {
			margin-bottom: 0;
		}
	}

	/* Adjust for laptops with a small max-height */
	@media only screen and (max-height: 850px) and (min-width: 1000px) {
		canvas {
			margin-bottom: 0;
		}
	}

	@media only screen and (max-height: 768px) and (min-width: 1000px) {
		canvas {
			margin-top: 2.5em;
		}
	}
</style>