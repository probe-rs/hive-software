<script setup lang="ts">
import Login from "@/components/Login.vue";
import logo from "@/assets/probe-rs-color-primary.svg";

</script>

<template>
    <v-main>
        <v-container fluid>
            <v-row class="justify-center align-stretch align-self-center">
                <v-col md="6" class="ma-6" style="max-height: 350px;">
                    <v-img :src="logo" alt="logo" />
                </v-col>
            </v-row>
            <v-row class="justify-center align-stretch align-self-center">
                <v-col md="6">
                    <Login />
                </v-col>
            </v-row>
        </v-container>
    </v-main>
</template>