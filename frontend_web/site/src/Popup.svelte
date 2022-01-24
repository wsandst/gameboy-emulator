<script>
    // Component for displaying small popup messages
    let content;
    let messageVisible = false;

    export function display(message, duration) {
        console.log("Displaying popup message: ", message);
        content.textContent = message;
        messageVisible = true;
        setTimeout(fadeOut, duration);
    }

    function fadeOut() {
        messageVisible = false;
    }

    export function logConsoleToPopup() {
        console.log("Warning: Future console logs will now also show up as popups!");
        display("", 10000000);
        var oldLogger = console.log;
        console.log = function (message) {
            oldLogger(message);
            if (typeof message == 'object') {
                content.innerHTML += (JSON && JSON.stringify ? JSON.stringify(message) : message) + '<br />';
            } else {
                content.innerHTML += message + '<br />';
            }
        }
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