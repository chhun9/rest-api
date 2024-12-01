<script setup>
import { defineProps } from 'vue';

// Props와 Emits 선언
const props = defineProps({
    response: String,
});
</script>
<template>
    <div class="response-viewer">
        <h3>Response</h3>
        <pre v-if="response">{{ response }}</pre>
        <p v-else>No response yet</p>
    </div>
</template>


<style>
.response-viewer {
    width: 40%;
    padding: 1rem;
    background: #f9f9f9;
    border-left: 1px solid #ddd;
}
</style>