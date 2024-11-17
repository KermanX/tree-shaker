<script setup lang="ts">
import Input from './Input.vue';
import Logs from './Logs.vue';
import Editor from './Editor.vue';
import { hideDiagnostics, debouncedInput, doMinify, doTreeShake, diagnostics, load, output, showLogs, onlyMinifiedSize, treeShakedMinifiedSize, treeShakedUnminifiedSize, treeShakeRate } from './states';
</script>

<template>
  <div py-2 md:py-4 fixed inset-0 flex flex-col>
    <div px-4 flex flex-wrap gap-x-2 pb-2>
      <h1 text-xl md:text-3xl font-bold md:pb-2 select-none flex flex-wrap items-center gap-x-2>
        <img src="/favicon.ico" h-1em bg-gray-200 rounded-lg>
        <div @click="load(true)">
          Tree Shaker
        </div>
        <div text-sm self-end flex items-center gap-1 op-80>
          Experimental Tree Shaker for JS Based on Oxc
          <a i-carbon-logo-github flex-grow inline-block w-1.2em h-1.2em hover:op-80
            href="https://github.com/KermanX/tree-shaker" target="_blank" />
        </div>
      </h1>
      <div flex-grow />
      <div flex w-fit md:flex-col h-min md:h-0 z-10 gap-x-4 font-mono items-end mr-2>
        <label flex align-center gap-1 select-none>
          <span op-80>
            Tree shake:
          </span>
          <input v-model="doTreeShake" type="checkbox" placeholder="ast_builder">
        </label>
        <label flex align-center gap-1 select-none>
          <span op-80>
            Minify by Oxc:
          </span>
          <input v-model="doMinify" type="checkbox">
        </label>
      </div>
    </div>
    <div flex-grow h-0 flex flex-col md:flex-row gap-x-2 gap-y-2>
      <div :class="showLogs ? 'flex-grow-2' : 'flex-grow'" h-0 md:h-full md:w-0 flex flex-col>
        <div flex items-center>
          <h2 md:text-xl pb-2 pl-4 select-none>
            Input
            <span text-sm op80 font-mono>
              (Raw: {{ debouncedInput.length }}B, Minified: {{ onlyMinifiedSize }}B)
            </span>
          </h2>
        </div>
        <Input class="flex-grow h-0 max-h-full" />
      </div>
      <div flex-grow h-0 md:h-full md:w-0 flex flex-col>
        <h2 md:text-xl pb-2 pl-4 select-none flex items-center>
          Output
          <span text-sm font-mono self-end ml-2 mb--1>
            <span op80>(Raw: {{ treeShakedUnminifiedSize }}B,
              Minified: {{ treeShakedMinifiedSize }}B, </span>
            <math display="inline">
              <mfrac>
                <mi>Output Minified</mi>
                <mi>Input Minified</mi>
              </mfrac>
            </math>={{ treeShakeRate.toFixed(2) }}%<span op80>)</span>
          </span>
          <div flex-grow />
          <button v-if="!showLogs" @click="showLogs = true" mr-4 op-80 w-5 h-5 b-none i-carbon-asset-view />
        </h2>
        <div flex-grow relative max-h-full>
          <Editor v-model="output" lang="javascript" readonly class="w-full h-full max-h-full" />
          <div z-20 absolute left-1 right-2 bottom--2 children:p-2 children:px-3 children:b-2 children:rounded flex
            flex-col gap-2>
            <div v-if="diagnostics.length" v-show="!hideDiagnostics" relative text-yellow-200 bg-yellow-900 bg-op-80 b-yellow-500>
              <h3 text-lg pb-1>
                Warning
              </h3>
              <div font-mono max-h-8em overflow-y-auto>
                <p v-for="d, i in diagnostics" :key="i" style="text-indent: -1em" ml-1em>
                  {{ d }}
                </p>
              </div>
              <button absolute right-3 top-3 w-6 h-6 b-none i-carbon-close @click="hideDiagnostics = true" />
            </div>
          </div>
        </div>
      </div>
      <div v-show="showLogs" flex-grow-2 h-0 md:h-full md:w-0 flex flex-col>
        <h2 md:text-xl pb-2 pl-4 select-none flex items-end>
          Logs
          <div flex-grow />
          <button i-carbon-close mr-4 op-80 w-6 h-6 b-none @click="showLogs = false" />
        </h2>
        <Logs class="flex-grow relative h-full" />
      </div>
    </div>
  </div>
</template>
