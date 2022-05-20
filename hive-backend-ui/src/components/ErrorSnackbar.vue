<script setup lang="ts">
import { ref, watch } from 'vue';

const SNACKBAR_TIMEOUT = 3000; // 3s

const emit = defineEmits(["closeEvent"]);

const props = defineProps({
    isError: {
        type: Boolean,
        required: true,
    },
    message: {
        type: String,
        required: true,
    },
});

const snackbar = ref(false);

watch(() => props.isError, (newVal) => {
    snackbar.value = newVal;
})

watch(snackbar, (isActive) => {
    if (!isActive) {
        emit("closeEvent");
    }
})

</script>

<template>
    <v-snackbar v-model="snackbar" :timeout="SNACKBAR_TIMEOUT" color="error">
        {{ props.message }}
    </v-snackbar>
</template>