<script lang="ts">
  import { Toaster } from "@skeletonlabs/skeleton-svelte";
  import type { Snippet } from "svelte";

  import "@app/app.css";
  import { toaster } from "@app/utils/toaster";

  export let children: Snippet<[]>;
</script>

<Toaster {toaster} />
{@render children()}
