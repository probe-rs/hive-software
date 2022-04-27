/// Holds all data which is sent by the server.
/// Will later be used by the websocket connection to automatically
/// update the state of the app on data change
import { defineStore } from "pinia";

export const useServerData = defineStore("serverData", {
    state: () => ({
        targetData: [
            [
                {
                    Known: {
                        name: 'stm32f000',
                        architecture: null,
                        memory_address: null,
                        status: [Object]
                    }
                },
                'NotConnected',
                'NotConnected',
                'Unknown'
            ],
            [],
            [],
            [],
            null,
            [],
            null,
            null
        ],
    }),
});