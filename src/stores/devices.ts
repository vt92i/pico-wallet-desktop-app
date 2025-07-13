import { writable } from "svelte/store";

import { scanDevices } from "@app/api";
import type { Device } from "@app/types";

const createDevicesStore = () => {
  const { subscribe, set } = writable<Device[]>([]);

  return {
    subscribe,
    scan: async () => set(await scanDevices()),
    reset: () => set([]),
  };
};

export const devicesStore = createDevicesStore();
