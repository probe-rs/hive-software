<script setup lang="ts">
import TargetOverview from '@/components/TargetOverview.vue';
import { defineProps } from 'vue';
import { useServerData } from "@/stores/serverData";
import { useAppConfig } from '@/stores/appConfig';
import { storeToRefs } from 'pinia';
import { computed, toRef } from '@vue/reactivity';
import { AppTheme } from '@/plugins/vuetify';

// Assets
import ferrisGesture from "@/assets/ferris/rustacean-flat-gesture.svg";

const serverData = useServerData();
const appConfig = useAppConfig();

const { targetData } = storeToRefs(serverData)

const props = defineProps({
    tssPos: { type: Number, required: true },
})

const tssPos = computed(() => {
    return props.tssPos
});

const hasDaughterboard = computed(() => {
    if (targetData.value[tssPos.value]) {
        return true;
    }
    return false;
})

</script>

<template>
    <v-row>
        <v-col cols="12">
            <v-sheet rounded class="pa-4" color="transparent">
                <v-row class="pa-2">
                    <h2 class="align-self-center">Target Stack Shield {{ tssPos }}</h2>

                    <v-spacer />

                    <v-icon size="25" class="align-self-center"
                        :icon="hasDaughterboard ? 'mdi-card' : 'mdi-card-remove'"
                        :color="hasDaughterboard ? 'success' : 'info'" />
                    <p class="align-self-center pl-2">{{ hasDaughterboard ? 'Daughterboard Connected' :
                            'No Daughterboard Found'
                    }}</p>
                </v-row>
            </v-sheet>
            <v-divider />
        </v-col>
    </v-row>

    <template v-if="hasDaughterboard">
        <v-row>
            <v-col cols="6">
                <TargetOverview :tssPos="tssPos" :target="0" :status="false" />
            </v-col>
            <v-col cols="6">
                <TargetOverview :tssPos="tssPos" :target="1" :status="true" />
            </v-col>
        </v-row>
        <v-row>
            <v-col cols="6">
                <TargetOverview :tssPos="tssPos" :target="2" :status="true" />
            </v-col>
            <v-col cols="6">
                <TargetOverview :tssPos="tssPos" :target="3" :status="true" />
            </v-col>
        </v-row>

        <v-row>
            <v-col cols="12">
                <v-sheet rounded elevation="1" class="pa-4">
                    <v-row class="pa-2">
                        <v-btn color="secondary" variant="text">
                            Load Targets from File
                        </v-btn>
                        <v-spacer />
                        <v-btn color="success" variant="text">
                            Save and Reload
                        </v-btn>
                    </v-row>
                </v-sheet>
            </v-col>
        </v-row>
    </template>

    <template v-else>
        <v-row>
            <v-col cols="12">
                <v-sheet rounded elevation="1" class="pa-4">
                    <v-row class="pa-6">
                        <v-img :src="ferrisGesture" height="125"
                            :style="(appConfig.theme == AppTheme.Light) ? '' : 'filter: brightness(80%);'" />
                    </v-row>
                    <v-row class="pa-2 justify-center ">
                        <p class="align-self-center"
                            style="max-width: 70%; text-align: center; color: rgb(var(--v-theme-on-surface), var(--v-disabled-opacity));">
                            Could not detect any Daughterboard on this Target Stack Shield. If a Daughterboard is
                            connected but not shown in here it might be related to a hardware problem. In that case,
                            please make sure to check if the Daughterboard detect pin on the Daughterboard correctly
                            outputs 3.3V to the IO-Expander.</p>
                    </v-row>
                </v-sheet>
            </v-col>
        </v-row>
    </template>
</template>
