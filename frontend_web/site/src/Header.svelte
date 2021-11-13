<script>
    // Header dropdown with various emulator controls,
    // mostly rom/save loading. Uses hover if available,
    // otherwise click for mobile devices

    import { createEventDispatcher } from 'svelte';

    export let mostRecentSaveExists = false;

    let dropdown;

    let romFileInput;
    let saveFileInput;
    let bootromInput;

    let dropdownHide = true;
    let subdropdown1Hide = true;
    let subdropdown2Hide = true;

    let hasHover = window.matchMedia("(hover: hover)").matches;

    const dispatch = createEventDispatcher();

    function handleFileInput(input) {
        closeDropdowns();
        let file = input.target.files[0]; 
        dispatch('loadFile', {
            file: file
        });
    }

    function loadServerSideRomFile(filename) {
        closeDropdowns();
        const url = "roms/"+filename;
        let romFilename = filename.split(".")[0];
        fetch(url).then(function(response) {
            response.blob().then(function(data) {
                    dispatch('loadRomData', {
                        data: data,
                        filename: romFilename
                    });
            });
        });
    }

    function loadMostRecentSave() {
        closeDropdowns();
        if (mostRecentSaveExists) {
            dispatch('loadMostRecentSave');
        }
    }

    // Mobile dropdown handling

    function closeDropdowns() {
        dropdownHide = true;
        subdropdown1Hide = true;
        subdropdown2Hide = true;
        window.removeEventListener('click', outsideDropdownClickListener);
    }

    function outsideDropdownClickListener(event) {
        // Only close dropdown if click was outside dropdown
        if (!dropdown.contains(event.target)) {
            closeDropdowns();
        }
    }

    function toggleDropdown(event) {
        if (hasHover) {
          return;
        }
        dropdownHide = !dropdownHide;
        if (dropdownHide) {
            closeDropdowns();
        }
        else {
            subdropdown1Hide=true;
            subdropdown2Hide=true;
            event.stopPropagation();
            window.addEventListener('click', outsideDropdownClickListener);
        }
    }

    function toggleSubDropdown1() {
        if (!hasHover) {
            subdropdown2Hide=true;
            subdropdown1Hide=!subdropdown1Hide;
        }
    }

    function toggleSubDropdown2() {
        if (!hasHover) {
            subdropdown1Hide=true;
            subdropdown2Hide=!subdropdown2Hide;
        }
    }

</script>

<div class="header">
    <input type="file" accept=".save" bind:this={saveFileInput} on:change={handleFileInput}>
    <input type="file" accept=".gb,.boot,.bootrom" bind:this={bootromInput} on:change={handleFileInput}>
    <input type="file" accept=".gb,.rom" bind:this={romFileInput} on:change={handleFileInput}>
    <div class="header-content">
      <div class="dropdown" bind:this={dropdown}>
        <div class="dropbtn" on:click={toggleDropdown}>
            <h2>Load</h2>
        </div>
        <div class="dropdown-content" class:dropdown-show={!dropdownHide} >
          <button
          class={mostRecentSaveExists ? "content-btn" : "content-btn-disabled"}
             on:click={loadMostRecentSave}
          >
            <h3>↪️ Last Save</h3>
        </button>
          <button class="content-btn" on:click={() => romFileInput.click()}>
            <h3>🎮 ROM File</h3>
          </button>
          <button class="content-btn" on:click={() => saveFileInput.click()}>
            <h3>💾 Save File</h3>
          </button>
          <div class="sub-dropdown" on:click={toggleSubDropdown1}>
            <div class="content-btn">
              <h3>🎉 Demo ROMs</h3>
            </div>
            <div class="sub-dropdown-content" class:dropdown-show={!subdropdown1Hide}>
              <button class="content-btn" on:click={() => loadServerSideRomFile("flappy_boy.gb")}>
                <h3> 🕊️ Flappy Boy </h3>
              </button>
              <button class="content-btn" on:click={() => loadServerSideRomFile("rex_run.gb")}>
                <h3> 🦖 Rex Run </h3>
              </button>
              <button class="content-btn" on:click={() => loadServerSideRomFile("pocket.gb")}>
                <h3> 🎉 Is That a Demo in Your Pocket? </h3>
              </button>
              <div class="content-btn" on:click={() => loadServerSideRomFile("dmgp_01.gb")}>
                <h3> 🎨 DMG*P-01 </h3>
              </div>
            </div>
          </div>
          <div class="sub-dropdown" on:click={toggleSubDropdown2}>
            <div class="content-btn">
              <h3>🧪 Test ROMs</h3>
            </div>
            <div class="sub-dropdown-content" class:dropdown-show={!subdropdown2Hide}>
              <button class="content-btn" on:click={() => loadServerSideRomFile("blargg_cpu_instrs.gb")}>
                <h3> 🤖 Blargg CPU Instrs </h3>
              </button>
              <button class="content-btn" on:click={() => loadServerSideRomFile("blargg_instr_timing.gb")}>
                <h3> ⏲️ Blargg Instr Timings </h3>
              </button>
              <button class="content-btn" on:click={() => loadServerSideRomFile("acid2.gb")}>
                <h3> 🎨 Acid2 </h3>
              </button>
            </div>
          </div>
          <button class="content-btn" on:click={() => bootromInput.click()}>
            <h3>🤖 Optional BootROM</h3>
          </button>
        </div>
      </div>
      <h1> CorrodedBoy</h1>
    </div>
</div>

<style>
    .header {
        position: fixed;
        top: 0;
        width: 100%;
    }

    .header-content {
        display: flex;
        flex-direction: row;
        justify-content: space-between;
        margin-left: auto;
        margin-right: auto;
        padding-left: 1.5rem;
        padding-right: 1.5rem;
        padding-top: 0.75rem;
        max-width: 640px;
    }

    button {
        all: unset;
        cursor: pointer;
    }

    input {
        display: none;
        width: 100%;
    }

    h1 {
        margin: 0;
        margin-top: auto;
        margin-bottom: auto;
        padding-right: 0.5rem;
    }

    h2 {
        margin: 0;
        text-decoration: underline;
        text-underline-offset: 3px;
        cursor: pointer;
    }

    /* The container <div> - needed to position the dropdown content */
    .dropdown {
        position: relative;
        display: inline-block;
        cursor: pointer;
        -moz-user-select: none; /* Old versions of Firefox */
        -ms-user-select: none; /* Internet Explorer/Edge */
        -webkit-tap-highlight-color: transparent;
        -webkit-touch-callout: none; /* iOS Safari */
        -webkit-user-select: none; /* Safari */
    }

    /* Dropdown Content (Hidden by Default) */
    .dropdown-content {
        display: none;
        position: absolute;
        background-color: #585858;
        min-width: 160px;
        box-shadow: 0px 8px 16px 0px rgba(0,0,0,0.2);
        z-index: 1;
        border-radius: 4px;
        box-shadow: 3px black;
    }
    
    /* Links inside the dropdown */
    .dropdown-content h3 {
        margin: 0;
        cursor: pointer;
        width: 100%;
    }

    .sub-dropdown-content {
        display: none;
        position: absolute;
        background-color: #585858;
        min-width: 170px;
        box-shadow: 0px 8px 16px 0px rgba(0,0,0,0.2);
        z-index: 1;
        border-radius: 0px 4px 4px 0;
        box-shadow: 3px black;
        margin-left: 160px;
        margin-top: -46px;
    }
    
    .content-btn {
        padding: 12px 8px;
        display: block;
        margin: 0;
        cursor: pointer;
        width: 100%;
        box-sizing: border-box;
    }

    .content-btn:hover {
        background-color: #313131;
    }

      /* Load file dropdown */
    .dropbtn {
        background-color: transparent;
        padding-top: 7px;
        padding-bottom: 7px;
        padding-right: 12px;
        font-size: 18px;
        margin: 0;
        cursor: pointer;
        /*border-radius: 8px;
        border: 2px solid white;*/
    }

    .sub-dropdown {
        cursor: pointer;
    }

    /* Change color of dropdown links on hover */
    .dropdown-content .content-btn:hover {color: rgb(197, 197, 197)}
  

    /* Only show hover on non-mobile devices*/
    @media (hover: hover) {
        /* Show the dropdown menu on hover */
        /*.dropdown:hover .dropdown-content {display: block;}*/
        .dropdown:hover .dropdown-content {
            display: inline-block;
        }

        .sub-dropdown:hover .sub-dropdown-content {
            display: inline-block;
        }

        /* Change the background color of the dropdown button when the dropdown content is shown */
        .dropdown:hover .dropbtn {color: rgb(197, 197, 197)}
    }

    .dropdown-show {
        display: inline-block !important;
    }

    .content-btn-disabled {
        padding: 12px 8px;
        display: block;
        margin: 0;
        cursor: pointer;
        background-color: #3a3a3a;
        color: rgb(197, 197, 197);
    }

    .content-btn-disabled:hover {
        background-color: #3a3a3a;
    }

</style>