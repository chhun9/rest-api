import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';

export function useApiHandler() {
    const isLoading = ref(false);
    const response = ref(null);

    const sendRequest = async ({ method, url, headers, body }) => {
        isLoading.value = true;
        response.value = null;
        try {
            response.value = await invoke('send_request',{
              method, url, headers, body: body || null
            });
        } catch (error) {
            console.error(error);
        } finally {
            isLoading.value = false;
        }
    };

    const cancelRequest = async () => {
        try {
            isLoading.value = false;
            response.value = null;
            await invoke('cancel_request');
        } catch (error) {
            console.error(error);
        }
    };

    return { sendRequest, cancelRequest, isLoading, response };
}
