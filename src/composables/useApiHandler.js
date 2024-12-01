import { ref } from 'vue';
import axios from 'axios';

export function useApiHandler() {
  const response = ref('');

  const sendRequest = async ({ method, url, headers, body }) => {
    try {
      const res = await axios({
        method,
        url,
        headers,
        data: body || null,
      });
      response.value = JSON.stringify(res.data, null, 2);
    } catch (error) {
      response.value = JSON.stringify(error.response?.data || error.message, null, 2);
    }
  };

  return { response, sendRequest };
}
