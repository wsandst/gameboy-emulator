// Svelte stores
import watchMedia from "svelte-media";

const mediaqueries = {
  desktop: "(min-width: 1025px)",
  mobile: "(max-width: 1025px)",
  portrait: "(orientation: portrait) and (max-width: 1025px)",
  landscape: "(orientation: landscape) and (max-width: 1025px),",
};

export const media = watchMedia(mediaqueries);