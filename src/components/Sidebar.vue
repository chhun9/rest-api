<template>
    <div class="sidebar">
        <!-- Tab Buttons for Collections and API List -->
        <div class="tabs">
            <div :class="['tab', { active: activeTab === 'collections' }]" @click="setActiveTab('collections')">
                Collections
            </div>
            <div :class="['tab', { active: activeTab === 'apis' }]" @click="setActiveTab('apis')">
                API List
            </div>
        </div>

        <div class="create-new" @click="createNewItem">
            {{ activeTab === 'collections' ? 'New Collection' : 'New API' }}
        </div>

        <!-- Collection List -->
        <div v-if="activeTab === 'collections'" class="item-list">
            <!-- New Item Input -->
            <div v-if="isAddingItem" class="item">
                <input class="new-item-input" v-model="newItemName" @keydown.enter="saveNewItem" @blur="saveNewItem"
                    placeholder="Enter collection name" />
            </div>

            <div v-for="(collection, index) in collections" :key="index" class="item"
                @contextmenu="showContextMenu($event, collection, 'collection', index)">
                <!-- Only the collection name is highlighted -->
                <div @click="selectCollection(collection)" :class="{ highlighted: activeCollection === collection }">
                    {{ activeCollection === collection ? '▼' : '►' }} {{ collection.name }}
                </div>

                <!-- APIs under the selected collection -->
                <div v-if="activeCollection === collection" class="api-list">
                    <!-- New Item Input in collection-->
                    <div v-if="isAddingItemInCollection" class="item-in-collection">
                        <input class="new-item-input-in-collection" v-model="newItemName" @keydown.enter="saveNewItem"
                            @blur="saveNewItem" placeholder="Enter API name in collection" />
                    </div>

                    <!-- Individual APIs -->
                    <div v-for="(api, apiIndex) in collection.apis" :key="apiIndex"
                        :class="['item-api', { highlighted: selectedApi === api }]" @click="selectApi(api)"
                        @contextmenu="contextApi(api), showContextMenu($event, api, 'api', apiIndex)">
                        - {{ api.name }}
                    </div>
                    <button class="btn-in-collection" @click="createNewApi">
                        New API in {{ activeCollection.name }}
                    </button>
                </div>
            </div>
        </div>

        <!-- Global API List  -->
        <div v-if="activeTab === 'apis'" class="item-list">
            <div v-if="isAddingItem" class="item">
                <input class="new-item-input" v-model="newItemName" @keydown.enter="saveNewItem" @blur="saveNewItem"
                    placeholder="Enter collection name" />
            </div>
            <div v-for="(api, index) in apis" :key="index" :class="['item', { highlighted: selectedApi === api }]"
                @click="selectApi(api)" @contextmenu="showContextMenu($event, api, 'api', index)">
                {{ api.name }}
            </div>
        </div>

        <!-- Context Menu -->
        <div v-if="contextMenu.visible" :style="{ top: contextMenu.y + 'px', left: contextMenu.x + 'px' }"
            class="context-menu">
            <button @click="deleteItem">Delete</button>
        </div>
    </div>
</template>

<script setup>
import { ref, onMounted, nextTick, defineEmits } from 'vue';
import { invoke } from '@tauri-apps/api/core';

// Local state
const collections = ref([]);
const apis = ref([]);
const activeTab = ref('collections');
const activeCollection = ref(null);
const deleteSelectApi = ref(null);
const isAddingItem = ref(false);
const isAddingItemInCollection = ref(null);
const newItemName = ref('');
const contextMenu = ref({ visible: false, x: 0, y: 0, item: null, type: '' });
const selectedApi = ref('');
const emit = defineEmits();

onMounted(() => {
    loadData();
    document.addEventListener('click', handleOutsideClick);
});

onUnmounted(() => {
    document.removeEventListener('click', handleOutsideClick);
});
// Load data from backend
const loadData = async () => {
    try {
        const data = await invoke('read_data');
        collections.value = data.collections;
        apis.value = data.apis;
    } catch (err) {
        console.error('Error loading data:', err);
    }
};

// Set active tab
const setActiveTab = (tab) => {
    activeCollection.value = null;
    activeTab.value = tab;
};

// Set active collection
const selectCollection = (collection) => {
    if (activeCollection.value === null) {
        activeCollection.value = collection;
        deleteSelectApi.value = null;
    } else {
        activeCollection.value = null;
        deleteSelectApi.value = null;
    }
};

const selectApi = (api) => {
    if (activeCollection.value) {
        emit('selectCollection', activeCollection.value);
    }
    else {
        emit('selectCollection', null);
    }
    selectedApi.value = api;
    emit('selectApi', selectedApi.value);
};

const contextApi = (api) => {
    deleteSelectApi.value = api;
};

// Show context menu
const showContextMenu = (event, item, type, index) => {
    event.preventDefault();
    contextMenu.value = { visible: true, x: event.clientX, y: event.clientY, item, type, index };
};
const handleOutsideClick = (event) => {
    const contextMenuElement = document.querySelector('.context-menu');
    deleteSelectApi.value = null;
    if (contextMenuElement && !contextMenuElement.contains(event.target)) {
        hideContextMenu();
    }
};
// Hide context menu
const hideContextMenu = () => {
    contextMenu.value.visible = false;
};

// Create new item
const createNewItem = () => {
    isAddingItem.value = true;
    isAddingItemInCollection.value = false;
    newItemName.value = '';
    activeCollection.value = null;
    deleteSelectApi.value = null;
    nextTick(() => {
        const input = document.querySelector('.new-item-input');
        if (input) input.focus();
    });
};

// Create new API for selected collection or globally
const createNewApi = () => {
    isAddingItemInCollection.value = true;
    newItemName.value = '';
    deleteSelectApi.value = null;
    nextTick(() => {
        const input = document.querySelector('.new-item-input-in-collection');
        if (input) input.focus();
    });
};

// Save data to backend
const saveData = async () => {
    const appData = { collections: collections.value, apis: apis.value };
    const serializedData = JSON.stringify(appData);
    await invoke('save_data', { data: serializedData });
};

// Save new item
const saveNewItem = () => {
    if (newItemName.value.trim() === '') {
        isAddingItem.value = false;
        isAddingItemInCollection.value = false;
        return;
    }

    const newItem = activeTab.value === 'collections' && !isAddingItemInCollection.value
        ? { id: crypto.randomUUID(), name: newItemName.value, apis: [] }
        : { id: crypto.randomUUID(), name: newItemName.value, method: 'GET', url: '', headers: [], body: '' };

    if (activeTab.value === 'collections' && !activeCollection.value) {
        collections.value.unshift(newItem);
    } else if (activeTab.value === 'collections' && activeCollection.value) {
        activeCollection.value.apis.unshift(newItem);
    } else {
        apis.value.unshift(newItem);
    }

    newItemName.value = '';
    isAddingItem.value = false;
    isAddingItemInCollection.value = false;
    saveData();
};

// Delete item
const deleteItem = () => {
    if (contextMenu.value.type === 'collection') {
        if (!deleteSelectApi.value) {
            collections.value.splice(contextMenu.value.index, 1);
            activeCollection.value = null;
        } else {
            activeCollection.value.apis = activeCollection.value.apis.filter(api => api.id !== deleteSelectApi.value.id);
        }
        deleteSelectApi.value = null;
    } else if (contextMenu.value.type === 'api') {
        apis.value.splice(contextMenu.value.index, 1);
    }
    saveData();
    hideContextMenu();
};
</script>

<style scoped>
.sidebar {
    padding: 10px;
    background-color: #f4f4f4;
    border-right: 1px solid #ddd;
}

.tabs {
    display: flex;
    margin-bottom: 10px;
}

.tab {
    flex: 1;
    padding: 10px;
    text-align: center;
    cursor: pointer;
    background-color: #e4e4e4;
    border-radius: 5px;
    margin-right: 5px;
}

.tab.active {
    background-color: #a1c4fd;
    font-weight: bold;
}

.tab.active:hover {
    background-color: #8fb7f7;
    font-weight: bold;
}

.item-list {
    margin-top: 20px;
}

.create-new {
    flex: 1;
    padding: 10px;
    text-align: center;
    cursor: pointer;
    background-color: #5894f5;
    font-weight: bold;
    border-radius: 5px;
    margin-right: 5px;
}

.create-new:hover {
    background-color: #4b8cf4;
}

.item {
    padding: 10px;
    cursor: pointer;
}

.item .highlighted {
    background-color: #4d66b5;
    font-weight: bold;
    overflow: hidden;
    text-overflow: ellipsis;
}

.item.highlighted {
    background-color: #4d66b5;
    font-weight: bold;
    overflow: hidden;
    text-overflow: ellipsis;
}

.item-api {
    padding-left: 20px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}

.item-api:hover {
    background-color: #7371d5;
    font-weight: bold;
}

.item-api.highlighted {
    background-color: #4d66b5;
    font-weight: bold;
}

button {
    margin-top: 10px;
    padding: 10px;
    background-color: #4caf50;
    color: white;
    border: none;
    cursor: pointer;
}

button:hover {
    background-color: #45a049;
}

.context-menu {
    position: absolute;
    background-color: white;
    border: 1px solid #ddd;
    padding: 10px;
    box-shadow: 0 0 10px rgba(0, 0, 0, 0.2);
}

.context-menu button {
    background-color: #ff4d4d;
    border: none;
    color: white;
    padding: 5px 10px;
    cursor: pointer;
}

.context-menu button:hover {
    background-color: #ff0000;
}

.api-list button {
    margin-top: 10px;
    padding: 10px;
    background-color: #7cc97f;
    color: white;
    border: none;
    cursor: pointer;
}

.api-list button:hover {
    background-color: #78bd7a;
}
</style>