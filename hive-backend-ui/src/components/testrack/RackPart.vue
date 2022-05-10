<script setup lang="ts">
import {
  defineProps,
  type PropType,
  ref,
  onMounted,
  watch,
  toRefs,
} from "vue";
import type { PartType } from "./types";
import { defaultRackScale } from "./constants";
import Konva from "konva";

const props = defineProps({
  type: {
    type: Number as PropType<PartType>,
    required: true,
  },
  location: {
    type: Number,
    required: true,
  },
  config: {
    type: Object,
    required: true,
  },
  isSelected: {
    type: Boolean,
    required: true,
  },
});

onMounted(() => {
  const node = (part.value! as any).getNode();

  node.enterTween = new Konva.Tween({
    node: node,
    y: node.absolutePosition().y - 20,
    duration: 0.2,
    easing: Konva.Easings.EaseOut,
  });
});

const scale = ref(defaultRackScale);
const part = ref(null);

function handleMouseEnter() {
  (part.value! as any).getNode().getStage().container().style.cursor =
    "pointer";

  if (!props.isSelected) {
    (part.value! as any).getNode().enterTween.play();
  }
}

function handleMouseLeave() {
  (part.value! as any).getNode().getStage().container().style.cursor =
    "default";

  if (!props.isSelected) {
    (part.value! as any).getNode().enterTween.reverse();
  }
}

watch(() => props.isSelected, (isSelected) => {
  if (isSelected) {
    (part.value! as any).getNode().enterTween.play();
  } else {
    (part.value! as any).getNode().enterTween.reverse();
  }
})
</script>

<template>
  <v-image @click="$emit('mouseClick', props.location)" @mouseenter="handleMouseEnter" @mouseleave="handleMouseLeave"
    ref="part" :config="{
      ...props.config,
      scale: {
        x: scale,
        y: scale,
      },
    }" />
</template>
