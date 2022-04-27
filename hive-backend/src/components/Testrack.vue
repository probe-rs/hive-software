<script setup lang="ts">
import { ref, onBeforeMount, onUnmounted, onMounted } from "vue";
import konva from "konva";
import { computed } from "@vue/reactivity";
import { useServerData } from "@/stores/serverData";

const referenceStageWidth = 1750; // Reference width based on which all canvas objects are scaled
const paddingHorizontal = 12; // Standard Horizontal padding of v-col in px for stageWidth calculations
const stageAspectRatio = 3; // Stage Aspect ratio of height : width (1:x)
const defaultRackScale = 0.5; // Default scale of all rack components
const hoverRackScale = 0.55; // Scale of rack component if hovered
const rackXpos = 570; // X pos of first rack component
const rackYpos = 160; // Y pos of first rack component (and all consecutive components)

const serverData = useServerData();
const stageWidth = ref(1000);
const stageHeight = ref(250);
const stageScale = ref(1);
const rpiScale = ref(defaultRackScale);
const pssScale = ref(defaultRackScale);
const tssScales = ref([
    defaultRackScale,
    defaultRackScale,
    defaultRackScale,
    defaultRackScale,
    defaultRackScale,
    defaultRackScale,
    defaultRackScale,
    defaultRackScale,
]);
const selectedComponent = ref(undefined);

// Dynamically loaded images
const hiveRpiImage = ref<HTMLImageElement>();
const hiveProbeStackShieldImage = ref<HTMLImageElement>();
const hiveTargetStackShieldImage = ref<HTMLImageElement>();
const hiveTargetStackShieldImageDaughterboardSpacer = ref<HTMLImageElement>();
const hiveTargetStackShieldImageDaughterboard = ref<HTMLImageElement>();
const hiveTargetStackShieldImageSpacer = ref<HTMLImageElement>();
const hiveDaughterboardImage = ref<HTMLImageElement>();
const hiveSpacerImage = ref<HTMLImageElement>();

// Konva stage
const konvaStage = ref(null);
const stage = ref(null);

// Dummy placeholder data
const hive = {
    tss: [
        { hasDaughterboard: true },
        { hasDaughterboard: false },
        { hasDaughterboard: false },
        { hasDaughterboard: true },
        { hasDaughterboard: false },
        { hasDaughterboard: false },
        { hasDaughterboard: true },
        { hasDaughterboard: true },
    ],
};

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
        (hiveTargetStackShieldImageDaughterboard! as any).value = hiveTSSwDaughterboard;
    };

    const hiveTSSwDaughterboardSpacer = new window.Image();
    hiveTSSwDaughterboardSpacer.src =
        "./rack/hive_target_stack_shield_wDaughterboard_wSpacer.png";
    hiveTSSwDaughterboardSpacer.onload = () => {
        (hiveTargetStackShieldImageDaughterboardSpacer! as any).value =
            hiveTSSwDaughterboardSpacer;
    };
})

onMounted(() => {
    updateStageSize();
})

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

function tssConfig(idx: number, data: (string | {
    Known: {
        name: string;
        architecture: null;
        memory_address: null;
        status: ObjectConstructor[];
    };
})[] | null) {
    var img = undefined;
    var yVal = rackYpos - 71;

    if (idx == serverData.targetData.length - 1) {
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
        scale: {
            x: tssScales.value[idx],
            y: tssScales.value[idx],
        },
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
        //container: "konvaStage",
    };
});

const rpiConfig = computed(() => {
    return {
        image: hiveRpiImage.value,
        x: rackXpos,
        y: rackYpos,
        scale: {
            x: rpiScale.value,
            y: rpiScale.value,
        },
    };
})

const pssConfig = computed(() => {
    return {
        image: hiveProbeStackShieldImage.value,
        x: rackXpos + 133,
        y: rackYpos,
        scale: {
            x: pssScale.value,
            y: pssScale.value,
        },
    };
})
</script>

<template>
    <v-col id="konvaStage" ref="konvaStage" style="border-radius: 8px" cols="lg-10">
        <v-stage :config="stageConfig" ref="stage">
            <v-layer ref="layer">
                <v-image ref="rpi" :config="rpiConfig" @mouseenter="rpiScale = hoverRackScale"
                    @mouseleave="rpiScale = defaultRackScale" />

                <v-image ref="pss" :config="pssConfig" @mouseenter="pssScale = hoverRackScale"
                    @mouseleave="pssScale = defaultRackScale" />

                <v-image v-for="(data, idx) in serverData.targetData" v-bind:key="idx" :ref="'tss' + idx" :config="
                tssConfig(
                    idx,
                    data,
                )" @mouseenter="tssScales[idx] = hoverRackScale" @mouseleave="tssScales[idx] = defaultRackScale" />
            </v-layer>
        </v-stage>
    </v-col>
</template>