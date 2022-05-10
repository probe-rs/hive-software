<script setup lang="ts">
import { ref, onBeforeMount, onUnmounted, onMounted } from "vue";
import konva from "konva";
import {
  computed,
  reactive,
  type ComputedRef,
  type Ref,
} from "@vue/reactivity";
import { useServerData } from "@/stores/serverData";
import RackPartComponent from "./RackPart.vue";
import {
  paddingHorizontal,
  rackXpos,
  rackYpos,
  referenceStageWidth,
  stageAspectRatio,
} from "./constants";
import { storeToRefs } from "pinia";

enum PartType {
  RPI,
  PSS,
  TSS,
}

type RackPart = {
  type: PartType;
  // Location in the overall rack, including RPI and PSS
  location: number;
  // Index in TSS stack (Only used if type is TSS)
  index: number;
  // Wheter or not the part is currently selected by the user
  isSelected: boolean,
};

const emit = defineEmits<{
  (event: 'selectedPartLocation', location: number): void
}>();

const serverData = useServerData();
const { targetData } = storeToRefs(serverData);

const stageWidth = ref(1000);
const stageHeight = ref(250);
const stageScale = ref(1);
const selectedComponent = ref(undefined);

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

onBeforeMount(() => {
  window.addEventListener("resize", updateStageSize);

  //Load required images
  const hiveRpi = new window.Image();
  hiveRpi.src = "./rack/hive_rpi.png";
  hiveRpi.onload = () => {
    (hiveRpiImage! as any).value = hiveRpi;
  };

  const hivePSS = new window.Image();
  hivePSS.src = "./rack/hive_probe_stack_shield.png";
  hivePSS.onload = () => {
    (hiveProbeStackShieldImage! as any).value = hivePSS;
  };

  const hiveTSS = new window.Image();
  hiveTSS.src = "./rack/hive_target_stack_shield.png";
  hiveTSS.onload = () => {
    (hiveTargetStackShieldImage! as any).value = hiveTSS;
  };

  const hiveTSSwSpacer = new window.Image();
  hiveTSSwSpacer.src = "./rack/hive_target_stack_shield_wSpacer.png";
  hiveTSSwSpacer.onload = () => {
    (hiveTargetStackShieldImageSpacer! as any).value = hiveTSSwSpacer;
  };

  const hiveTSSwDaughterboard = new window.Image();
  hiveTSSwDaughterboard.src =
    "./rack/hive_target_stack_shield_wDaughterboard.png";
  hiveTSSwDaughterboard.onload = () => {
    (hiveTargetStackShieldImageDaughterboard! as any).value =
      hiveTSSwDaughterboard;
  };

  const hiveTSSwDaughterboardSpacer = new window.Image();
  hiveTSSwDaughterboardSpacer.src =
    "./rack/hive_target_stack_shield_wDaughterboard_wSpacer.png";
  hiveTSSwDaughterboardSpacer.onload = () => {
    (hiveTargetStackShieldImageDaughterboardSpacer! as any).value =
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

function tssConfig(
  idx: number,
  location: number,
  data: null | { state: String, data: { name: String } },
) {

  var img = undefined;
  var yVal = rackYpos - 71;

  if (idx == targetData.value.length - 1) {
    if (data) {
      img = hiveTargetStackShieldImageDaughterboard.value;
      yVal = rackYpos - 141;
    } else {
      img = hiveTargetStackShieldImage.value;
    }
  } else {
    if (data) {
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

function rpiConfig(location: number) {
  return {
    image: hiveRpiImage.value,
    x: rackXpos,
    y: rackYpos,
  };
}

function pssConfig(location: number) {
  return {
    image: hiveProbeStackShieldImage.value,
    x: rackXpos + 133,
    y: rackYpos,
  };
}

function getConfig(type: PartType, index: number, location: number) {
  switch (type) {
    case PartType.RPI:
      return rpiConfig(location);
    case PartType.PSS:
      return pssConfig(location);
    case PartType.TSS:
      return tssConfig(index, location, serverData.targetData[index]);
  }
}

function handlePartClick(location: number) {
  const prevSelectedLocation = rackParts.findIndex((e) => {
    return e.isSelected;
  });

  emit("selectedPartLocation", location);

  rackParts[prevSelectedLocation] = {
    ...rackParts[prevSelectedLocation],
    isSelected: false,
  }

  rackParts[location] = {
    ...rackParts[location],
    isSelected: true,
  }
}

const rackParts: RackPart[] = reactive([
  { type: PartType.RPI, location: 0, index: 0, isSelected: false },
  { type: PartType.PSS, location: 1, index: 0, isSelected: false },
  { type: PartType.TSS, location: 2, index: 0, isSelected: false },
  { type: PartType.TSS, location: 3, index: 1, isSelected: false },
  { type: PartType.TSS, location: 4, index: 2, isSelected: false },
  { type: PartType.TSS, location: 5, index: 3, isSelected: false },
  { type: PartType.TSS, location: 6, index: 4, isSelected: false },
  { type: PartType.TSS, location: 7, index: 5, isSelected: false },
  { type: PartType.TSS, location: 8, index: 6, isSelected: false },
  { type: PartType.TSS, location: 9, index: 7, isSelected: false },
]);
</script>

<template>
  <v-col id="konvaStage" ref="konvaStage" style="border-radius: 8px" cols="lg-10">
    <v-stage :config="stageConfig" ref="stage">
      <v-layer ref="layer">
        <RackPartComponent v-for="part in rackParts" :type="part.type"
          :config="getConfig(part.type, part.index, part.location)" :location="part.location"
          :isSelected="part.isSelected" @mouseClick="handlePartClick" />
      </v-layer>
    </v-stage>
  </v-col>
</template>
