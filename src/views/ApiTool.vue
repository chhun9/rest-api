<script setup>
import { ref } from 'vue'
import Sidebar from '@/components/Sidebar.vue';
import RequestEditor from '@/components/RequestEditor.vue';
import ResponseViewer from '@/components/ResponseViewer.vue';
import { useApiHandler } from '@/composables/useApiHandler';

const { sendRequest, cancelRequest, isLoading, response } = useApiHandler();
const selectedApi = ref(null);
const selectedCollection = ref(null);
const editorWidth = ref(50);
const startX = ref(0);
const startWidth = ref(0);

const handleRequest = async (request) => {
  await sendRequest(request);
};

const handleApiFromSidebar = (selectedApiFromSidebar) => {
  selectedApi.value = selectedApiFromSidebar;
};

const handleCollectionFromSidebar = (selectedCollectionFromSidebar) => {
  selectedCollection.value = selectedCollectionFromSidebar;
};

const startResize = (e) => {
  startX.value = e.clientX;
  startWidth.value = editorWidth.value;
  window.addEventListener('mousemove', doResize);
  window.addEventListener('mouseup', stopResize);
};

const doResize = (e) => {
  const diff = e.clientX - startX.value;
  editorWidth.value = Math.min(Math.max(startWidth.value + (diff / window.innerWidth * 100), 10), 90);
};

const stopResize = (e) => {
  window.removeEventListener('mousemove', doResize);
  window.removeEventListener('mouseup', stopResize)
};

</script>

<template>
  <div class="api-tool">
    <!-- Sidebar: API 컬렉션 -->
    <Sidebar @selectApi="handleApiFromSidebar" @selectCollection="handleCollectionFromSidebar" />

    <!-- Request Editor: 요청 작성 -->
    <RequestEditor :style="{ width: editorWidth + '%' }" :selectedApi="selectedApi"
      :selectedCollection="selectedCollection" @request="handleRequest" />

    <div class="request-resizer" @mousedown="startResize" />

    <!-- Response Viewer: 응답 데이터 -->
    <ResponseViewer 
    :isLoading="isLoading"
    :response="response" 
    @cancel="cancelRequest"/>
  </div>
</template>

<style>
.api-tool {
  display: flex;
  height: 100vh;
  width: 100vw;
}

.request-resizer {
  width: 3px;
  border-left: solid 1px;
  border-right: solid 1px;
  cursor: ew-resize;
  transition: top ease-out 0.5s;
}

.request-resizer:hover {
  background-color: #aaa;
}
</style>
