<script setup>
import { ref } from 'vue'
import Sidebar from '@/components/Sidebar.vue';
import RequestEditor from '@/components/RequestEditor.vue';
import ResponseViewer from '@/components/ResponseViewer.vue';
import { useApiHandler } from '@/composables/useApiHandler';

// API 핸들러 composable 사용
const { sendRequest, response } = useApiHandler();
const selectedApi = ref(null);
const selectedCollection = ref(null);
// 이벤트 핸들러
const handleRequest = async (request) => {
  await sendRequest(request);
};
const handleApiFromSidebar = (selectedApiFromSidebar) => {
  selectedApi.value = selectedApiFromSidebar;
};
const handleCollectionFromSidebar = (selectedCollectionFromSidebar) => {
  selectedCollection.value = selectedCollectionFromSidebar;
};
</script>

<template>
  <div class="api-tool">
    <!-- Sidebar: API 컬렉션 -->
    <Sidebar @selectApi="handleApiFromSidebar" @selectCollection="handleCollectionFromSidebar" />

    <!-- Request Editor: 요청 작성 -->
    <RequestEditor :selectedApi="selectedApi" :selectedCollection="selectedCollection" 
      @request="handleRequest" />

    <!-- Response Viewer: 응답 데이터 -->
    <ResponseViewer :response="response" />
  </div>
</template>

<style>
.api-tool {
  display: flex;
  height: 100vh;
}
</style>
