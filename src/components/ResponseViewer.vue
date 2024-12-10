<script setup>
import { defineProps, defineEmits } from 'vue';
import VueJsonPretty from 'vue-json-pretty';
import 'vue-json-pretty/lib/styles.css';

const props = defineProps({
    response: [String, Object],
    isLoading: Boolean,
});
const emit = defineEmits(['cancel']);
</script>

<template>
    <div class="response-viewer">
        <h3>Response</h3>
        <div v-if="isLoading" class="loading-overlay">
            <p>Loading...</p>
            <button @click="emit('cancel')">Cancel</button>
        </div>
        <VueJsonPretty v-else-if="response" :data="response" showIcon showLineNumber showLine showLength
            showDoubleQuotes showKeyValueSpace collapseOnClickBrackets virtual :height="700" />
        <p v-else>No response yet</p>
    </div>
</template>

<style>
.response-viewer {
    width: 50%;
    padding: 1rem;
    border-left: 1px solid #ddd;
    position: relative;
}

.loading-overlay {
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    text-align: center;
    background: rgba(255, 255, 255, 0.8);
    padding: 1rem;
    border: 1px solid #ccc;
    box-shadow: 0 2px 5px rgba(0, 0, 0, 0.2);
    z-index: 10;
}

.loading-overlay button {
    margin-top: 1rem;
    padding: 0.5rem 1rem;
    background: #e74c3c;
    color: #fff;
    border: none;
    border-radius: 4px;
    cursor: pointer;
}

.loading-overlay button:hover {
    background: #c0392b;
}
</style>