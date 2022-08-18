<script setup lang="ts">
import { type PropType, ref, onMounted, watch, toRefs } from "vue";
import type { PartType } from "./types";
import { defaultRackScale } from "./constants";
import Konva from "konva";

type RackPartConfig = {
  image?: HTMLImageElement;
  x: number;
  y: number;
};

type VueKonvaImage = {
  getNode(): VueKonvaImageTween;
};

interface VueKonvaImageTween extends Konva.Image {
  enterTween: Konva.Tween;
}

const props = defineProps({
  location: {
    type: Number,
    required: true,
  },
  config: {
    type: Object as PropType<RackPartConfig>,
    required: true,
  },
  isSelected: {
    type: Boolean,
    required: true,
  },
});

const { location, config, isSelected } = toRefs(props);

onMounted(() => {
  setTween(config.value.y);

  const node = (part.value! as VueKonvaImage).getNode();

  // Reset the tween to the new image size in case the image changed (for example a Daughterboard has been inserted)
  node.on("imageChange", () => {
    node.y(config.value.y);

    setTween(config.value.y);

    if (isSelected.value) {
      node.enterTween.play();
    }
  });
});

function setTween(y: number) {
  if (!part.value) {
    return;
  }

  const node = (part.value as VueKonvaImage).getNode();

  node.enterTween = new Konva.Tween({
    node: node,
    y: y - 20,
    duration: 0.2,
    easing: Konva.Easings.EaseOut,
  });
}

watch(isSelected, (isSelected) => {
  if (!part.value) {
    return;
  }

  if (isSelected) {
    (part.value as VueKonvaImage).getNode().enterTween.play();
  } else {
    (part.value as VueKonvaImage).getNode().enterTween.reverse();
  }
});

const scale = ref(defaultRackScale);
const part = ref(null);

function handleMouseEnter() {
  if (!part.value) {
    return;
  }

  (part.value as any).getNode().getStage().container().style.cursor = "pointer";

  if (!isSelected.value) {
    (part.value as VueKonvaImage).getNode().enterTween.play();
  }
}

function handleMouseLeave() {
  if (!part.value) {
    return;
  }

  (part.value as any).getNode().getStage().container().style.cursor = "default";

  if (!isSelected.value) {
    (part.value as VueKonvaImage).getNode().enterTween.reverse();
  }
}
</script>

<template>
  <v-image
    @click="$emit('mouseClick', location)"
    @mouseenter="handleMouseEnter"
    @mouseleave="handleMouseLeave"
    ref="part"
    :config="{
      ...config,
      scale: {
        x: scale,
        y: scale,
      },
    }"
  />
</template>
