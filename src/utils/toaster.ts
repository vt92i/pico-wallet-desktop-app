import { createToaster } from "@skeletonlabs/skeleton-svelte";

export const toaster = createToaster({
  max: 4,
  placement: "bottom-end",
});
