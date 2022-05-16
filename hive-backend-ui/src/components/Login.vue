<script setup lang="ts">
import { ref } from 'vue';

const showPassword = ref(false);
const loading = ref(false);

function dummyLoad(){
    loading.value = true;
    setTimeout(()=>{
        loading.value = false;
    }, 2000)
}
</script>

<template>
    <v-card elevation="1">
        <v-card-title>
            <h3 style="font-family: Poppins;">Hive Backend</h3>
        </v-card-title>
        <v-card-content>
            <v-text-field variant="underlined" density="compact" label="Username" />
            <v-text-field :append-icon="showPassword ? 'mdi-eye-off' : 'mdi-eye'" variant="underlined" density="compact"
                :type="showPassword ? 'text' : 'password'" label="Password"
                @click:append="showPassword = !showPassword" />
        </v-card-content>
        <v-card-actions>
            <v-spacer />
            <v-btn color="secondary" variant="text" @click="dummyLoad">Login</v-btn>
        </v-card-actions>

        <v-overlay v-model="loading" contained class="align-center justify-center">
            <v-progress-circular size="80" color="secondary" indeterminate />
        </v-overlay>
    </v-card>
</template>