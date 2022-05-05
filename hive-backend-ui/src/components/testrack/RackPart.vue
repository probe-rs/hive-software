<script setup lang="ts">
import {
  computed,
  defineProps,
  type PropType,
  onBeforeMount,
  ref,
  onMounted,
  watch,
  reactive,
} from "vue";
import type { PartType } from "./types";
import { defaultRackScale } from "./constants";
import Konva from "konva";

const props = defineProps({
  type: {
    type: Number as PropType<PartType>,
    required: true,
  },
  config: Object,
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
  (part.value! as any).getNode().enterTween.play();
}

function handleMouseLeave() {
  (part.value! as any).getNode().getStage().container().style.cursor =
    "default";
  (part.value! as any).getNode().enterTween.reverse();
}
</script>

<template>
  <v-image
    @mouseenter="handleMouseEnter"
    @mouseleave="handleMouseLeave"
    ref="part"
    :config="{
      ...props.config,
      scale: {
        x: scale,
        y: scale,
      },
    }"
  />
</template>
