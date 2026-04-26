<template>
  <ErrorBoundary>
    <router-view />
    <ToastContainer />
  </ErrorBoundary>
</template>

<script setup lang="ts">
import ErrorBoundary from '@/components/common/ErrorBoundary.vue'
import ToastContainer from '@/components/common/ToastContainer.vue'
</script>
