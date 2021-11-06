<script>
    let content;
    let messageVisible = false;
    // Functions for displaying small popup message

    export function display(message, duration) {
        console.log("Displaying popup message: ", message);
        content.textContent = message;
        messageVisible = true;
        setTimeout(fadeOut, duration);
    }

    function fadeOut() {
        messageVisible = false;
    }

</script>

<div class:messageVisible>
    <h2 bind:this={content}> Test </h2>
</div>

<style>
    div {
        position: absolute;
        display: flex;
        visibility: hidden;
        opacity: 0;
        justify-content: center;
        align-items: center;
        pointer-events: none;
        background-color: transparent;
        width: 100%;
        height: 100%;
        z-index: 2;
        text-align: center;
        transition: visibility 0.4s linear, opacity 0.3s linear;
    }

    .messageVisible {
        visibility: visible;
        opacity: 1;
    }

    h2 {
        position: relative;
        color: white;
        font-size: 22px;
        text-decoration: none;
        transform: translateY(-50%);
        background-color: #353535;
        border-radius: 5px;
        padding: 20px;
    }
</style>