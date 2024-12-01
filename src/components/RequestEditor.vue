<template>
    <div class="request-editor">
        <!-- Selected API Name -->
        <div class="api-name">
            <h2> apiname : {{ selectedApi?.name }}</h2>
            <h5 v-if="selectedCollection"> collection name : {{ selectedCollection?.name }}</h5>
        </div>

        <!-- Method, URL Input, and Send Button -->
        <div class="method-url">
            <select v-model="method" class="method-select">
                <option value="GET">GET</option>
                <option value="POST">POST</option>
                <option value="PUT">PUT</option>
                <option value="DELETE">DELETE</option>
            </select>
            <input v-model="url" type="text" class="url-input" placeholder="Enter API URL" @blur="analyzeUrl" />
            <button @click="sendRequest" class="send-btn">Send</button>
        </div>

        <!-- Parameters -->
        <div class="request-settings">
            <h3>Parameters</h3>
            <div v-for="(param, index) in parameters" :key="index" class="key-value-pair">
                <select v-model="param.type" class="param-type-select">
                    <option value="Path">Path</option>
                    <option value="Query">Query</option>
                </select>
                <input v-model="param.key" type="text" placeholder="Key" class="key-input" />
                <input v-model="param.value" type="text" placeholder="Value" class="value-input" />
                <button @click="removeParameter(index)" class="remove-btn">-</button>
            </div>
            <button @click="addParameter" class="add-btn">+ Add Parameter</button>
        </div>

        <!-- Request Body -->
        <div v-if="method !== 'GET'" class="request-body">
            <h3>Request Body</h3>
            <textarea v-model="body" class="body-input" placeholder="Enter JSON body"></textarea>
            <button @click="formatRequestBody" class="format-btn">Format JSON</button>
        </div>
    </div>
</template>

<script setup>
import { ref, defineProps, watch, defineEmits } from 'vue';
import { invoke } from '@tauri-apps/api/core';

// Props: receive selectedApi from the parent (sidebar)
const props = defineProps({
    selectedApi: {
        type: Object,
        default: null,
    },
    selectedCollection: {
        type: Object,
        default: null,
    },
});

// Request State
const method = ref('');
const url = ref('');
const headers = ref([]);
const parameters = ref([]); // Combined parameters (query + path)
const body = ref('');
const emit = defineEmits();

watch(() => props.selectedApi, (newValue, oldValue) => {
    method.value = newValue.method
    url.value = newValue.url
    headers.value = newValue.headers
    body.value = newValue.body
});

watch(() => method.value , (newValue, oldValue) => {
    props.selectedApi.method = newValue;
    saveApi(props.selectedApi);
});
watch(() => url.value , (newValue, oldValue) => {
    props.selectedApi.url = newValue;
    saveApi(props.selectedApi);
});
watch(() => headers.value , (newValue, oldValue) => {
    props.selectedApi.headers = newValue;
    saveApi(props.selectedApi);
});
watch(() => body.value , (newValue, oldValue) => {
    props.selectedApi.body = newValue;
    saveApi(props.selectedApi);
});


// Save api to backend
const saveApi = async (api) => {
    const serializedData = JSON.stringify(api);
    await invoke('save_api', { data: serializedData });
};

// Add and remove handlers
const addParameter = () => parameters.value.push({ type: 'Query', key: '', value: '' });
const removeParameter = (index) => parameters.value.splice(index, 1);

// Analyze URL to extract query and path parameters
const analyzeUrl = () => {
    parameters.value = []; // Reset parameters

    const urlWithoutQuery = url.value.split('?')[0];
    const queryString = url.value.split('?')[1];

    // Extract Path Params
    const pathParams = urlWithoutQuery.match(/{([^}]+)}/g);
    if (pathParams) {
        pathParams.forEach((param) => {
            parameters.value.push({
                type: 'Path',
                key: param.replace(/[{}]/g, ''), // Remove curly braces
                value: '',
            });
        });
    }

    // Extract Query Params
    if (queryString) {
        const queries = queryString.split('&');
        queries.forEach((query) => {
            const [key, value] = query.split('=');
            parameters.value.push({
                type: 'Query',
                key: decodeURIComponent(key),
                value: decodeURIComponent(value || ''),
            });
        });
    }
};

// Format JSON in Request Body
const formatRequestBody = () => {
    try {
        const formattedBody = JSON.stringify(JSON.parse(body.value), null, 2);
        body.value = formattedBody;
    } catch (error) {
        alert('Invalid JSON format! Please correct the body before formatting.');
    }
};

// Send Request
const sendRequest = async () => {
    try {
        const fullUrl = buildUrlWithParameters(url.value, parameters.value);
        const requestOptions = {
            method: method.value,
            headers: {}, // Add headers logic here if needed
            body: method.value !== 'GET' ? body.value : undefined,
        };

        const response = await fetch(fullUrl, requestOptions);
        const result = await response.json();
        console.log(result);
    } catch (error) {
        console.error('Request failed:', error);
    }
};

// Helper: Build URL with Parameters
const buildUrlWithParameters = (baseUrl, params) => {
    let finalUrl = baseUrl;

    // Replace Path Params
    params
        .filter((param) => param.type === 'Path' && param.key)
        .forEach((param) => {
            finalUrl = finalUrl.replace(`{${param.key}}`, encodeURIComponent(param.value));
        });

    // Append Query Params
    const queryParams = params
        .filter((param) => param.type === 'Query' && param.key)
        .map((param) => `${encodeURIComponent(param.key)}=${encodeURIComponent(param.value)}`)
        .join('&');
    return queryParams ? `${finalUrl.split('?')[0]}?${queryParams}` : finalUrl;
};
</script>

<style scoped>
.request-editor {
    padding: 20px;
    background-color: #f9f9f9;
    border: 1px solid #ddd;
    border-radius: 8px;
}

.api-name h2 {
    margin: 0 0 20px;
    font-size: 24px;
}

.method-url {
    display: flex;
    align-items: center;
    gap: 10px;
    margin-bottom: 20px;
}

.method-select,
.url-input,
.send-btn {
    padding: 8px;
    font-size: 14px;
}

.send-btn {
    background-color: #4caf50;
    color: white;
    border: none;
    cursor: pointer;
}

.send-btn:hover {
    background-color: #45a049;
}

.request-settings h3,
.request-body h3 {
    margin: 20px 0 10px;
    font-size: 18px;
}

.key-value-pair {
    display: flex;
    align-items: center;
    gap: 10px;
    margin-bottom: 10px;
}

.key-input,
.value-input {
    padding: 8px;
    font-size: 14px;
    flex: 1;
}

.param-type-select {
    padding: 6px;
    font-size: 14px;
}

.add-btn,
.remove-btn,
.format-btn {
    padding: 6px 10px;
    font-size: 14px;
    border: none;
    cursor: pointer;
}

.add-btn {
    background-color: #007bff;
    color: white;
}

.remove-btn {
    background-color: #dc3545;
    color: white;
}

.format-btn {
    background-color: #007bff;
    color: white;
    margin-top: 10px;
}

.format-btn:hover {
    background-color: #0056b3;
}

.body-input {
    width: 100%;
    height: 150px;
    padding: 10px;
    font-size: 14px;
    border: 1px solid #ddd;
    border-radius: 4px;
}

.param-type {
    width: 60px;
    text-align: center;
    font-size: 12px;
    color: #fff;
    background-color: #6c757d;
    border-radius: 4px;
    padding: 2px 5px;
}
</style>