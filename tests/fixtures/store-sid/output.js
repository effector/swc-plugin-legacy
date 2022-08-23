import { createStore } from 'effector';
const $demo = createStore(0, {
    sid: "27smw5if358nn",
    name: "$demo"
});
const $demoWithOld = createStore(0, {
    sid: "3leemqv6ynndz",
    and: {
        name: 'what?'
    },
    name: "$demoWithOld"
});
