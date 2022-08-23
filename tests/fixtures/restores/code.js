import {createEffect, restore} from "effector";

const myFx = createEffect(() => {});
const $store = restore(myFx, 0);
