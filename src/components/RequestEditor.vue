<template>
    <div class="request-editor">
        <!-- Selected API Name -->
        <div class="api-name">
            <h2> api name : {{ selectedApi?.name }}</h2>
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

        <!-- Headers -->
        <div class="request-settings">
            <h3>Headers</h3>
            <button @click="addHeader" class="add-btn">+ Add Header</button>
            <div v-for="(header, index) in headers" :key="index" class="key-value-pair">
                <input v-model="header.key" type="text" placeholder="Key" class="key-input" />
                <input v-model="header.value" type="text" placeholder="Value" class="value-input" />
                <button @click="removeHeader(index)" class="remove-btn">-</button>
            </div>
        </div>

        <!-- Parameters -->
        <div class="request-settings">
            <h3>Parameters</h3>
            <button @click="addParameter" class="add-btn">+ Add Parameter</button>
            <div v-for="(param, index) in parameters" :key="index" class="key-value-pair">
                <select v-model="param.parameter_type" class="param-type-select">
                    <option value="PATH">PATH</option>
                    <option value="QUERY">QUERY</option>
                </select>
                <input v-model="param.key" type="text" placeholder="Key" class="key-input" />
                <input v-model="param.value" type="text" placeholder="Value" class="value-input" />
                <button @click="removeParameter(index)" class="remove-btn">-</button>
            </div>
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
import JsonEditorVue from 'json-editor-vue'
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
const parameters = ref([]);
const body = ref('');
const emit = defineEmits();

watch(() => props.selectedApi, (newValue) => {
    method.value = newValue.method
    url.value = newValue.url
    headers.value = newValue.headers
    parameters.value = newValue.parameters
    body.value = newValue.body
    analyzeUrl();
});

watch(() => method.value, (newValue) => {
    props.selectedApi.method = newValue;
    saveApi(props.selectedApi);
});
watch(() => url.value, (newValue) => {
    props.selectedApi.url = newValue;
    saveApi(props.selectedApi);
});
watch(() => headers.value, (newValue) => {
    props.selectedApi.headers = newValue;
    saveApi(props.selectedApi);
}, { deep: true });
watch(() => parameters.value, (newValue) => {
    props.selectedApi.parameters = newValue;
    saveApi(props.selectedApi);
}, { deep: true });
watch(() => body.value, (newValue) => {
    props.selectedApi.body = newValue;
    saveApi(props.selectedApi);
});

// Save api to backend
const saveApi = async (api) => {
    const serializedData = JSON.stringify(api);
    await invoke('save_api', { data: serializedData });
};

// Add and remove handlers
const addParameter = () => parameters.value.push({ parameter_type: 'QUERY', key: '', value: '' });
const removeParameter = (index) => parameters.value.splice(index, 1);

// Add and remove header handlers
const addHeader = () => headers.value.push({ key: '', value: '' });
const removeHeader = (index) => headers.value.splice(index, 1);

// Analyze URL to extract query and path parameters
const analyzeUrl = () => {
    url.value = url.value.trim()
    const currentParams = [...parameters.value];
    const urlWithoutQuery = url.value.split('?')[0];
    const queryString = url.value.split('?')[1];

    parameters.value = [];

    // Extract Path Params
    const pathParams = urlWithoutQuery.match(/{([^}]+)}/g);
    if (pathParams) {
        pathParams.forEach((param) => {
            const key = param.replace(/[{}]/g, '')
            const existingParam = currentParams.find(p => p.key === key && p.parameter_type === 'PATH')
            parameters.value.push(existingParam || { parameter_type: 'PATH', key, value: '' })
        });
    }

    // Extract Query Params
    if (queryString) {
        const queries = queryString.split('&');
        queries.forEach((query) => {
            const [key, value] = query.split('=');
            const decodeKey = decodeURIComponent(key)
            const decodeValue = decodeURIComponent(value || '')
            const existingParam = currentParams.find(p => p.key === decodeKey && p.parameter_type === 'QUERY')
            parameters.value.push(existingParam || { parameter_type: 'QUERY', key: decodeKey, value: decodeValue })
        });
    }

    currentParams.forEach(param => {
        if (!parameters.value.find(p => p.key === param.key && p.parameter_type === param.parameter_type)) {
            parameters.value.push(param)
        }
    })
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
        const requestHeaders = headers.value.reduce((header, current) => {
            if (current_key && current.value) {
                header.push({ key: current_key, value: current.value })
            }
            return header
        }, []);
        emit('request', {
            method: method.value,
            url: fullUrl,
            headers: requestHeaders,
            body: body.value
        })
    } catch (error) {
        console.error('Request failed:', error);
    }
};

// Helper: Build URL with Parameters
const buildUrlWithParameters = (baseUrl, params) => {
    let finalUrl = baseUrl;

    // Replace Path Params
    params
        .filter((param) => param.parameter_type === 'PATH' && param.key)
        .forEach((param) => {
            finalUrl = finalUrl.replace(`{${param.key}}`, encodeURIComponent(param.value));
        });

    // Append Query Params
    const queryParams = params
        .filter((param) => param.parameter_type === 'QUERY' && param.key)
        .map((param) => `${encodeURIComponent(param.key)}=${encodeURIComponent(param.value)}`)
        .join('&');
    return queryParams ? `${finalUrl.split('?')[0]}?${queryParams}` : finalUrl;
};
</script>

<style scoped>
.request-editor {
    padding: 10px;
    border-right: 1px solid #ddd;
    background: #f9f9f9;
}

.api-name h2 {
    margin: 0 0 0px;
    font-size: 24px;
}

.api-name h5 {
    margin: 0 0 0px;
    font-size: 15px;
}

.method-url {
    display: flex;
    gap: 10px;
    margin-bottom: 10px;
}

.method-select,
.url-input,
.send-btn {
    padding: 8px;
    font-size: 14px;
}

.url-input {
    width: 100%;
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
    margin: 10px 0;
    font-size: 18px;
}

.key-value-pair {
    display: flex;
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
    width: 97%;
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