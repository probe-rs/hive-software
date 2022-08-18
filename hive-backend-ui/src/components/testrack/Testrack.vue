<script setup lang="ts">
import type { BackendQuery, FlatTargetState } from "@/gql/backend";
import type { Maybe } from "@/gql/baseTypes";

import { ref, onBeforeMount, onUnmounted, onMounted } from "vue";
import { computed } from "@vue/reactivity";
import RackPartComponent from "./RackPart.vue";
import {
  paddingHorizontal,
  rackXpos,
  rackYpos,
  referenceStageWidth,
  stageAspectRatio,
} from "./constants";
import { useQuery } from "@vue/apollo-composable";
import gql from "graphql-tag";

const emit = defineEmits<{
  (event: "selectedPartLocation", location: number): void;
}>();

const stageWidth = ref(1000);
const stageHeight = ref(250);
const stageScale = ref(1);
const isSelected = ref([
  false,
  false,
  false,
  false,
  false,
  false,
  false,
  false,
  false,
  false,
]);

// Dynamically loaded images
const hiveRpiImage = ref<HTMLImageElement>();
const hiveProbeStackShieldImage = ref<HTMLImageElement>();
const hiveTargetStackShieldImage = ref<HTMLImageElement>();
const hiveTargetStackShieldImageDaughterboardSpacer = ref<HTMLImageElement>();
const hiveTargetStackShieldImageDaughterboard = ref<HTMLImageElement>();
const hiveTargetStackShieldImageSpacer = ref<HTMLImageElement>();

// Konva stage
const konvaStage = ref(null);
const stage = ref(null);

// Server data
const { result } = useQuery<BackendQuery>(gql`
  query {
    connectedTss
    assignedTargets {
      state
    }
  }
`);

const connectedTss = computed(() => {
  if (result.value) {
    return result.value.connectedTss;
  }
  return [false, false, false, false, false, false, false, false];
});

const assignedTargets = computed(() => {
  if (result.value) {
    return result.value.assignedTargets;
  }
  return [null, null, null, null, null, null, null, null];
});

onBeforeMount(() => {
  window.addEventListener("resize", updateStageSize);

  //Load required images
  const hiveRpi = new window.Image();
  hiveRpi.src = "./rack/hive_rpi.png";
  hiveRpi.onload = () => {
    hiveRpiImage.value = hiveRpi;
  };

  const hivePSS = new window.Image();
  hivePSS.src = "./rack/hive_probe_stack_shield.png";
  hivePSS.onload = () => {
    hiveProbeStackShieldImage.value = hivePSS;
  };

  const hiveTSS = new window.Image();
  hiveTSS.src = "./rack/hive_target_stack_shield.png";
  hiveTSS.onload = () => {
    hiveTargetStackShieldImage.value = hiveTSS;
  };

  const hiveTSSwSpacer = new window.Image();
  hiveTSSwSpacer.src = "./rack/hive_target_stack_shield_wSpacer.png";
  hiveTSSwSpacer.onload = () => {
    hiveTargetStackShieldImageSpacer.value = hiveTSSwSpacer;
  };

  const hiveTSSwDaughterboard = new window.Image();
  hiveTSSwDaughterboard.src =
    "./rack/hive_target_stack_shield_wDaughterboard.png";
  hiveTSSwDaughterboard.onload = () => {
    hiveTargetStackShieldImageDaughterboard.value = hiveTSSwDaughterboard;
  };

  const hiveTSSwDaughterboardSpacer = new window.Image();
  hiveTSSwDaughterboardSpacer.src =
    "./rack/hive_target_stack_shield_wDaughterboard_wSpacer.png";
  hiveTSSwDaughterboardSpacer.onload = () => {
    hiveTargetStackShieldImageDaughterboardSpacer.value =
      hiveTSSwDaughterboardSpacer;
  };
});

onMounted(() => {
  updateStageSize();
});

function updateStageSize() {
  if (!konvaStage.value) {
    return;
  }
  stageWidth.value =
    (konvaStage.value! as any).$el.offsetWidth - 2 * paddingHorizontal;

  stageHeight.value = stageWidth.value / stageAspectRatio;

  stageScale.value = stageWidth.value / referenceStageWidth;
}

onUnmounted(() => {
  window.removeEventListener("resize", updateStageSize);
});

function tssConfig(idx: number, daugtherboard: Maybe<Array<FlatTargetState>>) {
  var img = undefined;
  var yVal = rackYpos - 71;

  if (idx == showTssIndexes.value.length - 1) {
    if (daugtherboard) {
      img = hiveTargetStackShieldImageDaughterboard.value;
      yVal = rackYpos - 141;
    } else {
      img = hiveTargetStackShieldImage.value;
    }
  } else {
    if (daugtherboard) {
      img = hiveTargetStackShieldImageDaughterboardSpacer.value;
      yVal = rackYpos - 141;
    } else {
      img = hiveTargetStackShieldImageSpacer.value;
    }
  }
  return {
    image: img,
    x: rackXpos + 247 + idx * 114,
    y: yVal,
  };
}

const stageConfig = computed(() => {
  return {
    width: stageWidth.value,
    height: stageHeight.value,
    scale: {
      x: stageScale.value,
      y: stageScale.value,
    },
  };
});

const rpiConfig = computed(() => {
  return {
    image: hiveRpiImage.value,
    x: rackXpos,
    y: rackYpos,
  };
});

const pssConfig = computed(() => {
  return {
    image: hiveProbeStackShieldImage.value,
    x: rackXpos + 133,
    y: rackYpos,
  };
});

function handlePartClick(location: number) {
  const prevSelectedLocation = isSelected.value.findIndex(
    (isSelected) => isSelected,
  );

  if (prevSelectedLocation === location) {
    location = -1;
    emit("selectedPartLocation", location);

    isSelected.value[prevSelectedLocation] = false;
    return;
  }

  emit("selectedPartLocation", location);

  isSelected.value[prevSelectedLocation] = false;

  isSelected.value[location] = true;
}

const showTssIndexes = computed(() => {
  if (connectedTss.value) {
    let locations: number[] = [];

    connectedTss.value.forEach((isConnected: boolean, idx: number) => {
      if (isConnected) {
        locations.push(idx);
      }
    });

    return locations;
  }
  return [];
});
</script>

<template>
  <v-col
    id="konvaStage"
    ref="konvaStage"
    style="border-radius: 8px"
    cols="lg-10"
  >
    <v-stage :config="stageConfig" ref="stage">
      <v-layer ref="layer">
        <RackPartComponent
          :config="rpiConfig"
          :location="0"
          :isSelected="isSelected[0]"
          @mouseClick="handlePartClick"
        />
        <RackPartComponent
          :config="pssConfig"
          :location="1"
          :isSelected="isSelected[1]"
          @mouseClick="handlePartClick"
        />
        <RackPartComponent
          v-for="idx in showTssIndexes"
          :config="tssConfig(idx, assignedTargets[idx])"
          :location="idx + 2"
          :isSelected="isSelected[idx + 2]"
          @mouseClick="handlePartClick"
        />
      </v-layer>
    </v-stage>
  </v-col>
</template>
