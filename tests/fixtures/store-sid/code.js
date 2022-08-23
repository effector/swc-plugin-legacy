import { createStore } from 'effector';
const $demo = createStore(0)
const $demoWithOld = createStore(0, {
    name: 'what?'
})