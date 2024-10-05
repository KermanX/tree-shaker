<script setup lang="ts">
import { useTemplateRef, watchEffect } from 'vue';
import { activeCallScope, currentCallScopes, activeLogIndex, logsRaw, currentCfScopes, activeCfScope, activeVarScope, currentVarScopes } from './states';

const logItems = useTemplateRef("logItems");

function getLogItemClass(index: number) {
  return {
    'text-gray-200 bg-gray-800 bg-op-60': index < activeLogIndex.value,
    'bg-gray-600 bg-op-60': index === activeLogIndex.value,
    'text-gray-400 bg-gray-700 bg-op-0': index > activeLogIndex.value,
  };
}

function getCallScopeClass(index: number) {
  return {
    'text-gray-200 bg-amber-900 bg-op-30': index > activeCallScope.value,
    'bg-amber-700 bg-op-30': index === activeCallScope.value,
    'text-gray-400 bg-amber-800 bg-op-0': index < activeCallScope.value,
  };
}

function getCfScopeClass(index: number) {
  return {
    'text-gray-200 bg-green-900 bg-op-30': index > activeCfScope.value,
    'bg-green-700 bg-op-30': index === activeCfScope.value,
    'text-gray-400 bg-green-800 bg-op-0': index < activeCfScope.value,
  };
}

function getVarScopeClass(index: number) {
  return {
    'text-gray-200 bg-blue-900 bg-op-30': index > activeVarScope.value,
    'bg-blue-700 bg-op-30': index === activeVarScope.value,
    'text-gray-400 bg-blue-800 bg-op-0': index < activeVarScope.value,
  };
}

function next() {
  activeLogIndex.value++;
}

function prev() {
  activeLogIndex.value--;
}

watchEffect(() => {
  if (activeLogIndex.value >= logsRaw.value.length) {
    activeLogIndex.value = logsRaw.value.length - 1;
  }
  if (activeLogIndex.value < 0) {
    activeLogIndex.value = 0;
  }

  logItems.value?.children[activeLogIndex.value]?.scrollIntoView({
    block: 'nearest',
    behavior: 'smooth',
  });
});

function setActiveCallScope(index: number) {
  const scope = currentCallScopes.value[index];
  activeCallScope.value = index;
  activeCfScope.value = currentCfScopes.value.length - 1 - scope.cf_scope_depth;
  activeVarScope.value = currentVarScopes.value.findIndex(varScope => varScope.id === scope.body_variable_scope);
}

function setActiveCfScope(index: number) {
  activeCfScope.value = index;
  activeCallScope.value = currentCallScopes.value.findIndex(scope => scope.cf_scope_depth <= currentCfScopes.value.length - 1 - index);
}

function setActiveVarScope(index: number) {
  const scope = currentVarScopes.value[index];
  activeVarScope.value = index;
  activeCfScope.value = currentCfScopes.value.findIndex(cfScope => cfScope.id === scope.cf_scope);
}
</script>

<template>
  <div flex flex-col gap-y-1 text-sm>
    <div flex gap-x-2 b-t-1 border-gray-600 b-solid pt-2 px-1>
      Events
      <div flex-grow />
      <button @click="prev" :disabled="activeLogIndex <= 0">Prev</button>
      <button @click="next" :disabled="activeLogIndex >= logsRaw.length - 1">Next</button>
    </div>
    <div font-mono flex-grow h-0 overflow-auto scroll-hidden>
      <div ref="logItems" my-2>
        <div v-for="log, index in logsRaw" class="hover:bg-op-100" :class="getLogItemClass(index)"
          @click="activeLogIndex = index">
          <div ml-2>
            {{ log }}
          </div>
        </div>
      </div>
    </div>

    <div font-mono flex-grow h-0 b-t-1 border-gray-600 b-solid flex flex-col>
      <div grid grid-cols-3 gap-x-2 my-.5>
        <div>
          Call Scopes
        </div>
        <div>
          Cf Scopes
        </div>
        <div>
          Var Scopes
        </div>
      </div>
      <div h-0 flex-grow grid grid-cols-3 gap-x-2 class="scopes-lists">
        <div>
          <div v-for="scope, index in currentCallScopes" flex class="hover:bg-op-40" :class="getCallScopeClass(index)"
            @click="setActiveCallScope(index)">
            <div flex-grow ml-2>
              [{{ index }}]
              cf[{{ scope.cf_scope_depth }}]
              var#{{ scope.body_variable_scope }}
            </div>
          </div>
        </div>
        <div>
          <div v-for="scope, index in currentCfScopes" flex class="hover:bg-op-40" :class="getCfScopeClass(index)"
            @click="setActiveCfScope(index)">
            <div flex-grow ml-2>
              [{{ index }}]
              #{{ scope.id }}
              {{ scope.kind }}
              {{ scope.exited }}
            </div>
          </div>
        </div>
        <div>
          <div v-for="scope, index in currentVarScopes" flex class="hover:bg-op-40" :class="getVarScopeClass(index)"
            @click="setActiveVarScope(index)">
            <div flex-grow ml-2>
              [{{ index }}]
              #{{ scope.id }}
              cf#{{ scope.cf_scope }}
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
button {
  @apply text-xs px-2 py-.5 rounded text-white b-1 b-gray-400 hover:bg-white hover:bg-op-10 active:bg-op-20 disabled:op-40 user-select-none;
}

.scopes-lists>* {
  overflow-y: auto;
  scrollbar-width: thin;
}
</style>
