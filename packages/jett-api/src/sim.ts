import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

export class Sim {
  async connect(): Promise<void> {
    return invoke('sim_connect');
  }

  async disconnect(): Promise<void> {
    return invoke('sim_disconnect');
  }

  async subscribeVariable(
    name: string,
    unit: string,
    fps: number,
    callback: (value: number) => void
  ): Promise<UnlistenFn> {
    await invoke('sim_subscribe_variable', { name, unit, fps });
    return listen<{ name: string; value: number }>('jett-variable', (event) => {
      if (event.payload.name === name) {
        callback(event.payload.value);
      }
    });
  }

  async unsubscribeVariable(name: string): Promise<void> {
    return invoke('sim_unsubscribe_variable', { name });
  }

  async getVariable(name: string, unit: string): Promise<number> {
    return invoke<number>('sim_get_variable', { name, unit });
  }

  async subscribeEvent(
    eventName: string,
    callback: (data: number) => void
  ): Promise<UnlistenFn> {
    await invoke('sim_subscribe_event', { eventName });
    return listen<{ name: string; data: number }>('jett-event', (event) => {
      if (event.payload.name === eventName) {
        callback(event.payload.data);
      }
    });
  }

  async transmitEvent(eventName: string, data: number = 0): Promise<void> {
    return invoke('sim_transmit_event', { eventName, data });
  }
}
