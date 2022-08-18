<script setup lang="ts">
import { toRefs } from "vue";

const props = defineProps({
  text: {
    type: String,
    required: true,
  },
  isActive: {
    type: Boolean,
    required: true,
  },
});

const { isActive, text } = toRefs(props);

const emit = defineEmits(["confirm", "cancel"]);

function confirm() {
  emit("confirm");
}

function cancel() {
  emit("cancel");
}
</script>

<template>
  <v-dialog v-model="isActive" persistent>
    <v-card>
      <v-card-title class="text-h5"> Confirmation </v-card-title>
      <v-card-text>{{ text }}</v-card-text>
      <v-card-actions>
        <v-spacer></v-spacer>
        <v-btn color="error" text @click="cancel"> Cancel </v-btn>
        <v-btn color="success" text @click="confirm"> Confirm </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>
